use std::str::FromStr;

use actix_session::Session;
use actix_web::{HttpResponse, Responder, error::*, http::header, web};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::utils::Identity;

/// リソース
pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.service(web::resource("register").route(web::post().to(register)));
	cfg.service(web::resource("login").route(web::post().to(login)));
	cfg.service(web::resource("logout").route(web::post().to(logout)));
}

async fn auth(code: String) -> common::Result<Uuid> {
	#[derive(serde::Serialize)]
	struct Auth {
		code: String,
	}

	let client = reqwest::Client::new();
	// dockerなしの内部通信なら"http://localhost:8000/auth"
	let res = client.post("http://portal:8000/auth").json(&Auth { code }).send().await.and_then(|r| r.error_for_status())?;
	let user_id = res.text().await?;
	Ok(Uuid::from_str(&user_id)?)
}

#[derive(serde::Deserialize)]
struct Register {
	code: String,
	name: String,
}
async fn register(web::Form(info): web::Form<Register>, session: Session, pool: web::Data<SqlitePool>) -> common::Result<impl Responder> {
	let user = auth(info.code).await?;

	println!("user_id: {user}");

	let user = user.to_bytes_le().to_vec();

	let pool = pool.as_ref();
	let actor_id = sqlx::query_scalar!("INSERT INTO actor(user,name) VALUES(?,?) RETURNING id", user, info.name).fetch_one(pool).await?;
	Identity::set(&session, actor_id)?;

	Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(actor_id.to_string()))
}

#[derive(serde::Deserialize)]
struct Login {
	code: String,
	id: i64,
}
async fn login(web::Form(info): web::Form<Login>, session: Session, pool: web::Data<SqlitePool>) -> common::Result<impl Responder> {
	let user = auth(info.code).await?;

	println!("user_id: {user}");

	let user = user.to_bytes_le().to_vec();

	let pool = pool.as_ref();
	let ok = sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM actor WHERE id=? AND user=?)", info.id, user).fetch_one(pool).await? != 0;
	if ok {
		Identity::set(&session, info.id)?;
		Ok(HttpResponse::NoContent().finish())
	} else {
		Err(ErrorUnauthorized("idが違う、またはキャラクターを未登録です").into())
	}
}

async fn logout(session: Session) -> common::Result<impl Responder> {
	Identity::remove(&session);
	Ok(HttpResponse::NoContent().finish())
}
