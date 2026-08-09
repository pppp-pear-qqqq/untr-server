mod actor;
mod entry;
mod location;

use actix_web::{HttpResponse, Responder, http::header, web};
use common::PageRender;
use sqlx::SqlitePool;
use tera::Tera;

use crate::utils::{ActorData, Identity, Page};

pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.route("/", web::get().to(index));
	cfg.service(web::scope("entry").configure(entry::cfg));
	cfg.service(web::scope("actor").configure(actor::cfg));
	cfg.service(web::scope("location").configure(location::cfg));
	cfg.route("info", web::get().to(info));
}

async fn index(id: Option<Identity>, pool: web::Data<SqlitePool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	let body = Page::default().actor_data_opt(ActorData::load_opt(&id, &pool).await?).render("index.html", &tmpl)?;
	Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
}

async fn info(id: Option<Identity>, pool: web::Data<SqlitePool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	let body = Page::default().actor_data_opt(ActorData::load_opt(&id, &pool).await?).render("info.html", &tmpl)?;
	Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
}
