use std::str::FromStr;

use actix_session::Session;
use rand::{RngExt, seq::IteratorRandom};

use crate::utils::client;

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

async fn register(web::Form(info): web::Form<Auth>, session: Session, state: StateHandle, pool: web::Data<SqlitePool>) -> common::Result<impl Responder> {
	state.get().only_open()?;
	info.validate()?;
	let user = auth(info.code).await?;

	let user = user.to_bytes_le().to_vec();
	let name = generate_random_unicode(0x30a0..=0x30ff, 2..=8);

	let pool = pool.as_ref();
	let actor_id = match sqlx::query_scalar!("INSERT INTO actor(user,name) VALUES(?,?) RETURNING id", user, name).fetch_one(pool).await {
		Ok(r) => r,
		Err(sqlx::Error::Database(err)) if err.is_unique_violation() => return Err(ErrorBadRequest("そのユーザーは既に登録されています").into()),
		Err(err) => return Err(err.into()),
	};
	Identity::set(&session, actor_id)?;

	Ok(HttpResponse::Ok().content_type(header::ContentType::plaintext()).body(actor_id.to_string()))
}

async fn login(web::Form(info): web::Form<Auth>, session: Session, _: StateHandle, pool: web::Data<SqlitePool>) -> common::Result<impl Responder> {
	info.validate()?;
	let user = auth(info.code).await?;

	let user = user.to_bytes_le().to_vec();

	let pool = pool.as_ref();
	match sqlx::query_scalar!("SELECT id FROM actor WHERE user=?", user).fetch_one(pool).await {
		Ok(id) => {
			Identity::set(&session, id)?;
			Ok(HttpResponse::Ok().content_type(header::ContentType::plaintext()).body(id.to_string()))
		}
		Err(sqlx::Error::RowNotFound) => Err(ErrorUnauthorized("idが違う、またはキャラクターを未登録です").into()),
		Err(err) => Err(err.into()),
	}
}

async fn auth(code: String) -> common::Result<Uuid> {
	// dockerなしの内部通信なら"http://localhost:8000/auth"
	let res = client().post("http://portal:8000/auth").json(&Auth { code }).send().await.and_then(|r| r.error_for_status())?;
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
