use std::str::FromStr;

use actix_web::{HttpResponse, Responder, error::ErrorUnauthorized, http::header, web};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::utils::Identity;

pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.service(web::resource("").get(get).post(post));
}

/// 一時認証コードを発行
async fn get(id: Option<Identity>, pool: web::Data<SqlitePool>) -> common::Result<impl Responder> {
	const AUTH_EXPIRY: i64 = 300;
	let now = chrono::Utc::now().timestamp();

	let pool = pool.as_ref();
	sqlx::query!("DELETE FROM auth WHERE timestamp <= ?", now).execute(pool).await?; // 期限切れの認証情報を削除
	match id {
		Some(id) if sqlx::query_scalar!("SELECT EXISTS(SELECT * FROM user WHERE id=?)", *id).fetch_one(pool).await? != 0 => {
			let code = Uuid::new_v4();
			let code_slice = code.as_bytes().as_slice();
			let timestamp = now + AUTH_EXPIRY;
			sqlx::query!("INSERT INTO auth(code,timestamp,user) VALUES(?,?,?)", code_slice, timestamp, *id).execute(pool).await?;
			Ok(HttpResponse::Ok().content_type(header::ContentType::plaintext()).body(code.to_string()))
		}
		Some(_) => Err(ErrorUnauthorized("ログインセッションが無効です").into()),
		None => Ok(HttpResponse::SeeOther().insert_header((header::LOCATION, "/entry/login")).finish()),
	}
}

#[derive(serde::Deserialize)]
struct Post {
	code: String,
}
/// 送られてきた認証コードを検証して良ければユーザーIDを返す
async fn post(web::Json(info): web::Json<Post>, pool: web::Data<SqlitePool>) -> common::Result<impl Responder> {
	// HTTPリクエスト・レスポンスに乗せるUUIDは文字列
	// データベースに保存されているUUIDはバイナリ列
	let code = Uuid::from_str(&info.code)?;
	let code = code.as_bytes().as_slice();

	let pool = pool.as_ref();
	let r = sqlx::query!("DELETE FROM auth WHERE code=? RETURNING user,timestamp", code).fetch_optional(pool).await?;

	match r {
		Some(r) if r.timestamp > chrono::Utc::now().timestamp() => {
			let user = Uuid::from_slice(r.user.as_slice())?;
			Ok(HttpResponse::Ok().content_type(header::ContentType::plaintext()).body(user.to_string()))
		}
		Some(_) => Err(ErrorUnauthorized("認証コードが期限切れです").into()),
		None => Err(ErrorUnauthorized("認証コードが無効です").into()),
	}
}
