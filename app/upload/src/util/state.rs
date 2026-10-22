#[derive(Debug, Clone, PartialEq, strum::EnumString, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum State {
	Active,
	Closed,
	Maintenance,
}

impl common::IsMaintenance for State {
	fn is_maintenance(&self) -> bool {
		self == &State::Maintenance
	}
}

impl State {
	pub fn only_active(&self) -> Result<(), actix_web::Error> {
		if self == &State::Active { Ok(()) } else { Err(actix_web::error::ErrorGone("当サイトの運営は終了しました")) }
	}
}
