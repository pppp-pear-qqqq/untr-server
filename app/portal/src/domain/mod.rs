mod auth;
mod entry;
mod home;
mod user;

use actix_web::{HttpResponse, Responder, http::header, web};
use common::PageRender;
use sqlx::SqlitePool;
use tera::Tera;

use crate::utils::{Identity, Page, UserData};

pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.route("/", web::get().to(index));
	cfg.service(web::scope("entry").configure(entry::cfg));
	cfg.service(web::scope("auth").configure(auth::cfg));
	cfg.service(web::scope("home").configure(home::cfg));
	cfg.service(web::scope("user").configure(user::cfg));
	cfg.route("info", web::get().to(index));
}

async fn index(id: Option<Identity>, pool: web::Data<SqlitePool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	let body = Page::default().user_data_opt(UserData::load_opt(&id, &pool).await?).render("index.html", &tmpl)?;
	Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
}
