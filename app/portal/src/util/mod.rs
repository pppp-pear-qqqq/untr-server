//! アプリケーション全体で使用するような汎用構造体・関数
mod client;
mod guard;
mod page;
mod state;
pub mod tag_parse;

pub use client::client;
pub use guard::*;
pub use page::{Page, UserData};
pub use state::State;
pub use tag_parse as tag;

pub type Identity = common::Identity<Vec<u8>>;
pub type StateHandle = common::StateHandle<State>;

#[allow(dead_code)]
pub fn app(path: &str) -> String {
	format!("{}/{}", env!("CARGO_MANIFEST_DIR"), path)
}
pub fn resource(path: &str) -> String {
	if cfg!(debug_assertions) { format!("{}/{}/{}", env!("CARGO_MANIFEST_DIR"), "resource", path) } else { format!("/app/app/portal/{}", path) }
}
