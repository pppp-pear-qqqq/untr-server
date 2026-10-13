use dashmap::DashMap;

pub struct UserMap(DashMap<Vec<u8>, String>);

impl UserMap {
	pub fn load() -> Self {
		Self(DashMap::new())
	}
	// pub fn save() {}
	pub fn get(&self, key: &[u8]) -> Option<String> {
		self.0.get(key).map(|v| v.value().clone())
	}
	pub fn insert(&self, key: &Vec<u8>, value: String) {
		self.0.insert(key.clone(), value);
	}
}
