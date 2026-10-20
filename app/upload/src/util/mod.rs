//! アプリケーション全体で使用するような汎用構造体・関数
mod client;
pub mod setting;
mod setting_dto;
mod state;
mod usermap;

pub use client::client;
pub use setting::*;
pub use state::State;
pub use usermap::UserMap;

pub type Identity = common::Identity<Vec<u8>>;
pub type StateHandle = common::StateHandle<State>;

pub fn resource(path: impl std::fmt::Display) -> String {
	if cfg!(debug_assertions) { format!("resource/{path}") } else { format!("resource/upload/{path}") }
}

pub fn upload(path: impl std::fmt::Display) -> String {
	if cfg!(debug_assertions) { format!("upload/{path}") } else { format!("data/upload/{path}") }
}
