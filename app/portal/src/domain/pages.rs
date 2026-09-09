use std::fs;

use super::*;

pub async fn index(id: Option<Identity>, pool: web::Data<Pool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	let body = fs::read_to_string(resource("html/index.html"))?;
	let mut ctx = tera::Context::new();
	ctx.insert("body", &body);
	let body = Page::default().user_data_opt(UserData::load_opt(&id, &pool).await?).render_with_ctx("note.html", &tmpl, ctx)?;
	Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
}

pub async fn info(id: Option<Identity>, pool: web::Data<Pool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	let body = fs::read_to_string(resource("html/info.html"))?;
	let mut ctx = tera::Context::new();
	ctx.insert("body", &body);
	let body = Page::default().user_data_opt(UserData::load_opt(&id, &pool).await?).render_with_ctx("note.html", &tmpl, ctx)?;
	Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
}
