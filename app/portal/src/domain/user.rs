use super::*;

pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.route("", web::get().to(list));
	cfg.route("{user}", web::get().to(user));
}

async fn list(web::Query(page): web::Query<common::Pagination>, req_type: ReqType, id: Option<Identity>, pool: web::Data<SqlitePool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	let offset = page.offset as i64;
	let limit = page.limit as i64;

	let pool = pool.as_ref();
	let records = sqlx::query_scalar!("SELECT name FROM user LIMIT ?,?", offset, limit).fetch_all(pool).await?;

	match req_type {
		ReqType::Empty => Ok(HttpResponse::Ok().json(records)),
		_ => {
			let mut ctx = tera::Context::new();
			ctx.insert("list", &records);
			let body = Page::default().user_data_opt(UserData::load_opt(&id, &pool).await?).render_with_ctx("user_list.html", &tmpl, ctx)?;
			Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
		}
	}
}

async fn user(user: web::Path<String>, id: Option<Identity>, pool: web::Data<SqlitePool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	#[derive(serde::Serialize)]
	struct Record {
		name: String,
		profile: String,
	}

	let name = user.into_inner();

	let pool = pool.as_ref();
	let record = sqlx::query_as!(Record, "SELECT name,profile FROM user WHERE name=?", name).fetch_one(pool).await?;

	let mut ctx = tera::Context::new();
	ctx.insert("target", &record);
	let body = Page::default().title(&format!("{} - untroche.portal", record.name)).user_data_opt(UserData::load_opt(&id, &pool).await?).render_with_ctx("user.html", &tmpl, ctx)?;
	Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
}
