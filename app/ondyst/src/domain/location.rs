use std::sync::OnceLock;

use fxhash::FxHashSet as HashSet;
use regex::Regex;

use crate::util::{APP_URL, Webhook};

use super::*;

/// リソース
pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.route("", web::to(location_list));
	cfg.route("new", web::post().to(new_location));
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

const BASE62: [char; 62] = [
	'0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W',
	'X', 'Y', 'Z',
];

async fn new_location(id: Identity, state: StateHandle, pool: web::Data<Pool>) -> common::Result<impl Responder> {
	let timestamp = chrono::Utc::now().timestamp();
	state.get().only_active()?;

	let key = nanoid::nanoid!(8, &BASE62);

	let message = format!("新しい場所を追加しました<br><a href=\"location/{key}\">移動する</a>");

	let pool = pool.as_ref();
	sqlx::query!("INSERT INTO log(timestamp,actor,body) VALUES(?,?,?)", timestamp, *id, message).execute(pool).await?;

	Ok(HttpResponse::SeeOther().insert_header((header::LOCATION, key)).finish())
}

async fn location(key: web::Path<String>, page: Pagination<20, 100>, req_type: ReqType, id: Option<Identity>, _: StateHandle, pool: web::Data<Pool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	#[derive(serde::Serialize)]
	struct Location {
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

	if key.contains('_') {
		// actor
		let json = serde_json::to_string(&key.split('_').map(|s| s.parse::<i64>()).collect::<Result<Vec<_>, _>>()?)?;

		let size = sqlx::query_scalar!("SELECT COUNT(*) FROM chat WHERE actor IN (SELECT value FROM json_each(?))", json).fetch_one(pool).await?;
		let chat_list = sqlx::query_as!(Chat, "SELECT id,timestamp,actor,name,icon,body FROM chat WHERE actor IN (SELECT value FROM json_each(?)) ORDER BY id DESC LIMIT ?,?", json, offset, limit)
			.fetch_all(pool)
			.await?;
		Ok(HttpResponse::Ok().json(serde_json::json!({
			"size": size,
			"list": chat_list,
		})))
	} else {
		// location
		let size = sqlx::query_scalar!("SELECT COUNT(*) FROM chat WHERE location=?", key).fetch_one(pool).await?;
		let chat_list = sqlx::query_as!(Chat, "SELECT id,timestamp,actor,name,icon,body FROM chat WHERE location=? ORDER BY id DESC LIMIT ?,?", key, offset, limit).fetch_all(pool).await?;

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
}

#[derive(serde::Deserialize, Validate)]
struct Chat {
	#[validate(length(max = 16, message = "16文字以内で入力してください"))]
	location: String, // key
	#[validate(length(max = 16, message = "16文字以内で入力してください"))]
	name: String,
	#[validate(length(max = 256, message = "256文字以内にしてください"))]
	icon: String,
	#[validate(length(max = 500, message = "500文字以内で入力してください"))]
	body: String,
}
async fn post_chat(web::Form(info): web::Form<Chat>, id: Identity, state: StateHandle, pool: web::Data<Pool>) -> common::Result<impl Responder> {
	let timestamp = chrono::Utc::now().timestamp();
	state.get().only_active()?;
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

	let pool = pool.as_ref();

	// 発言の投稿
	let mut tx = pool.begin().await?;
	let chat_id = sqlx::query_scalar!("INSERT INTO chat(timestamp,location,actor,name,icon,body) VALUES(?,?,?,?,?,?) RETURNING id", timestamp, info.location, id, info.name, info.icon, body)
		.fetch_one(&mut *tx)
		.await?;
	// メンション
	let mentions = if !mentions.is_empty() {
		let json = serde_json::to_string(&mentions).unwrap();
		sqlx::query_scalar!("INSERT OR IGNORE INTO chat_mention(source,target) SELECT ?,actor.id FROM json_each(?) JOIN actor ON actor.id=value RETURNING target", chat_id, json)
			.fetch_all(&mut *tx)
			.await?
	} else {
		Vec::new()
	};
	// アンカー
	let anchors = if !anchors.is_empty() {
		let json = serde_json::to_string(&anchors).unwrap();
		sqlx::query_scalar!("INSERT OR IGNORE INTO chat_anchor(source,target) SELECT ?,chat.id FROM json_each(?) JOIN chat ON chat.id=value RETURNING target", chat_id, json)
			.fetch_all(&mut *tx)
			.await?
	} else {
		Vec::new()
	};
	tx.commit().await?;

	// 対象整理
	let mut targets = HashSet::default();
	targets.extend(mentions);
	// アンカーの対象追跡
	if !anchors.is_empty() {
		let mut builder = sqlx::QueryBuilder::new("SELECT actor FROM chat WHERE id IN (");
		let mut sep = builder.separated(',');
		for anchor in anchors {
			sep.push_bind(anchor);
		}
		builder.push(")");
		targets.extend(builder.build_query_scalar::<Option<i64>>().fetch_all(pool).await?.into_iter().flatten());
	}
	targets.remove(&id); // 自分対象を除外

	// 通知
	if !targets.is_empty() {
		let targets = targets.into_iter().collect();

		// サイト内通知
		let message = format!("id:{} から言及されました<br><a href=\"location/{}\">発言場所へ移動</a>", id, info.location);
		let json = serde_json::to_string(&targets).unwrap();
		sqlx::query!("INSERT INTO log(timestamp,actor,body) SELECT ?,value,? FROM json_each(?)", timestamp, message, json).execute(pool).await?;

		// webhook通知
		let preview = raw_body.char_indices().nth(48).map(|(idx, _)| &raw_body[..idx]).unwrap_or(&raw_body);
		let webhook = Webhook::new(format!("{}\n\n{}", preview, APP_URL)).username(format!("{} (one day's' talk)", info.name)).avatar_url(info.icon);
		tokio::spawn(async move {
			if let Err(err) = webhook.send(targets).await {
				error!("{:?}", err);
			}
		});
	}

	Ok(HttpResponse::NoContent().finish())
}
