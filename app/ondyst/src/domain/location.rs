use std::sync::OnceLock;

use fxhash::FxHashSet as HashSet;
use regex::Regex;

use crate::utils::{APP_URL, Webhook};

use super::*;

/// リソース
pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.route("", web::to(location_list));
	cfg.service(web::resource("{key}").get(location).post(post_chat));
	// TODO stream
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

async fn location(key: web::Path<String>, web::Query(page): web::Query<Pagination>, req_type: ReqType, id: Option<Identity>, _: StateHandle, pool: web::Data<Pool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	#[derive(serde::Serialize)]
	struct Location {
		key: String,
		name: String,
		lore: String,
	}
	#[derive(serde::Serialize)]
	struct Chat {
		id: i64,
		timestamp: i64,
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
	let location = sqlx::query_as!(Location, "SELECT key,name,lore FROM location WHERE key=?", key).fetch_optional(pool).await?.ok_or(ErrorNotFound("指定された場所は存在しません"))?;
	let chat_list = sqlx::query_as!(Chat, "SELECT id,timestamp,actor,name,icon,body FROM chat WHERE location=? LIMIT ?,?", location.name, offset, limit).fetch_all(pool).await?;

	match req_type {
		ReqType::Empty => Ok(HttpResponse::Ok().json(chat_list)),
		_ => {
			let item_list = sqlx::query_as!(Item, "SELECT id,name,lore,message FROM item WHERE location=?", key).fetch_all(pool).await?;
			let mut ctx = tera::Context::new();
			if let Some(id) = &id {
				let icon_list = sqlx::query_scalar!("SELECT icon_list FROM actor WHERE id=?", **id).fetch_one(pool).await?;
				let icon_list: Vec<_> = icon_list.lines().collect();
				ctx.insert("icon_list", &icon_list);
			}
			ctx.insert("location", &location);
			ctx.insert("chat_list", &chat_list);
			ctx.insert("item_list", &item_list);
			let body = Page::default().actor_data_opt(ActorData::load_opt(&id, &pool).await?).render_with_ctx("location.html", &tmpl, ctx)?;
			Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
		}
	}
}

#[derive(serde::Deserialize, Validate)]
struct Chat {
	#[validate(length(max = 16, message = "16文字以内で入力してください"))]
	location: String,
	#[validate(length(max = 16, message = "16文字以内で入力してください"))]
	name: String,
	#[validate(length(max = 256, message = "256文字以内にしてください"))]
	icon: String,
	#[validate(length(max = 500, message = "500文字以内で入力してください"))]
	body: String,
}
async fn post_chat(web::Form(info): web::Form<Chat>, id: Identity, state: StateHandle, pool: web::Data<Pool>) -> common::Result<impl Responder> {
	state.get().only_active()?;

	let timestamp = chrono::Utc::now().timestamp();
	info.validate()?;

	// 入力のパース
	let id = *id;
	let raw_body = info.body.clone();
	let body = info.body.to_html(&tag::Ondyst, false);

	// メンション・アンカーの処理
	static RE: OnceLock<Regex> = OnceLock::new();
	let re = RE.get_or_init(|| Regex::new(r"@(?<mention>\d+)|&gt;&gt;(?<anchor>\d+)").unwrap());
	let mut mentions = HashSet::<i64>::default();
	let mut anchors = HashSet::<i64>::default();
	let body = re.replace_all(&body, |caps: &regex::Captures| {
		if let Some(m) = caps.name("mention") {
			if let Ok(v) = m.as_str().parse::<i64>() {
				mentions.insert(v);
				return format!("<a data-mention=\"{0}\">@{0}</a>", v);
			}
		} else if let Some(m) = caps.name("anchor") {
			if let Ok(v) = m.as_str().parse::<i64>() {
				anchors.insert(v);
				return format!("<a data-anchor=\"{0}\">&gt;&gt;{0}</a>", v);
			}
		}
		caps[0].to_string()
	});

	// 発言の投稿
	let pool = pool.as_ref();
	let mut tx = pool.begin().await?;
	let chat_id = sqlx::query_scalar!("INSERT INTO chat(timestamp,location,actor,name,icon,body) VALUES(?,?,?,?,?,?) RETURNING id", timestamp, info.location, id, info.name, info.icon, body)
		.fetch_one(&mut *tx)
		.await?;
	// メンション
	if !mentions.is_empty() {
		let mut builder = sqlx::QueryBuilder::new("INSERT INTO chat_mention(source,target) VALUES");
		let mut sep = builder.separated(',');
		for t in &mentions {
			sep.push('(').push_bind_unseparated(chat_id).push_bind(t).push_unseparated(')');
		}
		builder.build().execute(&mut *tx).await?;
	}
	// アンカー
	if !anchors.is_empty() {
		let mut builder = sqlx::QueryBuilder::new("INSERT INTO chat_anchor(source,target) VALUES");
		let mut sep = builder.separated(',');
		for t in &anchors {
			sep.push('(').push_bind_unseparated(chat_id).push_bind(t).push_unseparated(')');
		}
		builder.build().execute(&mut *tx).await?;
	}
	tx.commit().await?;

	// アンカーの対象追跡
	if !anchors.is_empty() {
		let mut builder = sqlx::QueryBuilder::new("SELECT actor FROM chat WHERE id IN (");
		let mut sep = builder.separated(',');
		for anchor in anchors {
			sep.push_bind(anchor);
		}
		builder.push(")");
		mentions.extend(builder.build_query_scalar::<Option<i64>>().fetch_all(pool).await?.into_iter().flatten());
	}
	// 通知
	mentions.remove(&id); // 自分対象を除外
	if !mentions.is_empty() {
		let preview = raw_body.char_indices().nth(48).map(|(idx, _)| &raw_body[..idx]).unwrap_or(&raw_body);
		let webhook = Webhook::new(format!("{}\n\n{}", preview, APP_URL)).username(format!("{} (one day's' talk)", info.name)).avatar_url(info.icon);
		let target = mentions.into_iter().collect();
		tokio::spawn(async move {
			if let Err(err) = webhook.send(target).await {
				eprintln!("{:?}", err);
			}
		});
	}

	Ok(HttpResponse::NoContent().finish())
}
