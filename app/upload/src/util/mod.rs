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

use base64::prelude::*;

pub type Identity = common::Identity<Vec<u8>>;
pub type StateHandle = common::StateHandle<State>;

pub fn resource(path: impl std::fmt::Display) -> String {
	if cfg!(debug_assertions) { format!("resource/{path}") } else { format!("resource/upload/{path}") }
}

pub async fn upload_create_dir<T: AsRef<[u8]>>(username: T, key: impl std::fmt::Display) -> std::io::Result<String> {
	let username = BASE64_URL_SAFE_NO_PAD.encode(username);
	if cfg!(debug_assertions) {
		tokio::fs::create_dir_all(&format!("upload/{username}")).await?;
		Ok(format!("upload/{username}/{key}"))
	} else {
		tokio::fs::create_dir_all(&format!("data/upload/{username}")).await?;
		Ok(format!("data/upload/{username}/{key}"))
	}
}
pub fn upload_dir<T: AsRef<[u8]>>(username: T, key: impl std::fmt::Display) -> String {
	let enc = BASE64_URL_SAFE_NO_PAD.encode(username);
	if cfg!(debug_assertions) { format!("upload/{enc}/{key}") } else { format!("data/upload/{enc}/{key}") }
}
