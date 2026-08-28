use super::*;

/// リソース
pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.route("", web::to(index));
	cfg.service(web::resource("setting").get(view_setting).patch(patch_setting));
}

#[derive(serde::Serialize)]
struct Log {
	id: i64,
	timestamp: i64,
	body: String,
}

async fn index(id: Identity, _: StateHandle, pool: web::Data<Pool>, tmpl: web::Data<tera::Tera>) -> common::Result<impl Responder> {
	let pool = pool.as_ref();

	// favはlocalStorageに持っているので、クライアント側で諸々のAPIを叩く　バックエンドではあんまり何もしない
	let log_list = sqlx::query_as!(Log, "SELECT id,timestamp,body FROM log WHERE actor=? ORDER BY id DESC LIMIT 20", *id).fetch_all(pool).await?;

	let mut ctx = tera::Context::new();
	ctx.insert("log_list", &log_list);
	let body = Page::default().actor_data(ActorData::load(&id, pool).await?).render_with_ctx("home.html", &tmpl, ctx)?;
	Ok(HttpResponse::Ok().body(body))
}

async fn view_setting(id: Identity, _: StateHandle, pool: web::Data<Pool>, tmpl: web::Data<tera::Tera>) -> common::Result<impl Responder> {
	let pool = pool.as_ref();
	let record = sqlx::query!("SELECT comment,profile,icon_list,portrait_list FROM actor WHERE id=?", *id).fetch_one(pool).await?;

	let mut ctx = tera::Context::new();
	ctx.insert("comment", &record.comment);
	ctx.insert("profile", &record.profile);
	ctx.insert("icon_list", &record.icon_list);
	ctx.insert("portrait_list", &record.portrait_list);

	let body = Page::default().actor_data(ActorData::load(&id, pool).await?).render_with_ctx("setting.html", &tmpl, ctx)?;
	Ok(HttpResponse::Ok().body(body))
}

#[derive(serde::Deserialize, Validate)]
#[validate(schema(function = "any_some"))]
struct Setting {
	#[validate(length(max = 16, message = "16文字以内で入力してください"))]
	name: Option<String>,
	#[validate(length(max = 48, message = "48文字以内で入力してください"))]
	comment: Option<String>,
	#[validate(length(max = 8192, message = "8192文字以内で入力してください"))]
	profile: Option<String>,
	#[validate(length(max = 8192, message = "合計8192文字以内にしてください"))]
	icon_list: Option<String>,
	#[validate(length(max = 4096, message = "合計4096文字以内にしてください"))]
	portrait_list: Option<String>,
}
async fn patch_setting(web::Json(info): web::Json<Setting>, id: Identity, state: StateHandle, pool: web::Data<Pool>) -> common::Result<impl Responder> {
	state.get().only_open()?;
	info.validate()?;
	let mut builder = sqlx::QueryBuilder::new("UPDATE actor SET ");
	let mut sep = builder.separated(',');

	if let Some(v) = &info.name {
		sep.push("name=");
		sep.push_bind_unseparated(v);
	}
	if let Some(v) = &info.comment {
		sep.push("comment=");
		sep.push_bind_unseparated(v);
	}
	if let Some(v) = &info.profile {
		sep.push("profile=");
		sep.push_bind_unseparated(v);
	}
	if let Some(v) = &info.icon_list {
		let v = format_list(v);
		sep.push("icon_list=");
		sep.push_bind_unseparated(v);
	}
	if let Some(v) = &info.portrait_list {
		let v = format_list(v);
		sep.push("portrait_list=");
		sep.push_bind_unseparated(v);
	}
	builder.push(" WHERE id=");
	builder.push_bind(*id);

	let pool = pool.as_ref();
	builder.build().execute(pool).await?;
	Ok(HttpResponse::NoContent().finish())
}

fn any_some(v: &Setting) -> Result<(), validator::ValidationError> {
	if v.name.is_some() || v.comment.is_some() || v.profile.is_some() || v.icon_list.is_some() || v.portrait_list.is_some() {
		Ok(())
	} else {
		let mut err = validator::ValidationError::new("empty_setting");
		err.message = Some(std::borrow::Cow::Borrowed("少なくとも1つの項目を変更してください"));
		Err(err)
	}
}

fn format_list(v: &str) -> String {
	v.lines()
		.filter_map(|l| {
			let l = l.trim();
			if !l.is_empty() { Some(l) } else { None }
		})
		.collect::<Vec<_>>()
		.join("\n")
}
