//! アプリケーション全体で使用するような汎用構造体・関数
mod setting;
mod state;
mod usermap;

pub use setting::*;
pub use state::State;
pub use usermap::UserMap;

pub type Identity = common::Identity<Vec<u8>>;
pub type StateHandle = common::StateHandle<State>;

pub fn resource(path: impl std::fmt::Display) -> String {
	if cfg!(debug_assertions) {
		format!("{}/{}/{}", env!("CARGO_MANIFEST_DIR"), "resource", path)
	} else {
		format!("/app/app/uploader/{}", path)
	}
}
