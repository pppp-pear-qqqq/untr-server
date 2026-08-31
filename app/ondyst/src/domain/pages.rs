use super::*;

pub async fn index(id: Option<Identity>, req: actix_web::HttpRequest, pool: web::Data<Pool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	let state = req.app_data::<StateHandle>().map(|x| x.get());
	let body = Page::default().state(state).actor_data_opt(ActorData::load_opt(&id, &pool).await?).render("index.html", &tmpl)?;
	Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
}

pub async fn info(id: Option<Identity>, req: actix_web::HttpRequest, pool: web::Data<Pool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	let state = req.app_data::<StateHandle>().map(|x| x.get());
	let body = Page::default().state(state).actor_data_opt(ActorData::load_opt(&id, &pool).await?).render("info.html", &tmpl)?;
	Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
}

pub async fn guide(id: Option<Identity>, req: actix_web::HttpRequest, pool: web::Data<Pool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	let state = req.app_data::<StateHandle>().map(|x| x.get());
	let body = Page::default().state(state).actor_data_opt(ActorData::load_opt(&id, &pool).await?).render("guide.html", &tmpl)?;
	Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
}
