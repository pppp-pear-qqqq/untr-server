use std::sync::OnceLock;

use fxhash::FxHashSet as HashSet;
use regex::Regex;

use crate::util::{APP_URL, Webhook};

use super::*;

#[derive(serde::Serialize, sqlx::FromRow)]
pub struct Chat {
	id: i64,
	timestamp: i64,
	actor: Option<i64>,
	name: String,
	icon: String,
	body: String,
	location: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct Search {
	location: Option<Vec<String>>,
	actor: Option<Vec<i64>>,
	body: Option<String>,
	level: SearchLevel,
	force: bool,
}
#[derive(serde::Deserialize, Default)]
pub enum SearchLevel {
	#[serde(rename = "0")]
	Standalone,
	#[default]
	#[serde(rename = "1")]
	WithReplies,
	#[serde(rename = "2")]
	Conversation,
}

impl Search {
	pub fn new(location: Option<Vec<String>>, actor: Option<Vec<i64>>, body: Option<String>, level: SearchLevel, force: bool) -> Self {
		Self { location, actor, body, level, force }
	}
}

pub async fn get_chat(info: Search, page: Pagination<20, 100>, pool: &Pool) -> Result<Vec<Chat>, sqlx::Error> {
	if info.location.is_none() && info.actor.is_none() {
		return Ok(Vec::new());
	}

	let mut builder = sqlx::QueryBuilder::new("SELECT c.id,c.timestamp,c.actor,c.name,c.icon,c.body,l.name location FROM chat c LEFT JOIN location l ON c.location=l.key WHERE 1=1");

	if let Some(location) = &info.location {
		if !location.is_empty() {
			builder.push(" AND c.location IN(");
			let mut sep = builder.separated(',');
			for l in location {
				sep.push_bind(l);
			}
			builder.push(")");
		}
	} else if !info.force {
		builder.push(" AND l.name IS NOT NULL");
	}
	if let Some(actor) = &info.actor {
		if !actor.is_empty() {
			builder.push(" AND (");
			match info.level {
				SearchLevel::Standalone => {
					// 対象が発した発言 ＆ 誰への返信でもない
					builder.push("c.actor IN(");
					let mut sep = builder.separated(',');
					for a in actor {
						sep.push_bind(a);
					}
					builder.push(") AND NOT EXISTS (SELECT 1 FROM chat_anchor ca WHERE ca.source=c.id) AND NOT EXISTS (SELECT 1 FROM chat_mention cm WHERE cm.source=c.id)");
				}
				SearchLevel::WithReplies => {
					// 対象が発した発言 (返信を含む)
					builder.push("c.actor IN(");
					let mut sep = builder.separated(',');
					for a in actor {
						sep.push_bind(a);
					}
					builder.push(")");
				}
				SearchLevel::Conversation => {
					// 対象の発言 OR 対象宛のメンション OR 対象の発言への返信
					builder.push("c.actor IN(");
					let mut sep = builder.separated(',');
					for a in actor {
						sep.push_bind(a);
					}
					builder.push(") OR EXISTS (SELECT 1 FROM chat_mention cm WHERE cm.source=c.id AND cm.target IN(");
					let mut sep = builder.separated(',');
					for a in actor {
						sep.push_bind(a);
					}
					builder.push(")) OR EXISTS (SELECT 1 FROM chat_anchor ca JOIN chat tc ON ca.target=tc.id WHERE ca.source=c.id AND tc.actor IN(");
					let mut sep = builder.separated(',');
					for a in actor {
						sep.push_bind(a);
					}
					builder.push("))");
				}
			}
			builder.push(")");
		}
	}
	if let Some(body) = &info.body {
		if !body.is_empty() {
			builder.push(" AND c.body LIKE ");
			builder.push_bind(format!("%{}%", body));
		}
	}
	builder.push(" ORDER BY c.id DESC");
	builder.push(" LIMIT ");
	builder.push_bind(page.limit as i64);
	builder.push(" OFFSET ");
	builder.push_bind(page.offset as i64);

	builder.build_query_as::<Chat>().fetch_all(pool).await
}

pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.service(web::resource("").get(get_chat_handler).post(post_chat));
}

async fn get_chat_handler(web::Query(info): web::Query<Search>, page: Pagination<20, 100>, _: StateHandle, pool: web::Data<Pool>) -> common::Result<impl Responder> {
	let result = get_chat(info, page, &pool).await?;
	Ok(HttpResponse::Ok().json(result))
}

#[derive(serde::Deserialize, Validate)]
struct Post {
	#[validate(length(max = 16, message = "16文字以内で入力してください"))]
	location: String, // key
	#[validate(length(max = 16, message = "16文字以内で入力してください"))]
	name: String,
	#[validate(length(max = 256, message = "256文字以内にしてください"))]
	icon: String,
	#[validate(length(max = 500, message = "500文字以内で入力してください"))]
	body: String,
}
async fn post_chat(web::Form(info): web::Form<Post>, id: Identity, state: StateHandle, pool: web::Data<Pool>, channel: web::Data<ChannelMap>) -> common::Result<impl Responder> {
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
	// チャンネル通知
	if let Some(tx) = channel.read().unwrap().get(&info.location) {
		let _ = tx.send(());
	}

	Ok(HttpResponse::NoContent().finish())
}
