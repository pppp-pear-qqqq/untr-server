use uuid::Uuid;

use crate::util::client;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Webhook {
	content: String,
	username: Option<String>,
	avatar_url: Option<String>,
}

#[derive(serde::Serialize)]
pub struct Post {
	target: Vec<Uuid>,
	content: Webhook,
}

impl Webhook {
	pub fn new(content: impl ToString) -> Self {
		Self {
			content: content.to_string(),
			username: None,
			avatar_url: None,
		}
	}
	pub fn username(mut self, username: impl ToString) -> Self {
		self.username = Some(username.to_string());
		self
	}
	pub fn avatar_url(mut self, avatar_url: impl ToString) -> Self {
		self.avatar_url = Some(avatar_url.to_string());
		self
	}

	pub async fn target(self, pool: &crate::app_data::Pool, target: Vec<i64>) -> Result<Post, common::Error> {
		let target = target.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(",");
		let target = sqlx::query!("SELECT user FROM actor WHERE id IN (?)", target)
			.fetch_all(pool)
			.await?
			.into_iter()
			.map(|r| Uuid::from_slice(&r.user))
			.collect::<Result<Vec<_>, _>>()?;
		Ok(Post { target, content: self })
	}
}

impl Post {
	pub async fn send(self) -> Result<reqwest::Response, reqwest::Error> {
		client()
			.post(if cfg!(debug_assertions) { "http://portal:8000/webhook" } else { "http://127.0.0.1:8000/webhook" })
			.json(&self)
			.send()
			.await
			.and_then(|r| r.error_for_status())
	}
}
