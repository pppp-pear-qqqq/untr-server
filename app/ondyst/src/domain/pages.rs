use std::fs;

use super::*;

pub async fn index(id: Option<Identity>, req: actix_web::HttpRequest, pool: web::Data<Pool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	let tos = fs::read_to_string(resource("html/tos.html"))?;
	let mut ctx = tera::Context::new();
	ctx.insert("tos", &tos);
	let state = req.app_data::<StateHandle>().map(|x| x.get());
	let body = Page::default().state(state).actor_data_opt(ActorData::load_opt(&id, &pool).await?).render_with_ctx("index.html", &tmpl, ctx)?;
	Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
}

pub async fn info(id: Option<Identity>, req: actix_web::HttpRequest, pool: web::Data<Pool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	let body = fs::read_to_string(resource("html/info.html"))?;
	let mut ctx = tera::Context::new();
	ctx.insert("body", &body);
	let state = req.app_data::<StateHandle>().map(|x| x.get());
	let body = Page::default().state(state).actor_data_opt(ActorData::load_opt(&id, &pool).await?).render_with_ctx("note.html", &tmpl, ctx)?;
	Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
}

pub async fn guide(id: Option<Identity>, req: actix_web::HttpRequest, pool: web::Data<Pool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	let body = fs::read_to_string(resource("html/guide.html"))?;
	let mut ctx = tera::Context::new();
	ctx.insert("body", &body);
	let state = req.app_data::<StateHandle>().map(|x| x.get());
	let body = Page::default().state(state).actor_data_opt(ActorData::load_opt(&id, &pool).await?).render_with_ctx("note.html", &tmpl, ctx)?;
	Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
}
