mod admin;
mod control;
mod delivery;
mod entry;

use actix_web::{HttpResponse, Responder, error::*, http::header, web};
// use common::{PageRender, Pagination, ReqType};
#[allow(unused_imports)]
use log::{debug, error, info};
use tera::Tera;
use uuid::Uuid;
use validator::Validate;

use crate::util::{Identity, StateHandle};

pub fn make_cfg(admin_key: String) -> std::io::Result<impl FnOnce(&mut web::ServiceConfig)> {
	Ok(|cfg: &mut web::ServiceConfig| {
		cfg.route("/", web::get().to(index));
		cfg.route("login", web::to(entry::login));
		cfg.route("logout", web::to(entry::logout));
		cfg.service(web::scope("control").configure(control::cfg));
		cfg.route("u/{username}/{path:.+}", web::get().to(delivery::plane));
		cfg.route("b/{encoded}", web::get().to(delivery::b64));
		cfg.service(web::scope("admin").wrap(common::AdminGuardMiddleware(admin_key)).configure(admin::cfg));
	})
}

async fn index(tera: web::Data<Tera>) -> common::Result<impl Responder> {
	let ctx = tera::Context::new();
	let body = tera.render("index.html", &ctx)?;
	Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
}
