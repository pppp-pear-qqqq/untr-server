use super::*;

use base64::prelude::*;

#[derive(serde::Deserialize)]
pub struct Addr {
	username: String,
	path: String,
}
impl Addr {
	fn try_from_slice(v: &[u8]) -> common::Result<Self> {
		let str = String::from_utf8_lossy(v).to_string();
		let (username, path) = str.split_once('/').ok_or_else(|| ErrorBadRequest("不正なパス形式"))?;
		Ok(Self {
			username: username.to_string(),
			path: path.to_string(),
		})
	}
}

pub async fn plane(path: web::Path<Addr>) -> common::Result<impl Responder> {
	file(path.into_inner()).await
}
pub async fn b64(path: web::Path<String>) -> common::Result<impl Responder> {
	let decoded = BASE64_URL_SAFE.decode(&path.into_inner())?;
	file(Addr::try_from_slice(&decoded)?).await
}
pub async fn encrypt(path: web::Path<String>) -> common::Result<impl Responder> {
	let decoded = BASE64_URL_SAFE.decode(&path.into_inner())?; // TODO
	file(Addr::try_from_slice(&decoded)?).await
}

async fn file(addr: Addr) -> common::Result<impl Responder> {
	Ok("")
}
