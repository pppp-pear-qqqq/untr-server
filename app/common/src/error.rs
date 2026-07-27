use actix_web::{HttpResponse, http::StatusCode};

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

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
