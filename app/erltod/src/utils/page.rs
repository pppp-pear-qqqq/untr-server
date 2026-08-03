use std::ops::Deref;

use sqlx::SqlitePool;

#[derive(serde::Serialize)]
pub struct Page {
	title: String,
	#[serde(flatten)]
	page_type: PageType,
}
#[derive(serde::Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum PageType {
	Standard {
		#[serde(rename = "user", skip_serializing_if = "Option::is_none")]
		user_data: Option<ActorData>,
	},
	Min,
}
#[derive(serde::Serialize)]
pub struct ActorData {
	name: String,
}

impl common::PageRender for Page {}
impl Default for Page {
	fn default() -> Self {
		Self {
			title: "untroche.portal".into(),
			page_type: PageType::Standard { user_data: None },
		}
	}
}
impl Page {
	pub fn title(self, title: &str) -> Self {
		Self { title: title.into(), ..self }
	}
	pub fn actor_data(self, user_data: ActorData) -> Self {
		Self {
			page_type: PageType::Standard { user_data: Some(user_data) },
			..self
		}
	}
	pub fn actor_data_opt(self, user_data: Option<ActorData>) -> Self {
		Self {
			page_type: PageType::Standard { user_data },
			..self
		}
	}
	pub fn min(self) -> Self {
		Self { page_type: PageType::Min, ..self }
	}
}

impl ActorData {
	pub async fn load(id: &super::Identity, pool: &SqlitePool) -> Result<Self, sqlx::Error> {
		let id = id.deref();
		let name = sqlx::query_scalar!("SELECT name FROM actor WHERE id=?", id).fetch_one(pool).await?;
		Ok(Self { name })
	}
	pub async fn load_opt(id: &Option<super::Identity>, pool: &SqlitePool) -> Result<Option<Self>, sqlx::Error> {
		match id {
			Some(id) => Self::load(id, pool).await.map(Some),
			None => Ok(None),
		}
	}
}
