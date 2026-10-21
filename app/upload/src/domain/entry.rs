use std::str::FromStr;

use actix_session::Session;

use crate::util::{UserMap, client};

use super::*;

#[derive(serde::Serialize, serde::Deserialize, Validate)]
pub struct Auth {
	#[validate(length(max = 40))]
	code: String,
}
pub async fn login(web::Form(info): web::Form<Auth>, session: Session, state: StateHandle, users: web::Data<UserMap>) -> common::Result<impl Responder> {
	state.get().only_active()?;
	info.validate()?;

	let (user_id, user_name) = auth(info.code).await?;
	if user_name.contains(&['/', '\\', '?', '#', '%', '*', '<', '>', ':', '"', '|', '\0']) {
		return Err(ErrorBadRequest("ユーザー名に使用不可文字が含まれているため当サービスを利用できません").into());
	} else if user_name == "." || user_name == ".." {
		return Err(ErrorBadRequest("ユーザー名が予約語であるため当サービスを利用できません").into());
	}
	let user_id = user_id.to_bytes_le().to_vec();

	users.insert(&user_id, user_name);

	Identity::set(&session, user_id)?;

	Ok(HttpResponse::NoContent().finish())
}

async fn auth(code: String) -> common::Result<(Uuid, String)> {
	let res = client()
		.post(if cfg!(debug_assertions) { "http://portal:8000/auth" } else { "http://127.0.0.1:8000/auth" })
		.json(&Auth { code })
		.send()
		.await
		.and_then(|r| r.error_for_status())?;
	let ret: (String, String) = res.json().await?;
	Ok((Uuid::from_str(&ret.0)?, ret.1))
}

pub async fn logout(session: Session) -> common::Result<impl Responder> {
	Identity::remove(&session);
	Ok(HttpResponse::NoContent().finish())
}
