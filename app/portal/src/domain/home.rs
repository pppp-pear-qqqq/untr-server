use std::ops::Deref;

use super::*;

pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.route("", web::get().to(index));
	cfg.service(web::resource("settings").get(view_settings).patch(update_settings));
}

async fn index(id: Identity, pool: web::Data<SqlitePool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	let body = Page::default().user_data(UserData::load(&id, &pool).await?).render("home.html", &tmpl)?;
	Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
}

async fn view_settings(id: Identity, pool: web::Data<SqlitePool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	let body = Page::default().user_data(UserData::load(&id, &pool).await?).render("settings.html", &tmpl)?;
	Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
}

#[derive(serde::Deserialize)]
struct Settings {
	profile: Option<String>,
	webhook: Option<String>,
}
async fn update_settings(web::Json(info): web::Json<Settings>, id: Identity, pool: web::Data<SqlitePool>) -> common::Result<impl Responder> {
	if info.profile.is_none() && info.webhook.is_none() {
		return Ok(HttpResponse::NoContent().finish());
	}

	let pool = pool.as_ref();
	let mut builder = sqlx::query_builder::QueryBuilder::new("UPDATE user SET ");
	let mut sep = builder.separated(',');
	if let Some(profile) = &info.profile {
		sep.push("profile=");
		sep.push_bind_unseparated(profile);
	}
	if let Some(webhook) = &info.webhook {
		sep.push("webhook=");
		sep.push_bind_unseparated(webhook);
	}
	builder.push(" WHERE id=");
	builder.push_bind(id.deref());
	builder.build().execute(pool).await?;

	Ok(HttpResponse::NoContent().finish())
}
