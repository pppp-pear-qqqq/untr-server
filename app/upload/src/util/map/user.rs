use dashmap::DashMap;

#[derive(Debug)]
pub struct UserMap(DashMap<u128, String>);

impl UserMap {
	pub fn new() -> Self {
		Self(DashMap::new())
	}

	pub fn get(&self, key: &u128) -> Option<String> {
		self.0.get(key).map(|v| v.value().clone())
	}

	pub fn insert(&self, key: u128, value: String) {
		self.0.insert(key, value);
	}
}
