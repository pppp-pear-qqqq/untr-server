use std::sync::Arc;

use crate::util::client;

use super::*;

pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.route("", web::post().to(index));
}

#[derive(serde::Deserialize)]
pub struct Webhook {
	target: Vec<Uuid>,
	content: Content,
}
#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct Content {
	pub content: String,
	pub username: Option<String>,
	pub avatar_url: Option<String>,
}

impl Content {
	pub fn target(self, target: Vec<Uuid>) -> Webhook {
		Webhook { target, content: self }
	}
}
impl Webhook {
	pub async fn send(self, pool: &Pool) -> Result<(), sqlx::Error> {
		if self.target.is_empty() {
			return Ok(());
		}

		let mut builder = sqlx::QueryBuilder::new("SELECT webhook FROM user WHERE id IN (");
		let mut sep = builder.separated(',');
		for id in self.target {
			sep.push_bind(id.as_bytes().to_vec());
		}
		builder.push(") AND webhook IS NOT NULL");

		let urls: Vec<String> = builder.build_query_scalar().fetch_all(pool).await?;

		let content = Arc::new(self.content);

		for url in urls {
			let client = client().clone();
			let content = content.clone();

			tokio::spawn(async move {
				let res = client.post(&url).json(&*content).send().await.and_then(|r| r.error_for_status());
				if let Err(err) = res {
					error!("Webhook送信失敗: {}({})", err, url);
				}
			});
		}
		Ok(())
	}
}

async fn index(web::Json(info): web::Json<Webhook>, state: StateHandle, pool: web::Data<Pool>) -> common::Result<impl Responder> {
	state.get().only_active()?;
	info.send(pool.as_ref()).await?;
	Ok(HttpResponse::NoContent().finish())
}
