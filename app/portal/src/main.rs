mod domain;
mod utils;

use std::io;

use actix_session::{SessionMiddleware, config::PersistentSession, storage};
use actix_web::{App, HttpResponse, HttpServer, cookie, middleware, web};
use base64::prelude::*;
use sqlx::SqlitePool;
use tera::Tera;

// 定数
const STATE: &str = "STATE";
const KEY: &str = "KEY";

// memo: ファイル分けしていないのは、この辺りも含めてアプリごとに定義するべきだから
#[derive(Clone)]
pub struct AppData {
	pub state: utils::StateHandle,
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
				let state = utils::State::Maintenance;
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
			Ok(t) => t,
			Err(e) => {
				println!("Parsing error(s): {}", e);
				std::process::exit(1);
			}
		};

		AppData {
			state: utils::StateHandle::new(state),
			pool: web::Data::new(pool),
			tera: web::Data::new(tera),
			session_key,
			admin_key,
		}
	}
}

fn load_env(path: &str) -> String {
	std::env::var(path).expect(&format!("`{path}` is undefined"))
}

#[actix_web::main]
async fn main() -> Result<(), io::Error> {
	#[cfg(feature = "test")]
	{
		use dotenv;
		if let Err(e) = dotenv::from_path(utils::app(".env")) {
			eprintln!("Failed to load .env file: {}", e);
			std::process::exit(1);
		}
	}

	env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

	// 環境変数読み込み
	let host = load_env("SERVER_HOST");
	let port = load_env("SERVER_PORT");
	let db_url = load_env("DATABASE_URL");

	let app_data = AppData::new(&db_url).await;

	println!("portal-admin: {}", app_data.admin_key);

	let server = HttpServer::new(move || {
		// memo: AppData側にAppの生成関数を組み込まない(組み込めない)のは、App<T>のTが特定困難または不定であるため
		// その辺り暗黙でよしなにできるんだったらやりたいが、仮にできたとしてもapp_dataが持たない設定の所在に困る
		let app_data = app_data.clone();
		let session = SessionMiddleware::builder(storage::CookieSessionStore::default(), app_data.session_key)
			.cookie_secure(false)
			.session_lifecycle(PersistentSession::default().session_ttl(cookie::time::Duration::days(14)))
			.build();
		let app = App::new()
			.wrap(middleware::Logger::default())
			.wrap(middleware::NormalizePath::trim())
			.wrap(session)
			.wrap(middleware::from_fn(common::mw_err_format::<utils::Page>))
			.default_service(web::to(|| HttpResponse::NotFound()))
			.app_data(app_data.state)
			.app_data(app_data.pool)
			.app_data(app_data.tera)
			.configure(domain::cfg);
		#[cfg(feature = "test")]
		let app = {
			use actix_files::Files;
			app.service(Files::new("/script", utils::resource("script/")).prefer_utf8(true))
				.service(Files::new("/style", utils::resource("style/")).prefer_utf8(true))
				.service(Files::new("/image", utils::resource("image/")).prefer_utf8(true))
		};
		app
	});
	server.bind(format!("{host}:{port}"))?.run().await
}
