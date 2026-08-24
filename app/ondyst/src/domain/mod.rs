mod actor;
mod entry;
mod home;
mod location;
mod pages;

use actix_web::{HttpResponse, Responder, error::*, http::header, web};
use common::{HTMLEncode, PageRender, Pagination, ReqType};
use sqlx::SqlitePool;
use tera::Tera;
use uuid::Uuid;
use validator::Validate;

use crate::utils::{ActorData, Identity, Page, StateHandle, tag_parse as tag};

pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.route("/", web::to(pages::index));
	cfg.route("info", web::to(pages::info));
	cfg.route("guide", web::to(pages::guide));
	cfg.service(web::scope("entry").configure(entry::cfg));
	cfg.service(web::scope("actor").configure(actor::cfg));
	cfg.service(web::scope("location").configure(location::cfg));
	cfg.service(web::scope("home").configure(home::cfg));
}
