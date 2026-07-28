mod actor;
mod entry;
mod home;

use actix_web::{HttpResponse, Responder, http::header, web};
use sqlx::SqlitePool;
use tera::Tera;

pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.route("/", web::get().to(index));
	cfg.service(web::scope("entry").configure(entry::cfg));
	cfg.service(web::scope("home").configure(home::cfg));
	cfg.service(web::scope("actor").configure(actor::cfg));
	cfg.route("info", web::get().to(index));
}

async fn index(id: Option<crate::utils::Identity>, pool: web::Data<SqlitePool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	let ctx = crate::utils::Page::standard_and_load(&id, &pool).await?.ctx()?;
	let body = tmpl.render("index.html", &ctx)?;
	Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
}
