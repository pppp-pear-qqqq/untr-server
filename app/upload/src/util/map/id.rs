use std::sync::Arc;

use dashmap::DashMap;
use fxhash::FxHashMap as HashMap;
use rkyv::{self, rancor};

pub struct IdMap(Map);

type Map = DashMap<String, (Arc<String>, Arc<String>)>;
type Dto = HashMap<String, (String, String)>;

impl IdMap {
	const SAVE_FILE_PATH: &str = "pld.id.rkyv";

	pub async fn load_or_default() -> std::io::Result<Self> {
		let bytes = match tokio::fs::read(Self::SAVE_FILE_PATH).await {
			Ok(b) => b,
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Self(Map::default())),
			Err(e) => return Err(e),
		};
		let dto = rkyv::from_bytes::<Dto, rancor::Error>(&bytes).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
		Ok(Self(Map::from_iter(dto.into_iter().map(|(k, (u, p))| (k, (Arc::new(u), Arc::new(p)))))))
	}

	pub async fn save(&self) -> std::io::Result<()> {
		let dto = Dto::from_iter(self.0.clone().into_iter().map(|(k, (u, p))| (k, (u.to_string(), p.to_string()))));
		let bytes = rkyv::to_bytes::<rancor::Error>(&dto).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
		tokio::fs::write(Self::SAVE_FILE_PATH, bytes.into_vec()).await
	}

	pub fn get(&self, id: &str) -> Option<(Arc<String>, Arc<String>)> {
		self.0.get(id).map(|v| (v.0.clone(), v.1.clone()))
	}

	pub async fn insert_and_save(&mut self, id: impl ToString, user: String, path: String) -> std::io::Result<()> {
		self.0.insert(id.to_string(), (Arc::new(user), Arc::new(path)));
		self.save().await
	}
}
