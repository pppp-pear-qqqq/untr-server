use std::{fmt, str::FromStr};

#[derive(Debug, Clone, PartialEq)]
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
impl fmt::Display for State {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			State::Active => write!(f, "active"),
			State::Closed => write!(f, "closed"),
			State::Maintenance => write!(f, "maintenance"),
		}
	}
}
impl FromStr for State {
	type Err = ();
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"active" => Ok(State::Active),
			"closed" => Ok(State::Closed),
			"maintenance" => Ok(State::Maintenance),
			_ => Err(()),
		}
	}
}

impl State {
	pub fn only_active(&self) -> Result<(), actix_web::Error> {
		if self == &State::Active { Ok(()) } else { Err(actix_web::error::ErrorGone("当サイトの運営は終了しました")) }
	}
}
