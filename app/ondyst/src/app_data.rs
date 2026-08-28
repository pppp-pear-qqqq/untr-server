use actix_web::{cookie, web};
use base64::prelude::*;
use sqlx::SqlitePool;
use tera::Tera;

use crate::utils::{self, State, StateHandle, tag_parse as tag};

// 定数
pub const STATE: &str = "STATE";
const KEY: &str = "KEY";

#[derive(Clone)]
pub struct AppData {
	pub state: StateHandle,
	pub pool: web::Data<SqlitePool>,
	pub tera: web::Data<Tera>,
	pub session_key: cookie::Key,
	pub admin_key: String,
}
impl AppData {
	pub async fn new(db_url: &str) -> Self {
		// SqlitePool生成
		let pool = SqlitePool::connect(&db_url).await.unwrap();
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
		let tera = match Tera::new(&utils::resource("**/*.html")) {
			Ok(mut t) => {
				t.register_filter("html", common::tera::html::<tag::Ondyst>);

				let jst = chrono::FixedOffset::east_opt(9 * 3600).unwrap();
				t.register_filter("time", common::tera::make_timestamp_filter(jst));
				t
			}
			Err(e) => {
				println!("Parsing error(s): {}", e);
				std::process::exit(1);
			}
		};

		AppData {
			state: StateHandle::new(state),
			pool: web::Data::new(pool),
			tera: web::Data::new(tera),
			session_key,
			admin_key,
		}
	}
}
