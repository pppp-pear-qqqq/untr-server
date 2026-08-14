mod app_data;
mod domain;
mod utils;

use std::io;

use actix_session::{SessionMiddleware, config::PersistentSession, storage};
use actix_web::{App, HttpResponse, HttpServer, cookie, middleware, web};

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

	let app_data = app_data::AppData::new(&db_url).await;

	println!("ondyst-admin: {}", app_data.admin_key);

	let server = HttpServer::new(move || {
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

fn load_env(path: &str) -> String {
	std::env::var(path).expect(&format!("`{path}` is undefined"))
}
