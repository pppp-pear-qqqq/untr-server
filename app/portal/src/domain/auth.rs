use std::str::FromStr;

use super::*;

pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.service(web::resource("").route(web::get().to(get)).route(web::post().guard(guard::fn_guard(is_internal)).to(post)));
}

/// 一時認証コードを発行
async fn get(id: Option<Identity>, state: StateHandle, pool: web::Data<SqlitePool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	state.get().only_active()?;

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
			let mut ctx = tera::Context::new();
			ctx.insert("message", &code.to_string());
			let body = tmpl.render("popup_res_message.min.html", &ctx)?;
			Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
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
async fn post(web::Json(info): web::Json<Post>, state: StateHandle, pool: web::Data<SqlitePool>) -> common::Result<impl Responder> {
	state.get().only_active()?;

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
