use sqlx::SqlitePool;

// まだあんまり整理できていない　もしかしたらcommonに移せる部分もあるかも

#[derive(serde::Serialize)]
pub struct Page {
	pub title: String,
	#[serde(flatten)]
	pub page_type: PageType,
}
#[derive(serde::Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum PageType {
	Standard {
		#[serde(rename = "user", skip_serializing_if = "Option::is_none")]
		user_data: Option<UserData>,
	},
	Min,
}

impl Page {
	pub async fn standard_and_load(id: &Option<super::Identity>, pool: &SqlitePool) -> Result<Self, sqlx::Error> {
		Ok(Self {
			// page_type: PageType::Standard {
			// 	user_data: if let Some(id) = id { Some(UserData::load(id, &pool).await?) } else { None },
			// },
			..Default::default()
		})
	}
	pub fn ctx(&self) -> tera::Result<tera::Context> {
		Ok(tera::Context::from_serialize(self)?)
	}
}
impl Default for Page {
	fn default() -> Self {
		Self {
			title: "untroche.portal".into(),
			page_type: PageType::Standard { user_data: None },
		}
	}
}

#[derive(serde::Serialize)]
pub struct UserData {
	name: String,
	icon: String,
}
impl UserData {
	// pub async fn load(id: &super::Identity, pool: &SqlitePool) -> Result<Self, sqlx::Error> {
	// 	let id = id.deref();
	// 	let name = sqlx::query_scalar!("SELECT name FROM actor WHERE id=?", id).fetch_one(pool).await?;
	// 	Ok(Self { name })
	// }
}
