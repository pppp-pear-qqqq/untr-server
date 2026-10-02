//! アプリケーション全体で使用するような汎用構造体・関数
mod setting;
mod state;

pub use setting::*;
pub use state::State;

pub type Identity = common::Identity<Vec<u8>>;
pub type StateHandle = common::StateHandle<State>;

pub fn app(path: &str) -> String {
	format!("{}/{}", env!("CARGO_MANIFEST_DIR"), path)
}
pub fn resource(path: &str) -> String {
	if cfg!(debug_assertions) {
		format!("{}/{}/{}", env!("CARGO_MANIFEST_DIR"), "resource", path)
	} else {
		format!("/app/app/uploader/{}", path)
	}
}
