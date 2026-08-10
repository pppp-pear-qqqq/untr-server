use std::ops::Deref;

use sqlx::SqlitePool;

#[derive(serde::Serialize)]
pub struct Page {
	title: String,
	#[serde(rename = "actor", skip_serializing_if = "Option::is_none")]
	actor_data: Option<ActorData>,
}
#[derive(serde::Serialize)]
pub struct ActorData {
	id: i64,
	name: String,
	icon: String,
}

impl common::PageRender for Page {}
impl Default for Page {
	fn default() -> Self {
		Self { title: "untroche.portal".into(), actor_data: None }
	}
}
impl Page {
	pub fn title(self, title: &str) -> Self {
		Self { title: title.into(), ..self }
	}
	pub fn actor_data(self, user_data: ActorData) -> Self {
		Self { actor_data: Some(user_data), ..self }
	}
	pub fn actor_data_opt(self, user_data: Option<ActorData>) -> Self {
		Self { actor_data: user_data, ..self }
	}
}

impl ActorData {
	pub async fn load(id: &super::Identity, pool: &SqlitePool) -> Result<Self, sqlx::Error> {
		let id = id.deref();
		let r = sqlx::query!("SELECT name,icon FROM actor WHERE id=?", id).fetch_one(pool).await?;
		Ok(Self { id: *id, name: r.name, icon: r.icon })
	}
	pub async fn load_opt(id: &Option<super::Identity>, pool: &SqlitePool) -> Result<Option<Self>, sqlx::Error> {
		match id {
			Some(id) => Self::load(id, pool).await.map(Some),
			None => Ok(None),
		}
	}
}
