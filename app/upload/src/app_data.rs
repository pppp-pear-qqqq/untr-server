use actix_web::{cookie, web};
use base64::prelude::*;
use deoxys::{DeoxysII256, aead::Generate};
use glob::glob;
use log::info;
use sqlx::SqlitePool;
use tera::Tera;

use crate::util::{self, IdMap, RouteMap, State, StateHandle, UserMap, resource};

pub type DeoxysKey = deoxys::Key<DeoxysII256>;

// 定数
pub const STATE: &str = "UPL_STATE";
const KEY: &str = "UPL_KEY";
const ENC_KEY: &str = "UPL_ENC_KEY";

#[derive(Clone)]
pub struct AppData {
	pub state: StateHandle,

	pub route_map: web::Data<RouteMap>,
	pub id_map: web::Data<IdMap>,
	pub users: web::Data<UserMap>,
	pub tera: web::Data<Tera>,
	pub enc_key: web::Data<DeoxysKey>,

	pub session_key: cookie::Key,
	pub admin_key: String,
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
			Err(err) => panic!("{err}"),
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
			Err(err) => panic!("{err}"),
		};
		let enc_key = match sqlx::query_scalar!("SELECT value FROM setting WHERE key=?", ENC_KEY).fetch_one(&pool).await {
			Ok(r) => {
				let rest = BASE64_STANDARD_NO_PAD.decode(r.as_bytes()).unwrap();
				DeoxysKey::try_from(rest.as_slice()).unwrap()
			}
			Err(sqlx::Error::RowNotFound) => {
				let key = DeoxysKey::generate();
				let sav = BASE64_STANDARD_NO_PAD.encode(key.as_slice());
				sqlx::query!("INSERT INTO setting VALUES(?,?)", ENC_KEY, sav).execute(&pool).await.unwrap();
				key
			}
			Err(err) => panic!("{err}"),
		};
		// tera
		let mut tera = Tera::default();
		tera.register_filter("urlencode", common::tera::urlencode_filter);
		tera.register_filter("time", common::tera::make_timestamp_filter(chrono::Local));
		// tera.add_template_file(resource("template.html"), Some("template.html")).unwrap();
		let files = glob(&resource("*.html")).unwrap().filter_map(|res| {
			let path = res.unwrap();
			let file_name = path.file_name().unwrap().to_str().unwrap();
			if file_name != "template.html" { Some((path.to_owned(), Some(file_name.to_string()))) } else { None }
		});
		tera.add_template_files(files).unwrap();

		let setting = util::RouteMap::load_or_default().await.unwrap();
		let id_map = IdMap::load_or_default().await.unwrap();
		let users = util::UserMap::new();

		Self {
			state: StateHandle::new(state),
			route_map: web::Data::new(setting),
			id_map: web::Data::new(id_map),
			users: web::Data::new(users),
			tera: web::Data::new(tera),
			enc_key: web::Data::new(enc_key),
			session_key,
			admin_key,
		}
	}
}
