use actix_web::{
	HttpResponse,
	body::{BoxBody, MessageBody},
	dev,
	http::header,
	middleware, web,
};
use common::ReqType;
use tera::Tera;

use super::Page;

pub async fn mw_err_format(req: dev::ServiceRequest, next: middleware::Next<impl MessageBody + 'static>) -> std::result::Result<dev::ServiceResponse<BoxBody>, actix_web::Error> {
	// 実行
	let res = next.call(req).await?.map_into_boxed_body();
	let (req, res) = res.into_parts();

	// エラー整形
	if let Some(err) = res.error() {
		let req_type = req.app_data::<ReqType>().copied().unwrap_or(ReqType::Empty);
		if req_type == ReqType::Document {
			if let (Some(tmpl), Ok(mut ctx)) = (req.app_data::<web::Data<Tera>>(), Page::default().ctx()) {
				ctx.insert("body", &err.to_string());
				if let Ok(body) = tmpl.render("error.html", &ctx) {
					let res = HttpResponse::build(res.status()).content_type(header::ContentType::html()).body(body);
					return Ok(dev::ServiceResponse::new(req, res));
				}
			}
		}
	}
	Ok(dev::ServiceResponse::new(req, res))
}
