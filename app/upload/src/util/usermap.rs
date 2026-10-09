use dashmap::DashMap;

pub struct UserMap(DashMap<Vec<u8>, String>);

impl UserMap {
	pub fn new() -> Self {
		Self(DashMap::new())
	}
	pub fn get(&self, key: &[u8]) -> Option<String> {
		self.0.get(key).map(|v| v.value().clone())
	}
	pub async fn get_or_load(&self, key: &[u8]) -> reqwest::Result<String> {
		match self.get(key) {
			Some(value) => Ok(value),
			None => {
				todo!()
			}
		}
	}
}
