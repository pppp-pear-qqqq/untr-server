use rand::seq::IteratorRandom;

use super::*;

pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.route("", web::get().to(list));
	cfg.route("{actor}", web::get().to(actor));
}

async fn list(web::Query(page): web::Query<Pagination>, req_type: ReqType, id: Option<Identity>, _: StateHandle, pool: web::Data<SqlitePool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	#[derive(serde::Serialize)]
	struct Record {
		id: i64,
		name: String,
		comment: String,
		icon: String,
	}

	let offset = page.offset as i64;
	let limit = page.limit as i64;

	let pool = pool.as_ref();
	let records = sqlx::query_as!(Record, "SELECT id,name,comment,icon FROM actor LIMIT ?,?", offset, limit).fetch_all(pool).await?;

	match req_type {
		ReqType::Empty => Ok(HttpResponse::Ok().json(records)),
		_ => {
			let mut ctx = tera::Context::new();
			ctx.insert("actor_list", &records);
			let body = Page::default().actor_data_opt(ActorData::load_opt(&id, &pool).await?).render_with_ctx("actor_list.html", &tmpl, ctx)?;
			Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
		}
	}
}

async fn actor(actor: web::Path<i32>, id: Option<Identity>, _: StateHandle, pool: web::Data<SqlitePool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	#[derive(serde::Serialize)]
	struct Record {
		name: String,
		profile: String,
		portrait_list: String,
	}
	#[derive(serde::Serialize)]
	struct Section<'a> {
		title: Option<&'a str>,
		content: &'a str,
	}

	let target_id = actor.into_inner();

	let pool = pool.as_ref();
	let record = match sqlx::query_as!(Record, "SELECT name,profile,portrait_list FROM actor WHERE id=?", target_id).fetch_one(pool).await {
		Ok(r) => r,
		Err(sqlx::Error::RowNotFound) => return Err(ErrorBadRequest("対象のキャラクターは存在しません").into()),
		Err(err) => return Err(err.into()),
	};

	let mut section_iter = record.profile.split("\n# ");
	let mut sections = Vec::new();
	if let Some(section) = section_iter.next() {
		if let Some(section) = section.strip_prefix("# ") {
			let (title, content) = section.split_once("\n").unwrap_or((section, ""));
			sections.push(Section { title: Some(title), content });
		} else {
			sections.push(Section { title: None, content: section });
		}
	}
	for section in section_iter {
		let (title, content) = section.split_once("\n").unwrap_or((section, ""));
		sections.push(Section { title: Some(title), content });
	}
	let portrait = record.portrait_list.lines().choose(&mut rand::rng());

	let mut ctx = tera::Context::new();
	ctx.insert("name", &record.name);
	ctx.insert("profile", &sections);
	ctx.insert("portrait", &portrait);
	let body = Page::default().title(&format!("{} - one day's' talk", record.name)).actor_data_opt(ActorData::load_opt(&id, &pool).await?).render_with_ctx("actor.html", &tmpl, ctx)?;
	Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
}
