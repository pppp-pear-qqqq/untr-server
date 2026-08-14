use std::ops::Deref;

use super::*;

pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.route("", web::get().to(index));
	cfg.service(web::resource("setting").get(view_setting).patch(update_setting));
}

async fn index(id: Identity, pool: web::Data<SqlitePool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	let pool = pool.as_ref();
	let record = sqlx::query!("SELECT profile,webhook FROM user WHERE id=?", *id).fetch_one(pool).await?;
	let mut ctx = tera::Context::new();
	ctx.insert("profile", &record.profile);
	ctx.insert("webhook", &record.webhook);

	let body = Page::default().user_data(UserData::load(&id, pool).await?).render("home.html", &tmpl)?;
	Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
}

async fn view_setting(id: Identity, pool: web::Data<SqlitePool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	let body = Page::default().user_data(UserData::load(&id, &pool).await?).render("setting.html", &tmpl)?;
	Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
}

#[derive(serde::Deserialize, Validate)]
#[validate(schema(function = "any_some"))]
struct Setting {
	#[validate(length(max = 4096, message = "プロフィールは4096文字以内で入力してください"))]
	profile: Option<String>,
	#[validate(length(max = 256, message = "ウェブフックURLはそんなに長くないと思います"))]
	webhook: Option<String>,
}
async fn update_setting(web::Json(info): web::Json<Setting>, id: Identity, pool: web::Data<SqlitePool>) -> common::Result<impl Responder> {
	info.validate()?;
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

	let pool = pool.as_ref();
	builder.build().execute(pool).await?;

	Ok(HttpResponse::NoContent().finish())
}

fn any_some(v: &Setting) -> Result<(), validator::ValidationError> {
	if v.profile.is_some() || v.webhook.is_some() {
		Ok(())
	} else {
		let mut err = validator::ValidationError::new("empty_setting");
		err.message = Some(std::borrow::Cow::Borrowed("少なくとも1つの項目を変更してください"));
		Err(err)
	}
}
