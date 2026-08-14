mod auth;
mod entry;
mod home;
mod pages;
mod user;
mod webhook;

use actix_web::{HttpResponse, Responder, error::*, http::header, web};
use common::{PageRender, ReqType};
use sqlx::SqlitePool;
use tera::Tera;
use uuid::Uuid;

use crate::utils::{Identity, Page, UserData};

pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.route("/", web::get().to(pages::index));
	cfg.route("info", web::get().to(pages::info));
	cfg.service(web::scope("entry").configure(entry::cfg));
	cfg.service(web::scope("auth").configure(auth::cfg));
	cfg.service(web::scope("home").configure(home::cfg));
	cfg.service(web::scope("user").configure(user::cfg));
	cfg.service(web::scope("webhook").configure(webhook::cfg));
}
