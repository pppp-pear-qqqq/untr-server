cfg_if::cfg_if! {
	if #[cfg(not(target_arch = "wasm32"))] {
		mod admin_guard;
		mod error;
		mod identity;
		mod page_render;
		mod pagination;
		mod req_type;
		mod state;
		pub mod tera_filter;

		pub use admin_guard::*;
		pub use error::*;
		pub use identity::Identity;
		pub use page_render::PageRender;
		pub use pagination::Pagination;
		pub use req_type::ReqType;
		pub use state::{IsMaintenance, StateHandle};
		pub use tera_filter as tera;
	}
}

mod html_encode;
pub use html_encode::*;
