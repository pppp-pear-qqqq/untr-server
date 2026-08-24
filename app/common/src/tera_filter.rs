use std::collections::HashMap;

use chrono::TimeZone;

use crate::html_encode::*;

pub fn html<T: TagFormat>(value: &tera::Value, args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
	let input = value.as_str().ok_or(tera::Error::msg("html filter can only be applied to strings"))?;
	let tag_format = T::from_args(args);
	let link = args.get("link").and_then(|v| v.as_bool()).unwrap_or(false);

	Ok(tera::Value::String(input.to_html(&tag_format, link)))
}

pub fn make_timestamp_filter<T: TimeZone + Send + Sync + 'static>(tz: T) -> impl tera::Filter
where
	T::Offset: std::fmt::Display,
{
	move |value: &tera::Value, _args: &HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
		let timestamp = value.as_i64().ok_or_else(|| tera::Error::msg("timestamp filter can only be applied to integers"))?;
		let dt = tz.timestamp_opt(timestamp, 0).single().ok_or_else(|| tera::Error::msg("invalid timestamp value"))?;
		let formatted = dt.format("%Y-%m-%d %H:%M:%S").to_string();
		Ok(tera::Value::String(formatted))
	}
}
