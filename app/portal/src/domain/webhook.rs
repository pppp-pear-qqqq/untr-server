use std::sync::Arc;

use crate::utils::client;

use super::*;

pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.route("", web::post().to(index));
}

#[derive(serde::Deserialize)]
struct Webhook {
	target: Vec<Uuid>,
	content: Content,
}
#[derive(serde::Deserialize, serde::Serialize)]
struct Content {
	content: String,
	username: Option<String>,
	avatar_url: Option<String>,
}
async fn index(web::Json(info): web::Json<Webhook>, state: StateHandle, pool: web::Data<Pool>) -> common::Result<impl Responder> {
	state.get().only_active()?;
	if info.target.is_empty() {
		return Ok(HttpResponse::NoContent().finish());
	}

	let mut builder = sqlx::QueryBuilder::new("SELECT webhook FROM user WHERE id IN (");
	let mut sep = builder.separated(',');
	for id in info.target {
		sep.push_bind(id.as_bytes().to_vec());
	}
	builder.push(") AND webhook IS NOT NULL");

	let pool = pool.as_ref();
	let urls: Vec<String> = builder.build_query_scalar().fetch_all(pool).await?;

	let content = Arc::new(info.content);

	for url in urls {
		let client = client().clone();
		let content = content.clone();

		tokio::spawn(async move {
			let res = client.post(&url).json(&*content).send().await.and_then(|r| r.error_for_status());
			if let Err(err) = res {
				eprintln!("Webhook送信失敗: {}({})", err, url);
			}
		});
	}
	Ok(HttpResponse::NoContent().finish())
}
