use actix_web::{HttpResponse, Responder, http::header, web};
use common::{PageRender, ReqType};
use sqlx::SqlitePool;
use tera::Tera;

use crate::utils::{ActorData, Identity, Page};

pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.route("", web::get().to(list));
	cfg.route("{actor}", web::get().to(actor));
}

async fn list(web::Form(pagination): web::Form<common::Pagination>, req_type: ReqType, id: Option<Identity>, pool: web::Data<SqlitePool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	#[derive(serde::Serialize)]
	struct Record {
		name: String,
		comment: String,
		icon: String,
	}

	let offset = pagination.offset as i64;
	let limit = pagination.limit as i64;

	let pool = pool.as_ref();
	let records = sqlx::query_as!(Record, "SELECT name,comment,icon FROM actor LIMIT ?,?", offset, limit).fetch_all(pool).await?;

	match req_type {
		ReqType::Empty => Ok(HttpResponse::Ok().json(records)),
		_ => {
			let mut ctx = tera::Context::new();
			ctx.insert("list", &records);
			let body = Page::default().actor_data_opt(ActorData::load_opt(&id, &pool).await?).render_with_ctx("actor_list.html", &tmpl, ctx)?;
			Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
		}
	}
}

async fn actor(actor: web::Path<i32>, id: Option<Identity>, pool: web::Data<SqlitePool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	#[derive(serde::Serialize)]
	struct Record {
		name: String,
		profile: String,
		portrait_list: String,
	}

	let target_id = actor.into_inner();

	let pool = pool.as_ref();
	let record = sqlx::query_as!(Record, "SELECT name,profile,portrait_list FROM actor WHERE id=?", target_id).fetch_one(pool).await?;

	let mut ctx = tera::Context::new();
	ctx.insert("target", &record);
	let body = Page::default().title(&format!("{} - untroche.portal", record.name)).actor_data_opt(ActorData::load_opt(&id, &pool).await?).render_with_ctx("actor.html", &tmpl, ctx)?;
	Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
}
