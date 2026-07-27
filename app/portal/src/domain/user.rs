use actix_web::{HttpResponse, Responder, http::header, web};
use common::ReqType;
use sqlx::SqlitePool;
use tera::Tera;

use crate::utils::{Identity, Page};

pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.route("", web::get().to(list));
	cfg.route("{user}", web::get().to(user));
}

async fn list(web::Form(pagination): web::Form<common::Pagination>, req_type: ReqType, id: Option<Identity>, pool: web::Data<SqlitePool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	let offset = pagination.offset as i64;
	let limit = pagination.limit as i64;

	let pool = pool.as_ref();
	let records = sqlx::query_scalar!("SELECT name FROM user LIMIT ?,?", offset, limit).fetch_all(pool).await?;

	match req_type {
		ReqType::Empty => Ok(HttpResponse::Ok().json(records)),
		_ => {
			let mut ctx = Page::standard_and_load(&id, &pool).await?.ctx()?;
			ctx.insert("list", &records);
			let body = tmpl.render("list.html", &ctx)?;
			Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
		}
	}
}

async fn user(user: web::Path<String>, id: Option<Identity>, pool: web::Data<SqlitePool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	let name = user.into_inner();

	let pool = pool.as_ref();
	let record = sqlx::query!("SELECT name,profile FROM user WHERE name=?", name).fetch_one(pool).await?;

	let mut ctx = Page::standard_and_load(&id, &pool).await?.ctx()?;
	ctx.insert("username", &record.name);
	ctx.insert("profile", &record.profile);
	let body = tmpl.render("user.html", &ctx)?;
	Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
}
