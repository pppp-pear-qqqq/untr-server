use actix_web::{HttpResponse, Responder, error::*, web};
use common::PageRender;
use sqlx::SqlitePool;

use crate::utils::{ActorData, Identity, Page};

/// リソース
pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.route("", web::to(index));
	cfg.service(web::resource("setting").route(web::get().to(setting)));
}

async fn index(id: Identity, pool: web::Data<SqlitePool>, tmpl: web::Data<tera::Tera>) -> common::Result<impl Responder> {
	let pool = pool.as_ref();
	let body = Page::default().actor_data(ActorData::load(&id, pool).await?).render("home.html", &tmpl)?;
	Ok(HttpResponse::Ok().body(body))
}

async fn setting(id: Identity, pool: web::Data<SqlitePool>, tmpl: web::Data<tera::Tera>) -> common::Result<impl Responder> {
	let pool = pool.as_ref();
	let body = Page::default().actor_data(ActorData::load(&id, pool).await?).render("setting.html", &tmpl)?;
	Ok(HttpResponse::Ok().body(body))
}
