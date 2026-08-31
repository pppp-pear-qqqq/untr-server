use std::ops::Deref;

use sqlx::SqlitePool;

#[derive(serde::Serialize)]
pub struct Page {
	title: String,
	state: Option<crate::util::State>,
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
		Self {
			title: "one day's' talk".into(),
			state: None,
			actor_data: None,
		}
	}
}
impl Page {
	pub fn title(self, title: &str) -> Self {
		Self { title: title.into(), ..self }
	}
	pub fn state(self, state: Option<crate::util::State>) -> Self {
		Self { state, ..self }
	}
	pub fn actor_data(self, data: ActorData) -> Self {
		Self { actor_data: Some(data), ..self }
	}
	pub fn actor_data_opt(self, data: Option<ActorData>) -> Self {
		Self { actor_data: data, ..self }
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
