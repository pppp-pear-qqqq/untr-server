mod account;
mod admin;
mod delivery;
mod upload;

use actix_web::{HttpResponse, Responder, error::*, http::header, web};
use common::{PageRender, Pagination, ReqType};
#[allow(unused_imports)]
use log::{debug, error, info};
use tera::Tera;
use uuid::Uuid;
use validator::Validate;

use crate::util::{Identity, StateHandle, resource};

pub fn make_cfg(admin_key: String) -> std::io::Result<impl FnOnce(&mut web::ServiceConfig)> {
	let index = make_static_page("index.html")?;

	Ok(|cfg: &mut web::ServiceConfig| {
		cfg.route("/", web::get().to(index));
		cfg.service(web::scope("u/{username}").route("", web::to(upload::home)).route("{path}", web::get().to(delivery::plane)));
		cfg.service(web::scope("b").route("{encoded}", web::get().to(delivery::b64)));
		cfg.service(web::scope("e").route("{encoded}", web::get().to(delivery::encrypt)));
		cfg.service(web::scope("admin").wrap(common::AdminGuardMiddleware(admin_key)).configure(admin::cfg));
	})
}

pub fn make_static_page(path: &str) -> std::io::Result<impl Fn() -> std::future::Ready<HttpResponse> + Clone> {
	let body = web::Bytes::from(std::fs::read(resource(path))?);
	Ok(move || {
		let res = HttpResponse::Ok().content_type(header::ContentType::html()).body(body.clone());
		std::future::ready(res)
	})
}
