use std::{future, ops::Deref};

use actix_session::{Session, SessionExt};
use actix_web::{FromRequest, error::*};
use serde::de::DeserializeOwned;

const KEY: &str = "user_id";

pub struct Identity<T: DeserializeOwned>(pub T);

impl<T: DeserializeOwned + serde::Serialize> Identity<T> {
	pub fn set(session: &Session, value: T) -> Result<(), actix_session::SessionInsertError> {
		session.insert(KEY, value)
	}
	pub fn remove(session: &Session) {
		session.remove(KEY);
	}
}

impl<T: DeserializeOwned> Deref for Identity<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T: DeserializeOwned> FromRequest for Identity<T> {
	type Error = actix_web::Error;
	type Future = future::Ready<Result<Self, Self::Error>>;

	fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
		future::ready(match req.get_session().get(KEY) {
			Ok(Some(v)) => Ok(Self(v)),
			Ok(None) => Err(ErrorUnauthorized("ログインしてください")),
			Err(err) => Err(ErrorBadRequest(err)),
		})
	}
}
