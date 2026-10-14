use std::str::FromStr;

use super::*;

/// リソース
pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.route("", web::to(index));
	cfg.route("change_name", web::to(change_name));
}

#[derive(serde::Deserialize)]
struct Config {
	state: Option<String>,
}
async fn index(web::Query(info): web::Query<Config>, req: actix_web::HttpRequest) -> common::Result<impl Responder> {
	if let Some(state) = info.state {
		// 取得
		let new = crate::util::State::from_str(&state).map_err(|_| ErrorBadRequest("Invalid State."))?;
		// app_data設定
		match req.app_data::<StateHandle>() {
			Some(data) => data.set(new),
			None => return Err(ErrorInternalServerError("State is not configured").into()),
		};
	}

	Ok(HttpResponse::Ok().body(""))
}

async fn change_name() -> common::Result<impl Responder> {
	Ok("todo")
}
