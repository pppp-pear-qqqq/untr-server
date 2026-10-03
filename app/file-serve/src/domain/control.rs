use super::*;
use crate::util::{self, PathMap, UserMap};

use actix_multipart::Multipart;
use futures_util::TryStreamExt as _;
use tokio::io::AsyncWriteExt as _;

pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.service(web::resource("").get(index).post(post));
	cfg.service(web::resource("{path:.+}").post(path_post).patch(path_patch).delete(path_delete));
}

async fn index() -> common::Result<impl Responder> {
	Ok("TODO")
}

async fn post(mut payload: Multipart, id: Identity, setting: web::Data<UserMap>) -> common::Result<impl Responder> {
	let mut result = String::new();
	let mut new: Vec<(String, Uuid)> = Vec::new();

	// 全フィールドをループ
	while let Ok(Some(mut field)) = payload.try_next().await {
		let content = field.content_disposition().ok_or_else(|| ErrorBadRequest("その種類のmultipartリクエストには対応していません"))?;

		if let Some(name) = content.get_filename() {
			if setting.contains_key(name) {
				result.push_str(&format!("{}: 既に同名のパスが設定されています\n", name));
				continue;
			}
			let name = name.to_string();

			// --- ファイルの保存処理 ---
			let file = Uuid::new_v4();
			let mut f = tokio::fs::File::create(&resource(&format!("upload/{}", file))).await?;

			while let Ok(Some(chunk)) = field.try_next().await {
				f.write_all(&chunk).await?;
			}

			// 成功したらリストに追加
			new.push((name, file));
		}
	}

	// 1つもファイルが保存されなかった場合のエラーハンドリング
	if new.is_empty() {
		return Err(ErrorBadRequest("ファイルがありません").into());
	}

	for (name, file) in &new {
		let username = "";
		let setting = setting.get(username);
		// TODO
	}

	Ok(HttpResponse::Ok().json(new))
}

async fn path_post(path: web::Path<String>, mut payload: Multipart) -> common::Result<impl Responder> {
	Ok("TODO")
}

async fn path_patch(path: web::Path<String>) -> common::Result<impl Responder> {
	Ok("TODO")
}

async fn path_delete(path: web::Path<String>) -> common::Result<impl Responder> {
	Ok("TODO")
}
