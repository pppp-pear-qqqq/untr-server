use chrono::TimeZone;
use tera::TeraResult;

use crate::html_encode::*;

pub fn urlencode_filter(value: String, _kwargs: tera::Kwargs, _state: &tera::State) -> TeraResult<String> {
	Ok(urlencoding::encode(&value).into_owned())
}

pub fn html_filter<T: TagFormat>(value: String, kwargs: tera::Kwargs, _state: &tera::State) -> TeraResult<String> {
	let link = kwargs.get("link")?.unwrap_or(false);
	let tag_format = T::from_args(&kwargs);

	Ok(value.to_html(&tag_format, link))
}

pub fn make_timestamp_filter<T: TimeZone + Send + Sync + 'static>(tz: T) -> impl tera::Filter<i64, TeraResult<String>>
where
	T::Offset: std::fmt::Display,
{
	move |timestamp: i64, _kwargs: tera::Kwargs, _state: &tera::State| -> TeraResult<String> {
		let dt = chrono::DateTime::from_timestamp_secs(timestamp).ok_or_else(|| tera::Error::message("invalid timestamp value"))?.with_timezone(&tz);
		Ok(dt.format("%Y-%m-%d %H:%M:%S").to_string())
	}
}
