use std::{fmt, str::FromStr};

#[derive(Debug, Clone, PartialEq)]
pub enum State {
	Prepare,
	Register,
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
			State::Prepare => write!(f, "prepare"),
			State::Register => write!(f, "register"),
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
			"prepare" => Ok(State::Prepare),
			"register" => Ok(State::Register),
			"active" => Ok(State::Active),
			"closed" => Ok(State::Closed),
			"maintenance" => Ok(State::Maintenance),
			_ => Err(()),
		}
	}
}
