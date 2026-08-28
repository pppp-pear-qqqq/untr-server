mod admin;
mod auth;
mod entry;
mod home;
mod pages;
mod user;
mod webhook;

use actix_web::{HttpResponse, Responder, error::*, guard, http::header, web};
use common::{PageRender, Pagination, ReqType};
#[allow(unused_imports)]
use log::{debug, error, info};
use sqlx::SqlitePool as Pool;
use tera::Tera;
use uuid::Uuid;
use validator::Validate;

use crate::utils::{Identity, Page, StateHandle, UserData, is_internal};

pub fn make_cfg(admin_key: String) -> impl FnOnce(&mut web::ServiceConfig) {
	|cfg: &mut web::ServiceConfig| {
		cfg.route("/", web::get().to(pages::index));
		cfg.route("info", web::get().to(pages::info));
		cfg.service(web::scope("entry").configure(entry::cfg));
		cfg.service(web::scope("auth").configure(auth::cfg));
		cfg.service(web::scope("home").configure(home::cfg));
		cfg.service(web::scope("user").configure(user::cfg));
		cfg.service(web::scope("webhook").guard(guard::fn_guard(is_internal)).configure(webhook::cfg));
		cfg.service(web::scope("admin").wrap(common::AdminGuardMiddleware(admin_key)).configure(admin::cfg));
	}
}
