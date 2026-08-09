use actix_web::{HttpResponse, Responder, error::*, http::header, web};
use common::PageRender;
use sqlx::SqlitePool;
use tera::Tera;

use crate::utils::{ActorData, Identity, Page};

/// リソース
pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.route("", web::to(location_list));
	cfg.service(web::resource("{key}").get(location).post(post_chat));
}

async fn location_list(id: Option<Identity>, pool: web::Data<SqlitePool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	#[derive(serde::Serialize)]
	struct Record {
		key: String,
		name: String,
		lore: String,
	}

	let pool = pool.as_ref();
	let records = sqlx::query_as!(Record, "SELECT * FROM location").fetch_all(pool).await?;

	let mut ctx = tera::Context::new();
	ctx.insert("list", &records);
	let body = Page::default().actor_data_opt(ActorData::load_opt(&id, &pool).await?).render_with_ctx("location_list.html", &tmpl, ctx)?;
	Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
}

async fn location(key: web::Path<String>, id: Option<Identity>, pool: web::Data<SqlitePool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
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
	}

	let key = key.into_inner();

	let pool = pool.as_ref();
	let location = sqlx::query_as!(Location, "SELECT name,lore FROM location WHERE key=?", key).fetch_one(pool).await?;
	let item_list = sqlx::query_as!(Item, "SELECT id,name,lore FROM item WHERE location=?", key).fetch_all(pool).await?;

	let mut ctx = tera::Context::new();
	ctx.insert("location", &location);
	ctx.insert("item_list", &item_list);
	let body = Page::default().actor_data_opt(ActorData::load_opt(&id, &pool).await?).render_with_ctx("location.html", &tmpl, ctx)?;
	Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
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
	let body = chat.body; // TODO タグ処理

	let pool = pool.as_ref();
	sqlx::query!("INSERT INTO chat (timestamp,location,actor,name,icon,body) VALUES (?,?,?,?,?,?)", timestamp, chat.location, id, chat.name, chat.icon, body).execute(pool).await?;
	Ok(HttpResponse::NoContent().finish())
}
