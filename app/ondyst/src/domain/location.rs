use actix_web::{HttpResponse, Responder, error::*, http::header, web};
use common::{PageRender, ReqType, html_codec::*};
use sqlx::SqlitePool;
use tera::Tera;

use crate::utils::{ActorData, Identity, Page, tag_parse as tag};

/// リソース
pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.route("", web::to(location_list));
	cfg.service(web::resource("{key}").get(location).post(post_chat));
	// TODO stream
}

async fn location_list(id: Option<Identity>, pool: web::Data<SqlitePool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
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

async fn location(key: web::Path<String>, web::Query(page): web::Query<common::Pagination>, req_type: ReqType, id: Option<Identity>, pool: web::Data<SqlitePool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	#[derive(serde::Serialize)]
	struct Location {
		name: String,
		lore: String,
	}
	#[derive(serde::Serialize)]
	struct Chat {
		id: i64,
		timestamp: chrono::DateTime<chrono::Utc>,
		actor: Option<i64>,
		name: String,
		icon: String,
		body: String,
	}
	#[derive(serde::Serialize)]
	struct Item {
		id: i64,
		name: String,
		lore: String,
		message: String,
	}

	let key = key.into_inner();
	let offset = page.offset as i64;
	let limit = page.limit as i64;

	let pool = pool.as_ref();
	let location = match sqlx::query_as!(Location, "SELECT name,lore FROM location WHERE key=?", key).fetch_one(pool).await {
		Ok(r) => r,
		Err(sqlx::Error::RowNotFound) => return Err(ErrorNotFound("指定された場所は存在しません").into()),
		Err(err) => return Err(err.into()),
	};
	let chat_list = sqlx::query!("SELECT id,timestamp,actor,name,icon,body FROM chat WHERE location=? LIMIT ?,?", location.name, offset, limit).fetch_all(pool).await?.into_iter().map(|r| Chat { id: r.id, timestamp: chrono::DateTime::from_timestamp_secs(r.timestamp).unwrap(), actor: r.actor, name: r.name, icon: r.icon, body: r.body }).collect::<Vec<_>>();

	match req_type {
		ReqType::Empty => Ok(HttpResponse::Ok().json(chat_list)),
		_ => {
			let item_list = sqlx::query_as!(Item, "SELECT id,name,lore,message FROM item WHERE location=?", key).fetch_all(pool).await?;

			let mut ctx = tera::Context::new();
			ctx.insert("location", &location);
			ctx.insert("chat_list", &chat_list);
			ctx.insert("item_list", &item_list);
			let body = Page::default().actor_data_opt(ActorData::load_opt(&id, &pool).await?).render_with_ctx("location.html", &tmpl, ctx)?;
			Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
		}
	}
}

#[derive(serde::Deserialize)]
struct Chat {
	location: String,
	name: String,
	icon: String,
	body: String,
}
async fn post_chat(web::Form(chat): web::Form<Chat>, id: Identity, pool: web::Data<SqlitePool>) -> common::Result<impl Responder> {
	let timestamp = chrono::Utc::now().timestamp();
	let id = *id;
	let body = chat.body.escape(false).br();
	let body = body.tag(tag::Ondyst);

	let pool = pool.as_ref();
	sqlx::query!("INSERT INTO chat(timestamp,location,actor,name,icon,body) VALUES(?,?,?,?,?,?)", timestamp, chat.location, id, chat.name, chat.icon, body).execute(pool).await?;

	Ok(HttpResponse::NoContent().finish())
}
