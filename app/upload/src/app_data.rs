use actix_web::{cookie, web};
use base64::prelude::*;
use log::info;
use sqlx::SqlitePool;

use crate::util::{self, RouteMap, State, StateHandle, UserMap};

// 定数
pub const STATE: &str = "UPL_STATE";
const KEY: &str = "UPL_KEY";

#[derive(Clone)]
pub struct AppData {
	pub state: StateHandle,
	pub session_key: cookie::Key,
	pub admin_key: String,

	pub setting: web::Data<RouteMap>,
	pub users: web::Data<UserMap>,
}
impl AppData {
	pub async fn new() -> Self {
		// SqlitePool生成
		let db_url = std::env::var("DATABASE_URL").expect(&format!("`DATABASE_URL` is undefined"));
		info!("DB: {db_url}");
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
		// setting
		let setting = util::RouteMap::new(); // TODO
		// users
		let users = util::UserMap::load();

		Self {
			state: StateHandle::new(state),
			session_key,
			admin_key,
			setting: web::Data::new(setting),
			users: web::Data::new(users),
		}
	}
}
