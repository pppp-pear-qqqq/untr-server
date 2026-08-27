use std::future;

use actix_web::{FromRequest, HttpRequest, dev::Payload, web};

// 1. Serde用の内部構造体（未入力かどうかをOptionで判定できるようにする）
#[derive(serde::Deserialize)]
struct PaginationQuery {
	offset: Option<usize>,
	limit: Option<usize>,
}

// 2. 表向きの構造体（ここに Const Generics を持たせる。Serdeからは切り離す）
pub struct Pagination<const DEFAULT_LIMIT: usize, const MAX_LIMIT: usize = 100> {
	pub offset: usize,
	pub limit: usize,
}

impl<const DEFAULT_LIMIT: usize, const MAX_LIMIT: usize> Pagination<DEFAULT_LIMIT, MAX_LIMIT> {
	pub fn page(&self) -> usize {
		self.offset.checked_div(self.limit).unwrap_or_default()
	}
}

// 3. Actix-web の FromRequest を実装（リクエストから直接パースする）
impl<const DEFAULT_LIMIT: usize, const MAX_LIMIT: usize> FromRequest for Pagination<DEFAULT_LIMIT, MAX_LIMIT> {
	type Error = actix_web::Error;
	type Future = future::Ready<Result<Self, Self::Error>>;

	fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
		// クエリ文字列を Option 付きの内部構造体としてパース
		let query_res = web::Query::<PaginationQuery>::from_query(req.query_string());

		match query_res {
			Ok(q) => {
				// 値が None ならここでデフォルト値を差し込む
				let pagination = Pagination {
					offset: q.offset.unwrap_or(0),
					limit: q.limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT),
				};
				future::ready(Ok(pagination))
			}
			Err(e) => future::ready(Err(e.into())),
		}
	}
}
