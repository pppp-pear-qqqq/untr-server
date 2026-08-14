use super::*;

pub async fn index(id: Option<Identity>, pool: web::Data<SqlitePool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	let body = Page::default().user_data_opt(UserData::load_opt(&id, &pool).await?).render("index.html", &tmpl)?;
	Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
}

pub async fn info(id: Option<Identity>, pool: web::Data<SqlitePool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	let body = Page::default().user_data_opt(UserData::load_opt(&id, &pool).await?).render("info.html", &tmpl)?;
	Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
}
