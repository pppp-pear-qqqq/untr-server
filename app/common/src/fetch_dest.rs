use std::str::FromStr;

use actix_web::{FromRequest, error::ErrorBadRequest};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize, strum::EnumString, strum::Display)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum FetchDest {
	Audio,
	Audioworklet,
	Document,
	Embed,
	Empty,
	Fencedframe,
	Font,
	Frame,
	Iframe,
	Image,
	Manifest,
	Object,
	Paintworklet,
	Report,
	Script,
	Serviceworker,
	Sharedworker,
	Style,
	Track,
	Video,
	Webidentity,
	Worker,
	Xslt,
}

impl FromRequest for FetchDest {
	type Error = actix_web::Error;
	type Future = std::future::Ready<Result<Self, Self::Error>>;

	fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
		let header_value = match req.headers().get("sec-fetch-dest") {
			Some(val) => val,
			None => return std::future::ready(Err(ErrorBadRequest("Missing Sec-Fetch-Dest header"))),
		};
		let dest_str = match header_value.to_str() {
			Ok(s) => s,
			Err(_) => return std::future::ready(Err(ErrorBadRequest("Invalid Sec-Fetch-Dest encoding"))),
		};
		std::future::ready(match Self::from_str(dest_str) {
			Ok(dest) => Ok(dest),
			Err(_) => Err(ErrorBadRequest(format!("Unknown Sec-Fetch-Dest: {}", dest_str))),
		})
	}
}
