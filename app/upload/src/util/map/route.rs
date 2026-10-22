use std::ops::Deref;
use std::ops::DerefMut;
use std::str::FromStr;

use actix_web::{error::ErrorBadRequest, mime};
use common::FetchDest;
use dashmap::DashMap;
use fxhash::FxHashMap as HashMap;
use rkyv::rancor;
use serde_with::{DisplayFromStr, serde_as};
use uuid::Uuid;

use self::dto::*;

pub struct RouteMap(DashMap<String, User>);
pub struct User(HashMap<String, Setting>);

pub type Setting = HashMap<Option<FetchDest>, Resource>;

#[serde_as]
#[derive(Debug, Clone, serde::Serialize)]
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

mod dto {
	use std::str::FromStr;

	use actix_web::mime;
	use fxhash::FxHashMap as HashMap;
	use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
	use uuid::Uuid;

	#[derive(Archive, RkyvDeserialize, RkyvSerialize)]
	pub struct RouteMapDto(pub HashMap<String, HashMap<String, Vec<(Option<String>, ResourceDto)>>>);

	// 保存用リソース（Mime を String に変換）
	#[derive(Archive, RkyvDeserialize, RkyvSerialize)]
	pub enum ResourceDto {
		File { key: u128, mime: String },
		RandomFile { items: Vec<(u128, String, u32)>, cache: bool },
		FontRender { key: u128, width: u32, height: u32, line_length: u32 },
	}

	impl From<super::Resource> for ResourceDto {
		fn from(value: super::Resource) -> Self {
			match value {
				super::Resource::File { key, mime } => ResourceDto::File { key: key.as_u128(), mime: mime.to_string() },
				super::Resource::RandomFile { items, cache } => ResourceDto::RandomFile {
					items: items.iter().map(|(k, m, w)| (k.as_u128(), m.to_string(), *w)).collect(),
					cache: cache,
				},
				super::Resource::FontRender { key, width, height, line_length } => ResourceDto::FontRender {
					key: key.as_u128(),
					width: width,
					height: height,
					line_length: line_length,
				},
			}
		}
	}
	impl TryInto<super::Resource> for ResourceDto {
		type Error = mime::FromStrError;

		fn try_into(self) -> Result<super::Resource, Self::Error> {
			Ok(match self {
				Self::File { key, mime } => super::Resource::File {
					key: Uuid::from_u128(key),
					mime: mime::Mime::from_str(&mime)?,
				},
				Self::RandomFile { items, cache } => {
					let mut parsed_items = Vec::new();
					for (k, m, w) in items {
						parsed_items.push((Uuid::from_u128(k), mime::Mime::from_str(&m)?, w));
					}
					super::Resource::RandomFile { items: parsed_items, cache }
				}
				Self::FontRender { key, width, height, line_length } => super::Resource::FontRender {
					key: Uuid::from_u128(key),
					width,
					height,
					line_length,
				},
			})
		}
	}
}

impl RouteMap {
	const SAVE_FILE_PATH: &str = if cfg!(debug_assertions) { "pld.route.rkyv" } else { "data/pld.route.rkyv" };

	pub async fn load_or_default() -> std::io::Result<Self> {
		let bytes = match tokio::fs::read(Self::SAVE_FILE_PATH).await {
			Ok(b) => b,
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Self(DashMap::new())),
			Err(e) => return Err(e),
		};

		let dto = rkyv::from_bytes::<RouteMapDto, rancor::Error>(&bytes).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

		let route_map = DashMap::new();
		for (username, path_map_dto) in dto.0 {
			let mut actual_path_map = HashMap::default();
			for (path, setting_vec_dto) in path_map_dto {
				let mut setting = HashMap::default();
				for (fetch_dest, resource_dto) in setting_vec_dto {
					// パース失敗したらスキップするかエラーにする (ここではスキップ)
					let fetch_dest = if let Some(d) = fetch_dest {
						match FetchDest::from_str(&d) {
							Ok(d) => Some(d),
							Err(_) => continue,
						}
					} else {
						None
					};
					if let Ok(resource) = resource_dto.try_into() {
						setting.insert(fetch_dest, resource);
					}
				}
				actual_path_map.insert(path, setting);
			}
			route_map.insert(username, User(actual_path_map));
		}

		Ok(Self(route_map))
	}

	pub async fn save(&self) -> std::io::Result<()> {
		let mut users_map = HashMap::default();

		// DashMapからDTO用のHashMapへ詰め替え
		for user_entry in self.iter() {
			let mut paths_map = HashMap::default();
			for (path_key, path_value) in user_entry.value().iter() {
				// Setting(HashMap) を Vec に平坦化
				let settings_vec: Vec<_> = path_value
					.iter()
					.map(|(k, v)| (k.map(|k| k.to_string()), ResourceDto::from(v.clone()))) // ※ ResourceにCloneを実装しておくか、参照から生成する必要があります
					.collect();
				paths_map.insert(path_key.clone(), settings_vec);
			}
			users_map.insert(user_entry.key().clone(), paths_map);
		}

		let dto = RouteMapDto(users_map);

		let bytes = rkyv::to_bytes::<rancor::Error>(&dto).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

		tokio::fs::write(Self::SAVE_FILE_PATH, bytes.into_vec()).await?;
		Ok(())
	}
}

impl Deref for RouteMap {
	type Target = DashMap<String, User>;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl DerefMut for RouteMap {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}
impl Default for User {
	fn default() -> Self {
		Self(Default::default())
	}
}
impl Deref for User {
	type Target = HashMap<String, Setting>;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl DerefMut for User {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}
impl serde::Serialize for User {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		let mut parsed = HashMap::default();
		for (path, map) in &self.0 {
			let mut stringified = HashMap::default();
			for (dest_opt, resource) in map.iter() {
				let key_str = dest_opt.as_ref().map(|d| d.to_string()).unwrap_or_else(|| "".to_string());
				stringified.insert(key_str, resource);
			}
			parsed.insert(path, stringified);
		}
		parsed.serialize(serializer)
	}
}
