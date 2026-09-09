use std::sync::RwLock;

use actix_web::{cookie, web};
use base64::prelude::*;
use fxhash::FxHashMap as HashMap;
use log::{error, info};
pub use sqlx::SqlitePool as Pool;
pub use tera::Tera;
use tokio::sync::broadcast;

pub type ChannelMap = RwLock<HashMap<String, broadcast::Sender<()>>>;

use crate::util::{self, State, StateHandle, tag_parse as tag};

// 定数
pub const STATE: &str = "STATE";
const KEY: &str = "KEY";

#[derive(Clone)]
pub struct AppData {
	pub state: StateHandle,
	pub pool: web::Data<Pool>,
	pub tera: web::Data<Tera>,
	pub channels: web::Data<ChannelMap>,
	pub session_key: cookie::Key,
	pub admin_key: String,
}
impl AppData {
	pub async fn new(db_url: &str) -> Self {
		// SqlitePool生成
		info!("DB: {db_url}");
		let pool = Pool::connect(&db_url).await.unwrap();
		// State読み込み
		let state = match sqlx::query_scalar!("SELECT value FROM setting WHERE key=?", STATE).fetch_one(&pool).await {
			Ok(r) => r.parse().unwrap(),
			Err(sqlx::Error::RowNotFound) => {
				let state = State::Maintenance;
				let str = state.to_string();
				sqlx::query!("INSERT INTO setting VALUES(?,?)", STATE, str).execute(&pool).await.unwrap();
				state
			}
			Err(err) => panic!("{}", err),
		};
		// Key読み込み
		let (session_key, admin_key) = match sqlx::query_scalar!("SELECT value FROM setting WHERE key=?", KEY).fetch_one(&pool).await {
			Ok(r) => (cookie::Key::from(&BASE64_STANDARD.decode(&r).unwrap()), r),
			Err(sqlx::Error::RowNotFound) => {
				let session_key = cookie::Key::generate();
				let admin_key = BASE64_STANDARD.encode(&session_key.master());
				sqlx::query_scalar!("INSERT INTO setting VALUES(?,?)", KEY, admin_key).execute(&pool).await.unwrap();
				(session_key, admin_key)
			}
			Err(err) => panic!("{}", err),
		};
		// teraコア生成
		info!("Tera: {}", util::resource("template/*.html"));
		let tera = match Tera::new(&util::resource("template/*.html")) {
			Ok(mut t) => {
				t.register_filter("html", common::tera::html::<tag::Ondyst>);

				let jst = chrono::FixedOffset::east_opt(9 * 3600).unwrap();
				t.register_filter("time", common::tera::make_timestamp_filter(jst));
				t
			}
			Err(e) => {
				error!("Parsing error(s): {}", e);
				std::process::exit(1);
			}
		};
		// チャンネル生成
		let channels = RwLock::new(HashMap::default());

		AppData {
			state: StateHandle::new(state),
			pool: web::Data::new(pool),
			tera: web::Data::new(tera),
			channels: web::Data::new(channels),
			session_key,
			admin_key,
		}
	}
}
