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

pub fn resource(path: impl std::fmt::Display) -> String {
	if cfg!(debug_assertions) { format!("resource/{path}") } else { format!("resource/portal/{path}") }
}
