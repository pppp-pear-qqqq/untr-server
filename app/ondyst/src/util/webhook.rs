use crate::util::client;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Webhook {
	content: String,
	username: Option<String>,
	avatar_url: Option<String>,
}

#[derive(serde::Serialize)]
struct Post {
	target: Vec<i64>,
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

	pub async fn send(self, target: Vec<i64>) -> Result<reqwest::Response, reqwest::Error> {
		client().post("http://portal:8000/webhook").json(&Post { target, content: self }).send().await.and_then(|r| r.error_for_status())
	}
}
