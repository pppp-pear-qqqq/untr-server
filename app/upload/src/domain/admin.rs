use super::*;

pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.route("change_name", web::to(change_name));
}

async fn change_name() -> common::Result<HttpResponse> {
	todo!()
}
