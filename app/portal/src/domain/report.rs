use super::*;

pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.service(web::resource("").get(index).post(post));
}

async fn index(id: Option<Identity>, _: StateHandle, pool: web::Data<Pool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	let body = Page::default().user_data_opt(UserData::load_opt(&id, &pool).await?).render("report.html", &tmpl)?;
	Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
}

#[derive(serde::Deserialize, Validate)]
struct Post {
	#[validate(length(max = 32, message = "32文字以内で入力してください"))]
	app_name: String,
	#[validate(length(max = 32, message = "32文字以内で入力してください"))]
	category: String,
	#[validate(length(max = 32, message = "32文字以内で入力してください"))]
	title: String,
	#[validate(length(min = 8, max = 1500, message = "8文字以上1500文字以内で入力してください"))]
	body: String,
}
async fn post(web::Form(info): web::Form<Post>, id: Option<Identity>, state: StateHandle, pool: web::Data<Pool>) -> common::Result<impl Responder> {
	let timestamp = chrono::Utc::now().timestamp();
	state.get().only_active()?;
	info.validate()?;

	let id = id.as_deref();
	let tag = format!("{} {}", info.app_name, info.category);
	let body = format!("## {}\n{}", info.title, info.body);

	let pool = pool.as_ref();
	sqlx::query!("INSERT INTO report(timestamp,user,tag,body) VALUES(?,?,?,?)", timestamp, id, tag, body).execute(pool).await?;

	if let Some(id) = id {
		super::webhook::Content {
			content: format!("連絡を受け付けました。\n>>> {}", body),
			username: Some("untroche".into()),
			avatar_url: None,
		}
		.target(vec![Uuid::from_slice(id)?])
		.send(pool)
		.await?;
	}

	Ok(HttpResponse::NoContent().finish())
}
