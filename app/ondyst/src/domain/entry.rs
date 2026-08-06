use actix_web::{HttpResponse, Responder, error::*, http::header, web};

use crate::utils::{ActorData, Identity, Page};

/// リソース
pub fn cfg(cfg: &mut web::ServiceConfig) {}
