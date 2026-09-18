//! アプリケーション全体で使用するような汎用構造体・関数
mod client;
mod page;
mod state;
pub mod tag_parse;
mod webhook;

pub use client::client;
pub use page::{ActorData, Page};
pub use state::State;
pub use webhook::Webhook;

pub const APP_URL: &str = if cfg!(debug_assertions) { "http://ondyst.localhost" } else { "https://ondyst.untroche.com" };
pub const BASE62: [char; 62] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z'];

pub type Identity = common::Identity<i64>;
pub type StateHandle = common::StateHandle<State>;

#[allow(dead_code)]
pub fn app(path: &str) -> String {
	format!("{}/{}", env!("CARGO_MANIFEST_DIR"), path)
}
pub fn resource(path: &str) -> String {
	if cfg!(debug_assertions) { format!("{}/{}/{}", env!("CARGO_MANIFEST_DIR"), "resource", path) } else { format!("/app/app/ondyst/{}", path) }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Handout {
	pub title: String,
	pub body: String,
}
pub fn get_handout() -> Result<Vec<Handout>, std::io::Error> {
	let raw = std::fs::read_to_string(resource("handout.json"))?;
	let ret = serde_json::from_str(&raw)?;
	Ok(ret)
}
