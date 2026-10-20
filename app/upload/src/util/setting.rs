use actix_web::{error::ErrorBadRequest, mime};
use common::FetchDest;
use dashmap::DashMap;
use fxhash::FxHashMap as HashMap;
use serde_with::{DisplayFromStr, serde_as};
use uuid::Uuid;

pub use super::setting_dto::load_or_default;
pub use super::setting_dto::save;

pub type RouteMap = DashMap<String, PathMap>;
pub type PathMap = HashMap<String, Setting>;
pub type Setting = HashMap<Option<FetchDest>, Resource>;

#[serde_as]
#[derive(Debug, serde::Serialize)]
pub enum Resource {
	File {
		key: Uuid,
		#[serde_as(as = "DisplayFromStr")]
		mime: mime::Mime,
	},
	RandomFile {
		#[serde_as(as = "Vec<(_, DisplayFromStr, _)>")]
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
	/// FormConfigと保存予定ファイルの情報からResourceを構築する（バリデーション兼任）
	pub fn build(config: ResourceConfig, mut file_infos: Vec<(Uuid, mime::Mime)>) -> actix_web::Result<Self> {
		match config {
			ResourceConfig::File => {
				if file_infos.len() != 1 {
					return Err(ErrorBadRequest("1 file required").into());
				}
				let (key, mime) = file_infos.pop().unwrap();
				Ok(Self::File { key, mime })
			}
			ResourceConfig::RandomFile { weights, cache } => {
				if file_infos.len() != weights.len() {
					return Err(ErrorBadRequest("Weights length mismatch").into());
				}
				let items = file_infos.into_iter().zip(weights).map(|((key, mime), weight)| (key, mime, weight)).collect();
				Ok(Self::RandomFile { items, cache })
			}
			ResourceConfig::FontRender { width, height, line_length } => {
				if file_infos.len() != 1 {
					return Err(ErrorBadRequest("1 file required").into());
				}
				let (key, _) = file_infos.pop().unwrap();
				Ok(Self::FontRender { key, width, height, line_length })
			}
		}
	}

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

#[derive(serde::Deserialize)]
#[serde(tag = "type")]
pub enum ResourceConfig {
	File,
	RandomFile {
		weights: Vec<u32>, // ファイルの数と一致させる
		cache: bool,
	},
	FontRender {
		width: u32,
		height: u32,
		line_length: u32,
	},
}

pub fn pathmap_to_serializable(map: &PathMap) -> HashMap<&String, HashMap<String, &Resource>> {
	let mut parsed = HashMap::default();
	for (path, path_map) in map {
		let mut stringified = HashMap::default();
		for (dest_opt, resource) in path_map.iter() {
			let key_str = dest_opt.as_ref().map(|d| d.to_string()).unwrap_or_else(|| "".to_string());
			stringified.insert(key_str, resource);
		}
		parsed.insert(path, stringified);
	}
	parsed
}
