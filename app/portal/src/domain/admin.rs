use std::str::FromStr;

use crate::util::State;

use super::*;

/// リソース
pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.route("", web::get().to(index));
	cfg.route("report", web::get().to(get_reports));
}

#[derive(serde::Deserialize)]
struct Config {
	state: Option<String>,
}
async fn index(web::Query(info): web::Query<Config>, req_type: common::ReqType, req: actix_web::HttpRequest, pool: web::Data<Pool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
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

	if req_type == common::ReqType::Empty {
		Ok(HttpResponse::NoContent().finish())
	} else {
		let mut ctx = tera::Context::new();
		ctx.insert("state", &state);
		let body = Page::default().title("admin").render_with_ctx("admin.html", &tmpl, ctx)?;
		Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
	}
}

#[derive(serde::Deserialize)]
struct GetReports {
	#[serde(default)]
	ignore_checked: bool,
}
async fn get_reports(web::Query(info): web::Query<GetReports>, pool: web::Data<Pool>) -> common::Result<impl Responder> {
	#[derive(serde::Serialize)]
	struct Report {
		id: i64,
		timestamp: i64,
		user: String,
		tag: String,
		body: String,
		checked: bool,
	}

	let pool = pool.as_ref();
	let reports: Vec<_> = if info.ignore_checked {
		sqlx::query!("SELECT r.id,r.timestamp,u.name AS user,r.tag,r.body FROM report r JOIN user u ON r.user=u.id WHERE checked=FALSE")
			.fetch_all(pool)
			.await?
			.into_iter()
			.map(|r| Report {
				id: r.id,
				timestamp: r.timestamp,
				user: r.user,
				tag: r.tag,
				body: r.body,
				checked: false,
			})
			.collect()
	} else {
		sqlx::query!("SELECT r.id,r.timestamp,u.name AS user,r.tag,r.body,r.checked FROM report r JOIN user u ON r.user=u.id")
			.fetch_all(pool)
			.await?
			.into_iter()
			.map(|r| Report {
				id: r.id,
				timestamp: r.timestamp,
				user: r.user,
				tag: r.tag,
				body: r.body,
				checked: r.checked != 0,
			})
			.collect()
	};
	Ok(HttpResponse::Ok().json(reports))
}
