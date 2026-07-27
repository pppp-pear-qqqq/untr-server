#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReqType {
	Document,
	Iframe,
	Image,
	Script,
	Style,
	Empty,
}
impl actix_web::FromRequest for ReqType {
	type Error = actix_web::Error;
	type Future = std::future::Ready<std::result::Result<Self, Self::Error>>;

	fn from_request(req: &actix_web::HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
		let dest = match req.headers().get("sec-fetch-dest") {
			Some(dest) => match dest.as_bytes() {
				b"document" => Self::Document,
				b"iframe" => Self::Iframe,
				b"image" => Self::Image,
				b"script" => Self::Script,
				b"style" => Self::Style,
				_ => Self::Empty,
			},
			None => Self::Empty,
		};
		std::future::ready(Ok(dest))
	}
}
