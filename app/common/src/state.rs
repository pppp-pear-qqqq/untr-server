use std::{
	future::{Ready, ready},
	sync::{Arc, RwLock},
};

use actix_web::FromRequest;

pub trait IsMaintenance {
	const MAINTENANCE_MESSAGE: &'static str = "メンテナンス中です";
	fn is_maintenance(&self) -> bool;
}

#[derive(Clone)]
pub struct StateHandle<T: Clone + IsMaintenance + 'static>(Arc<RwLock<T>>);

impl<T: Clone + IsMaintenance + 'static> StateHandle<T> {
	pub fn new(state: T) -> Self {
		Self(Arc::new(RwLock::new(state)))
	}
	pub fn get(&self) -> T {
		self.0.read().unwrap().clone()
	}
	pub fn set(&self, state: T) {
		*self.0.write().unwrap() = state;
	}
}

impl<T: Clone + IsMaintenance + 'static> FromRequest for StateHandle<T> {
	type Error = actix_web::Error;
	type Future = Ready<Result<Self, Self::Error>>;

	fn from_request(req: &actix_web::HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
		let state = match req.app_data::<StateHandle<T>>() {
			Some(data) => data.clone(),
			None => return ready(Err(actix_web::error::ErrorInternalServerError("State is not configured"))),
		};
		if state.get().is_maintenance() { ready(Err(actix_web::error::ErrorServiceUnavailable(T::MAINTENANCE_MESSAGE))) } else { ready(Ok(state)) }
	}
}
