use std::str::FromStr;

use actix_session::Session;
use rand::{RngExt, seq::IteratorRandom};

use crate::util::client;

use super::*;

/// リソース
pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.service(web::resource("register").route(web::post().to(register)));
	cfg.service(web::resource("login").route(web::post().to(login)));
	cfg.route("logout", web::to(logout));
}

#[derive(serde::Serialize, serde::Deserialize, Validate)]
struct Auth {
	#[validate(length(max = 40))]
	code: String,
}

async fn register(web::Form(info): web::Form<Auth>, session: Session, state: StateHandle, pool: web::Data<Pool>) -> common::Result<impl Responder> {
	state.get().only_open()?;
	info.validate()?;
	let user = auth(info.code).await?;

	let user = user.to_bytes_le().to_vec();
	let name = generate_random_unicode(0x30a0..=0x30ff, 2..=8);

	let pool = pool.as_ref();
	let actor_id = match sqlx::query_scalar!("INSERT INTO actor(user,name) VALUES(?,?) RETURNING id", user, name).fetch_one(pool).await {
		Ok(r) => r,
		Err(sqlx::Error::Database(err)) if err.is_unique_violation() => return Err(ErrorBadRequest("あなたは既にキャラクターを登録しています").into()),
		Err(err) => return Err(err.into()),
	};
	Identity::set(&session, actor_id)?;

	Ok(HttpResponse::Ok().content_type(header::ContentType::plaintext()).body(actor_id.to_string()))
}

async fn login(web::Form(info): web::Form<Auth>, session: Session, _: StateHandle, pool: web::Data<Pool>) -> common::Result<impl Responder> {
	info.validate()?;
	let user = auth(info.code).await?;

	let user = user.to_bytes_le().to_vec();

	let pool = pool.as_ref();
	let id = sqlx::query_scalar!("SELECT id FROM actor WHERE user=?", user).fetch_optional(pool).await?.ok_or(ErrorUnauthorized("idが違う、またはキャラクターを未登録です"))?;
	Identity::set(&session, id)?;
	Ok(HttpResponse::Ok().content_type(header::ContentType::plaintext()).body(id.to_string()))
}

async fn auth(code: String) -> common::Result<Uuid> {
	let res = client().post(if cfg!(debug_assertions) { "http://portal:8000/auth" } else { "http://localhost:8000/auth" }).json(&Auth { code }).send().await.and_then(|r| r.error_for_status())?;
	let user_id = res.text().await?;
	Ok(Uuid::from_str(&user_id)?)
}

async fn logout(session: Session) -> common::Result<impl Responder> {
	Identity::remove(&session);
	Ok(HttpResponse::NoContent().finish())
}

fn generate_random_unicode(range: std::ops::RangeInclusive<u32>, length: std::ops::RangeInclusive<usize>) -> String {
	let mut rng = rand::rng();
	let length = length.choose(&mut rng).unwrap();
	let mut result = String::with_capacity(length);

	for _ in 0..length {
		if let Some(c) = char::from_u32(rng.random_range(range.clone())) {
			result.push(c);
		}
	}

	result
}
