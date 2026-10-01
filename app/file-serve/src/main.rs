use std::io;

use actix_web::{App, HttpResponse, HttpServer, middleware, web};

#[actix_web::main]
async fn main() -> Result<(), io::Error> {
	env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

	// 環境変数読み込み
	let host = load_env("SERVER_HOST");
	let port = load_env("SERVER_PORT");

	// let app_data = app_data::AppData::new(&db_url).await;

	let server = HttpServer::new(move || {
		// let app_data = app_data.clone();
		// let session = SessionMiddleware::builder(storage::CookieSessionStore::default(), app_data.session_key)
		// 	.session_lifecycle(PersistentSession::default().session_ttl(cookie::time::Duration::days(14)).session_ttl_extension_policy(TtlExtensionPolicy::OnEveryRequest))
		// 	.build();
		App::new()
			.wrap(middleware::Logger::default())
			.wrap(middleware::NormalizePath::trim())
			// .wrap(session)
			// .wrap(middleware::from_fn(common::mw_err_format::<util::Page>))
			.default_service(web::to(HttpResponse::NotFound))
		// .app_data(app_data.state)
		// .app_data(app_data.pool)
		// .app_data(app_data.tera)
		// .configure(domain::make_cfg(app_data.admin_key))
	});
	server.bind(format!("{host}:{port}"))?.run().await
}

fn load_env(path: &str) -> String {
	std::env::var(path).expect(&format!("`{path}` is undefined"))
}
