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

pub const APP_URL: &str = "http://ondyst.localhost";
// pub const APP_URL: &str = "http://ondyst.untroche.com";

pub type Identity = common::Identity<i64>;
pub type StateHandle = common::StateHandle<State>;

pub fn app(path: &str) -> String {
	format!("{}/{}", env!("CARGO_MANIFEST_DIR"), path)
}
pub fn resource(path: &str) -> String {
	format!("{}/{}/{}", env!("CARGO_MANIFEST_DIR"), if cfg!(debug_assertions) { "resource" } else { "ondyst" }, path)
}
