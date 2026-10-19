use std::str::FromStr;

use actix_web::mime;
use common::FetchDest;
use dashmap::DashMap;
use fxhash::FxHashMap as HashMap;
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize, rancor};
use uuid::Uuid;

// 保存用のルート構造体（DashMap -> HashMap, SettingのHashMap -> Vec のタプル配列に変換）
#[derive(Archive, RkyvDeserialize, RkyvSerialize)]
pub struct RouteMapDto(pub HashMap<String, HashMap<String, Vec<(Option<String>, ResourceDto)>>>);

// 保存用リソース（Mime を String に変換）
#[derive(Archive, RkyvDeserialize, RkyvSerialize)]
pub enum ResourceDto {
	File { key: u128, mime: String },
	RandomFile { items: Vec<(u128, String, u32)>, cache: bool },
	FontRender { key: u128, width: u32, height: u32, line_length: u32 },
}

impl super::Resource {
	// Resource -> DTO
	fn into_dto(&self) -> ResourceDto {
		match self {
			Self::File { key, mime } => ResourceDto::File { key: key.as_u128(), mime: mime.to_string() },
			Self::RandomFile { items, cache } => ResourceDto::RandomFile {
				items: items.iter().map(|(k, m, w)| (k.as_u128(), m.to_string(), *w)).collect(),
				cache: *cache,
			},
			Self::FontRender { key, width, height, line_length } => ResourceDto::FontRender {
				key: key.as_u128(),
				width: *width,
				height: *height,
				line_length: *line_length,
			},
		}
	}

	// DTO -> Resource
	fn from_dto(dto: ResourceDto) -> Result<Self, mime::FromStrError> {
		Ok(match dto {
			ResourceDto::File { key, mime } => Self::File {
				key: Uuid::from_u128(key),
				mime: mime::Mime::from_str(&mime)?,
			},
			ResourceDto::RandomFile { items, cache } => {
				let mut parsed_items = Vec::new();
				for (k, m, w) in items {
					parsed_items.push((Uuid::from_u128(k), mime::Mime::from_str(&m)?, w));
				}
				Self::RandomFile { items: parsed_items, cache }
			}
			ResourceDto::FontRender { key, width, height, line_length } => Self::FontRender {
				key: Uuid::from_u128(key),
				width,
				height,
				line_length,
			},
		})
	}
}

const SAVE_FILE_PATH: &str = "route.rkyv";

pub async fn save(data: &super::RouteMap) -> std::io::Result<()> {
	let mut users_map = HashMap::default();

	// DashMapからDTO用のHashMapへ詰め替え
	for user_entry in data.iter() {
		let mut paths_map = HashMap::default();
		for (path_key, path_value) in user_entry.value().iter() {
			// Setting(HashMap) を Vec に平坦化
			let settings_vec: Vec<_> = path_value
				.iter()
				.map(|(k, v)| (k.map(|k| k.to_string()), v.into_dto())) // ※ ResourceにCloneを実装しておくか、参照から生成する必要があります
				.collect();
			paths_map.insert(path_key.clone(), settings_vec);
		}
		users_map.insert(user_entry.key().clone(), paths_map);
	}

	let dto = RouteMapDto(users_map);

	// rkyvでシリアライズ (1024は初期アロケーションサイズ)
	let bytes = rkyv::to_bytes::<rancor::Error>(&dto).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

	tokio::fs::write(SAVE_FILE_PATH, bytes.into_vec()).await?;
	Ok(())
}

pub async fn load_or_default() -> std::io::Result<super::RouteMap> {
	let bytes = match tokio::fs::read(SAVE_FILE_PATH).await {
		Ok(b) => b,
		Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(DashMap::new()),
		Err(e) => return Err(e),
	};

	let dto = rkyv::from_bytes::<RouteMapDto, rancor::Error>(&bytes).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

	// 3. DTOからDashMapへ復元
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
				if let Ok(resource) = super::Resource::from_dto(resource_dto) {
					setting.insert(fetch_dest, resource);
				}
			}
			actual_path_map.insert(path, setting);
		}
		route_map.insert(username, actual_path_map);
	}

	Ok(route_map)
}
