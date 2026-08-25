#[derive(serde::Deserialize)]
#[serde(default)]
pub struct Pagination {
	pub offset: usize,
	pub limit: usize,
}

impl Pagination {
	pub fn new(offset: usize, limit: usize) -> Self {
		Self { offset, limit }
	}
	pub fn page(&self) -> usize {
		self.offset.checked_div(self.limit).unwrap_or_default()
	}
}

impl Default for Pagination {
	fn default() -> Self {
		Self::new(0, 100)
	}
}
