use actix_session::Session;
use actix_web::{HttpResponse, Responder, http::header, web};
use sqlx::SqlitePool;

use crate::utils::{Identity, Page};

/// リソース
pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.route("", web::get().to(index));
	cfg.route("logout", web::to(logout));
}

/// ログイン・登録画面の表示
async fn index(id: Option<Identity>, pool: web::Data<SqlitePool>, tmpl: web::Data<tera::Tera>) -> common::Result<impl Responder> {
	let ctx = Page::standard_and_load(&id, &pool).await?.ctx()?;
	let body = tmpl.render("entry.html", &ctx)?;
	Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
}

async fn logout(session: Session) -> common::Result<impl Responder> {
	Identity::remove(&session);
	Ok(HttpResponse::NoContent().finish())
}
