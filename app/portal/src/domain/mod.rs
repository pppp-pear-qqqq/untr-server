mod auth;
mod entry;
mod home;
mod user;

use actix_web::{HttpResponse, Responder, http::header, web};
use tera::Tera;

pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.route("/", web::get().to(index));
	cfg.service(web::scope("entry").configure(entry::cfg));
	cfg.service(web::scope("auth").configure(auth::cfg));
	cfg.service(web::scope("home").configure(home::cfg));
	cfg.service(web::scope("user").configure(user::cfg));
	cfg.route("info", web::get().to(index));
}

async fn index(tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	let ctx = crate::utils::Page::default().ctx()?;
	let body = tmpl.render("index.html", &ctx)?;
	Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
}
