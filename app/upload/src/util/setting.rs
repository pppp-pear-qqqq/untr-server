use std::str::FromStr;

use actix_web::{FromRequest, error::ErrorBadRequest, mime};
use dashmap::DashMap;
use fxhash::FxHashMap as HashMap;
use uuid::Uuid;

pub type RouteMap = DashMap<String, PathMap>;
pub type PathMap = HashMap<String, Setting>;
pub type Setting = HashMap<Option<FetchDest>, Resource>;

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Hash, serde::Deserialize, strum::EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum FetchDest {
	audio,
	audioworklet,
	document,
	embed,
	empty,
	fencedframe,
	font,
	frame,
	iframe,
	image,
	manifest,
	object,
	paintworklet,
	report,
	script,
	serviceworker,
	sharedworker,
	style,
	track,
	video,
	webidentity,
	worker,
	xslt,
}
impl FromRequest for FetchDest {
	type Error = actix_web::Error;
	type Future = std::future::Ready<Result<Self, Self::Error>>;

	fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
		let header_value = match req.headers().get("sec-fetch-dest") {
			Some(val) => val,
			None => return std::future::ready(Err(ErrorBadRequest("Missing Sec-Fetch-Dest header"))),
		};
		let dest_str = match header_value.to_str() {
			Ok(s) => s,
			Err(_) => return std::future::ready(Err(ErrorBadRequest("Invalid Sec-Fetch-Dest encoding"))),
		};
		std::future::ready(match Self::from_str(dest_str) {
			Ok(dest) => Ok(dest),
			Err(_) => Err(ErrorBadRequest(format!("Unknown Sec-Fetch-Dest: {}", dest_str))),
		})
	}
}
// トレイト作って保持側はBox<dyn Resource>としてそれぞれ実装する形でもいい どうしよう
// RandomFileをVec<(u32, File)>と実装できるのは多少綺麗かもしれないが、そのくらいなら個別実装でも良いような気もする
// レスポンス作成関数の規模を確認してから？　もしくは設定アップロードの実装次第かも
#[derive(Debug)]
pub enum Resource {
	File {
		key: Uuid,
		mime: mime::Mime,
	},
	RandomFile {
		items: Vec<(Uuid, mime::Mime, u32)>, // key,mime,weight
		cache: bool,
	},
	FontRender {
		key: Uuid,
		width: u32,
		height: u32,
		line_length: u32,
	},
}

impl Resource {
	pub fn key_and_mimes(&self) -> Vec<(&Uuid, &mime::Mime)> {
		match self {
			Resource::File { key, mime, .. } => vec![(key, mime)],
			Resource::RandomFile { items, .. } => items.iter().map(|(key, mime, ..)| (key, mime)).collect(),
			Resource::FontRender { key, .. } => vec![(key, &mime::IMAGE_PNG)],
		}
	}
	pub fn key_count(&self) -> usize {
		match self {
			Resource::File { .. } => 1,
			Resource::RandomFile { items, .. } => items.len(),
			Resource::FontRender { .. } => 1,
		}
	}
	pub fn change_files(&mut self, files: &[(Uuid, mime::Mime)]) {
		match self {
			Resource::File { key, mime, .. } => {
				*key = files[0].0.clone();
				*mime = files[0].1.clone();
			}
			Resource::RandomFile { items, .. } => {
				let mut weights = items.iter().map(|(_, _, weight)| *weight);
				*items = files.iter().map(|(key, mime)| (key.clone(), mime.clone(), weights.next().unwrap_or(1))).collect();
			}
			Resource::FontRender { key, .. } => {
				*key = files[0].0.clone();
			}
		}
	}
}
