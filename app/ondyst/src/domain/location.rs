use actix_web::web::Bytes;
use tokio::sync::broadcast;
use tokio_stream::{StreamExt, wrappers::BroadcastStream};

use crate::util::BASE62;

use super::*;

/// リソース
pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.route("", web::to(location_list));
	cfg.route("new", web::post().to(new_location));
	cfg.service(web::scope("{key}").service(web::resource("").get(location)).route("stream", web::get().to(stream)));
}

async fn location_list(id: Option<Identity>, _: StateHandle, pool: web::Data<Pool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	#[derive(serde::Serialize)]
	struct Record {
		key: String,
		name: String,
		lore: String,
	}

	let pool = pool.as_ref();
	let records = sqlx::query_as!(Record, "SELECT key,name,lore FROM location").fetch_all(pool).await?;

	let mut ctx = tera::Context::new();
	ctx.insert("location_list", &records);
	let body = Page::default().actor_data_opt(ActorData::load_opt(&id, &pool).await?).render_with_ctx("location_list.html", &tmpl, ctx)?;
	Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
}

async fn location(key: web::Path<String>, page: Pagination<20, 100>, req_type: ReqType, id: Option<Identity>, _: StateHandle, pool: web::Data<Pool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	#[derive(serde::Serialize)]
	struct Location {
		name: String,
		lore: String,
	}
	#[derive(serde::Serialize)]
	struct Item {
		id: i64,
		name: String,
		lore: String,
		message: String,
	}

	let key = key.into_inner();

	let pool = pool.as_ref();

	let size = sqlx::query_scalar!("SELECT COUNT(*) FROM chat WHERE location=?", key).fetch_one(pool).await?;
	let chat_list = chat::get_chat(chat::Search::new(Some(vec![key.clone()]), None, None, chat::SearchLevel::default(), false), page, pool).await?;

	match req_type {
		ReqType::Empty => Ok(HttpResponse::Ok().json(serde_json::json!({
			"size": size,
			"list": chat_list,
		}))),
		_ => {
			let location = sqlx::query_as!(Location, "SELECT name,lore FROM location WHERE key=?", key).fetch_optional(pool).await?;
			let item_list = sqlx::query_as!(Item, "SELECT id,name,lore,message FROM item WHERE location=?", key).fetch_all(pool).await?;
			let mut ctx = tera::Context::new();
			if let Some(id) = &id {
				let icon_list = sqlx::query_scalar!("SELECT icon_list FROM actor WHERE id=?", **id).fetch_one(pool).await?;
				let icon_list: Vec<_> = icon_list.lines().collect();
				ctx.insert("icon_list", &icon_list);
			}
			ctx.insert("location", &location);
			ctx.insert("location_key", &key);
			ctx.insert("chat_size", &size);
			ctx.insert("chat_list", &chat_list);
			ctx.insert("item_list", &item_list);
			let body = Page::default().actor_data_opt(ActorData::load_opt(&id, &pool).await?).render_with_ctx("location.html", &tmpl, ctx)?;
			Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
		}
	}
}

async fn new_location(id: Identity, state: StateHandle, pool: web::Data<Pool>) -> common::Result<impl Responder> {
	let timestamp = chrono::Utc::now().timestamp();
	state.get().only_active()?;

	let key = nanoid::nanoid!(8, &BASE62);

	let message = format!("新しい場所を追加しました<br><a href=\"location/{key}\">移動する</a>");

	let pool = pool.as_ref();
	sqlx::query!("INSERT INTO log(timestamp,actor,body) VALUES(?,?,?)", timestamp, *id, message).execute(pool).await?;

	Ok(HttpResponse::SeeOther().insert_header((header::LOCATION, key)).finish())
}

async fn stream(key: web::Path<String>, state: StateHandle, channel: web::Data<ChannelMap>) -> common::Result<impl Responder> {
	state.get().only_active()?;
	let key = key.into_inner();

	let rx = {
		let mut map = channel.write().unwrap();
		let tx = map.entry(key).or_insert_with(|| {
			// 最大16件の未読通知を保持（適宜調整）
			let (tx, _rx) = broadcast::channel(16);
			tx
		});
		tx.subscribe() // 購読開始
	};
	let stream = BroadcastStream::new(rx).map(|_| Ok::<_, actix_web::Error>(Bytes::from("update\n\n")));
	Ok(HttpResponse::Ok()
		.insert_header((header::CONTENT_TYPE, "text/event-stream"))
		.insert_header(header::CacheControl(vec![header::CacheDirective::NoCache]))
		.insert_header((header::CONNECTION, "keep-alive"))
		.streaming(stream))
}
