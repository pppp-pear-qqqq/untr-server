use std::fmt;

use actix_web::{
	HttpResponse,
	body::{BoxBody, MessageBody},
	dev,
	http::{StatusCode, header},
	middleware, web,
};
use tera::Tera;

#[derive(Debug)]
pub struct Error {
	status: StatusCode,
	cause: Box<dyn std::error::Error + 'static>,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
	pub fn new<E>(status: StatusCode, error: E) -> Self
	where
		E: std::error::Error + 'static,
	{
		Self { status, cause: Box::new(error) }
	}
}

impl<E> From<E> for Error
where
	E: std::error::Error + 'static,
{
	fn from(err: E) -> Self {
		let boxed: Box<dyn std::error::Error> = Box::new(err);

		if let Some(actix_err) = boxed.downcast_ref::<actix_web::Error>() {
			let status = actix_err.as_response_error().status_code();
			Self { status, cause: boxed }
		} else {
			Self {
				status: StatusCode::INTERNAL_SERVER_ERROR,
				cause: boxed,
			}
		}
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.cause.fmt(f)
	}
}

impl actix_web::ResponseError for Error {
	fn status_code(&self) -> StatusCode {
		self.status
	}
	fn error_response(&self) -> HttpResponse {
		HttpResponse::build(self.status_code()).body(self.to_string())
	}
}

pub async fn mw_err_format<Page: Default + crate::PageRender>(
	req: dev::ServiceRequest,
	next: middleware::Next<impl MessageBody + 'static>,
) -> std::result::Result<dev::ServiceResponse<BoxBody>, actix_web::Error> {
	// 実行
	let res = next.call(req).await?.map_into_boxed_body();
	let (req, res) = res.into_parts();

	// エラー整形
	if let Some(err) = res.error() {
		let req_type = req.app_data::<crate::ReqType>().copied().unwrap_or(crate::ReqType::Empty);
		if req_type == crate::ReqType::Document {
			if let Some(tmpl) = req.app_data::<web::Data<Tera>>() {
				let mut ctx = tera::Context::new();
				ctx.insert("body", &err.to_string());
				if let Ok(body) = Page::default().render_with_ctx("error.html", &tmpl, ctx) {
					let res = HttpResponse::build(res.status()).content_type(header::ContentType::html()).body(body);
					return Ok(dev::ServiceResponse::new(req, res));
				}
			}
		}
	}
	Ok(dev::ServiceResponse::new(req, res))
}
