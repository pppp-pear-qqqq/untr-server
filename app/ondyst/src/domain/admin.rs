use std::str::FromStr;

use crate::util::State;

use super::*;

/// リソース
pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.route("", web::get().to(index));
	cfg.route("add/location", web::post().to(add_location));
	cfg.route("add/item", web::post().to(add_item));
}

#[derive(serde::Deserialize)]
struct Config {
	state: Option<String>,
}
async fn index(web::Query(info): web::Query<Config>, req_type: common::ReqType, req: actix_web::HttpRequest, pool: web::Data<Pool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	#[derive(serde::Serialize)]
	struct Location {
		key: String,
		name: String,
		lore: String,
	}
	#[derive(serde::Serialize)]
	struct Item {
		id: i64,
		location: String,
		name: String,
		lore: String,
		message: String,
		direct: i64,
	}

	let pool = pool.as_ref();

	let state = if let Some(state) = info.state {
		// 取得
		let new = State::from_str(&state).map_err(|_| ErrorBadRequest("ステートキーが不正"))?;
		let state = new.to_string();
		// app_data設定
		match req.app_data::<StateHandle>() {
			Some(data) => data.set(new.clone()),
			None => return Err(ErrorInternalServerError("State is not configured").into()),
		};
		// データベース更新
		sqlx::query!("UPDATE setting SET value=? WHERE key=?", state, crate::app_data::STATE).execute(pool).await?;
		new
	} else {
		State::from_str(&sqlx::query_scalar!("SELECT value FROM setting WHERE key=?", crate::app_data::STATE).fetch_one(pool).await?).map_err(|_| ErrorInternalServerError("サーバー状態が正しくない"))?
	};

	if req_type == common::ReqType::Empty {
		Ok(HttpResponse::NoContent().finish())
	} else {
		let location = sqlx::query_as!(Location, "SELECT * FROM location").fetch_all(pool).await?;
		let item = sqlx::query_as!(Item, "SELECT * FROM item").fetch_all(pool).await?;

		let mut ctx = tera::Context::new();
		ctx.insert("locations", &location);
		ctx.insert("items", &item);

		let body = Page::default().title("admin").state(Some(state)).render_with_ctx("admin.html", &tmpl, ctx)?;
		Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
	}
}

#[derive(serde::Deserialize)]
struct Location {
	key: String,
	name: String,
	lore: String,
}
async fn add_location(web::Form(info): web::Form<Location>, pool: web::Data<Pool>) -> common::Result<impl Responder> {
	let pool = pool.as_ref();
	sqlx::query!("INSERT INTO location(key,name,lore) VALUES(?,?,?)", info.key, info.name, info.lore).execute(pool).await?;
	Ok(HttpResponse::NoContent().finish())
}

#[derive(serde::Deserialize)]
struct Item {
	location: String,
	name: String,
	lore: String,
	message: String,
	direct: bool,
}
async fn add_item(web::Form(info): web::Form<Item>, pool: web::Data<Pool>) -> common::Result<impl Responder> {
	let pool = pool.as_ref();
	sqlx::query!("INSERT INTO item(location,name,lore,message,direct) VALUES(?,?,?,?,?)", info.location, info.name, info.lore, info.message, info.direct).execute(pool).await?;
	Ok(HttpResponse::NoContent().finish())
}
