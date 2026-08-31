use std::{fmt, str::FromStr};

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
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

impl State {
	// 静的ページ	StateHandleを受け取らない
	// View		is_maintenanceに譲渡

	// Prepare	閲覧のみ可で登録や変更・主機能へのアクセスは不可
	// Register	閲覧と登録・登録情報の編集のみ可
	// Active	全機能へのアクセス解放
	// Closed	閲覧のみ可

	// そのリソースが何を行うか
	// 閲覧・登録変更・主機能の3分類？

	pub fn only_active(&self) -> Result<(), actix_web::Error> {
		match self {
			State::Active => Ok(()),
			State::Prepare | State::Register => Err(actix_web::error::ErrorServiceUnavailable("準備中です。開催までお待ちください")),
			State::Closed => Err(actix_web::error::ErrorGone("当サイトの運営は終了しました")),
			_ => unreachable!(),
		}
	}
	pub fn only_open(&self) -> Result<(), actix_web::Error> {
		match self {
			State::Register | State::Active => Ok(()),
			State::Prepare => Err(actix_web::error::ErrorServiceUnavailable("準備中です。開催までお待ちください")),
			State::Closed => Err(actix_web::error::ErrorGone("当サイトの運営は終了しました")),
			_ => unreachable!(),
		}
	}
}
