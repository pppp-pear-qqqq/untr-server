mod actor;
mod admin;
mod entry;
mod home;
mod location;
mod pages;

use actix_web::{HttpResponse, Responder, error::*, http::header, web};
use common::{HTMLEncode, PageRender, Pagination, ReqType};
#[allow(unused_imports)]
use log::{debug, error, info};
use sqlx::SqlitePool as Pool;
use tera::Tera;
use uuid::Uuid;
use validator::Validate;

use crate::util::{ActorData, Identity, Page, StateHandle, tag_parse as tag};

pub fn make_cfg(admin_key: String) -> impl FnOnce(&mut web::ServiceConfig) {
	move |cfg| {
		cfg.route("/", web::to(pages::index));
		cfg.route("info", web::to(pages::info));
		cfg.route("guide", web::to(pages::guide));
		cfg.service(web::scope("entry").configure(entry::cfg));
		cfg.service(web::scope("actor").configure(actor::cfg));
		cfg.service(web::scope("location").configure(location::cfg));
		cfg.service(web::scope("home").configure(home::cfg));
		cfg.service(web::scope("admin").wrap(common::AdminGuardMiddleware(admin_key)).configure(admin::cfg));
	}
}
