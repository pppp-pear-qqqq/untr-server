//! アプリケーション全体で使用するような汎用構造体・関数
mod error;
mod page;
mod state;

pub use error::mw_err_format;
pub use page::{Page, PageType};
pub use state::State;

pub type Identity = common::Identity<Vec<u8>>;
pub type StateHandle = common::StateHandle<State>;

pub fn app(path: &str) -> String {
	format!("app/portal/{}", path)
}
pub fn resource(path: &str) -> String {
	format!("app/portal/resource/{}", path)
}
