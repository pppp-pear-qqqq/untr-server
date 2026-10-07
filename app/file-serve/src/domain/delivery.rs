use super::*;
use crate::util::RouteMap;

use base64::prelude::*;
use fxhash::FxHashMap as HashMap;

#[derive(serde::Deserialize)]
pub struct Addr {
	username: String,
	path: String,
	query: HashMap<String, String>,
}
impl Addr {
	fn try_from_slice(v: &[u8]) -> common::Result<Self> {
		let str = String::from_utf8_lossy(v).to_string();
		let (username, path) = str.split_once('/').ok_or_else(|| ErrorBadRequest("不正なパス形式"))?;
		let (path, query) = path.split_once('?').unwrap_or((path, ""));
		Ok(Self {
			username: username.to_string(),
			path: path.to_string(),
			query: query
				.split('&')
				.map(|s| {
					let (k, v) = s.split_once('=').unwrap_or((s, ""));
					(k.to_string(), v.to_string())
				})
				.collect(),
		})
	}
	fn with(mut self, mut query: HashMap<String, String>) -> Self {
		query.extend(self.query);
		self.query = query;
		self
	}
}

pub async fn plane(path: web::Path<Addr>, setting: web::Data<RouteMap>) -> common::Result<impl Responder> {
	file(path.into_inner(), setting).await
}
pub async fn b64(path: web::Path<String>, web::Query(query): web::Query<HashMap<String, String>>, setting: web::Data<RouteMap>) -> common::Result<impl Responder> {
	let decoded = BASE64_URL_SAFE.decode(&path.into_inner())?;
	file(Addr::try_from_slice(&decoded)?.with(query), setting).await
}
pub async fn encrypt(path: web::Path<String>, web::Query(query): web::Query<HashMap<String, String>>, setting: web::Data<RouteMap>) -> common::Result<impl Responder> {
	let decoded = BASE64_URL_SAFE.decode(&path.into_inner())?; // TODO
	file(Addr::try_from_slice(&decoded)?.with(query), setting).await
}

async fn file(addr: Addr, setting: web::Data<RouteMap>) -> common::Result<impl Responder> {
	Ok("TODO")
}
