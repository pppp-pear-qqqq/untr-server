use actix_session::SessionExt as _;
use actix_web::{
	HttpResponse,
	body::{BoxBody, EitherBody},
	dev::{Service, ServiceRequest, ServiceResponse, Transform},
};
use futures_util::future::{LocalBoxFuture, Ready, ok};

const ADMIN_KEY: &str = "admin";
const AUTHORIZE: &str = "Authorize";

pub struct AdminGuardMiddleware(pub String);

impl<S, B> Transform<S, ServiceRequest> for AdminGuardMiddleware
where
	S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
	B: 'static,
{
	type Response = ServiceResponse<EitherBody<B, BoxBody>>;
	type Error = actix_web::Error;
	type InitError = ();
	type Transform = AdminGuardMiddlewareImpl<S>;
	type Future = Ready<Result<Self::Transform, Self::InitError>>;

	fn new_transform(&self, service: S) -> Self::Future {
		ok(AdminGuardMiddlewareImpl { service, key: self.0.clone() })
	}
}

pub struct AdminGuardMiddlewareImpl<S> {
	service: S,
	key: String,
}

impl<S, B> Service<ServiceRequest> for AdminGuardMiddlewareImpl<S>
where
	S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
	B: 'static,
{
	type Response = ServiceResponse<EitherBody<B, BoxBody>>;
	type Error = actix_web::Error;
	type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

	fn poll_ready(&self, ctx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
		self.service.poll_ready(ctx)
	}

	fn call(&self, req: ServiceRequest) -> Self::Future {
		let session = req.get_session();
		if let Some(Ok(value)) = req.headers().get(AUTHORIZE).map(|x| x.to_str()) {
			let _ = session.insert(ADMIN_KEY, value);
		}
		if let Ok(Some(admin)) = session.get::<String>(ADMIN_KEY) {
			if admin == self.key {
				let fut = self.service.call(req);
				return Box::pin(async move { Ok(fut.await?.map_into_left_body()) });
			}
		}
		Box::pin(async { Ok(req.into_response(HttpResponse::Forbidden().body("Access denied.").map_into_right_body())) })
	}
}
