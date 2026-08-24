use std::str::FromStr;

use crate::utils::State;

use super::*;

/// リソース
pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.route("", web::get().to(index));
}

#[derive(serde::Deserialize)]
struct Config {
	state: Option<String>,
}
async fn index(web::Query(info): web::Query<Config>, req: actix_web::HttpRequest, pool: web::Data<Pool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	let pool = pool.as_ref();

	let state = if let Some(state) = info.state {
		// 取得
		let new = State::from_str(&state).map_err(|_| ErrorBadRequest("ステートキーが不正"))?;
		let state = new.to_string();
		// app_data設定
		match req.app_data::<StateHandle>() {
			Some(data) => data.set(new),
			None => return Err(ErrorInternalServerError("State is not configured").into()),
		};
		// データベース更新
		sqlx::query!("UPDATE setting SET value=? WHERE key=?", state, crate::app_data::STATE).execute(pool).await?;
		state
	} else {
		sqlx::query_scalar!("SELECT value FROM setting WHERE key=?", crate::app_data::STATE).fetch_one(pool).await?
	};

	let mut ctx = tera::Context::new();
	ctx.insert("state", &state);
	let body = Page::default().title("admin").render_with_ctx("admin.html", &tmpl, ctx)?;
	Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
}
