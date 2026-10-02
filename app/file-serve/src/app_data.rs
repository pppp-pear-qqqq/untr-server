use actix_web::{cookie, web};
use tera::Tera;

use crate::util::{State, StateHandle, UserMap};

// 定数
pub const STATE: &str = "STATE";
const KEY: &str = "KEY";

#[derive(Clone)]
pub struct AppData {
	pub state: StateHandle,
	pub tera: web::Data<Tera>,
	pub session_key: cookie::Key,
	pub admin_key: String,

	pub setting: web::Data<UserMap>,
}
impl AppData {
	pub async fn new() -> Self {
		todo!()
	}
}
