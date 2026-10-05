use super::*;
use crate::util::{self, UserMap};

use actix_multipart::{
	Multipart,
	form::{self, MultipartForm, tempfile::TempFile},
};
use actix_web::mime;

pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.service(web::resource("").get(index).post(post));
	cfg.service(web::resource("{path:.+}").post(path_post).patch(path_patch).delete(path_delete));
}

async fn index() -> common::Result<impl Responder> {
	Ok("TODO")
}

#[derive(MultipartForm)]
struct Post {
	files: Vec<TempFile>,
}
async fn post(MultipartForm(info): MultipartForm<Post>, id: Identity, setting: web::Data<UserMap>) -> common::Result<impl Responder> {
	let mut result = String::new();
	let mut new = Vec::new();

	let username = "TODO".into(); // ユーザー名
	let mut setting = setting.entry(username).or_default();

	for file in info.files {
		if file.file_name.is_none() {
			result.push_str("[ERROR] ファイル名を読み込めません\n");
			continue;
		}
		let name = file.file_name.unwrap();

		if setting.contains_key(&name) {
			result.push_str(&format!("[ERROR] {}: 既に同名のパスが設定されています\n", name));
			continue;
		}

		// ファイルの保存処理
		let key = Uuid::new_v4();
		match file.file.persist(resource(&format!("upload/{}", key))) {
			Ok(_) => {
				// 成功したらリストに追加
				let mime = file.content_type.unwrap_or(mime::TEXT_PLAIN);
				new.push((name, key, mime));
			}
			Err(e) => {
				result.push_str(&format!("[ERROR] {}: {}\n", name, e));
			}
		}
	}

	// 1つもファイルが保存されなかった場合のエラーハンドリング
	if new.is_empty() {
		return Err(ErrorBadRequest("ファイルがありません").into());
	}

	for (name, key, mime) in new {
		// デフォルトの設定作成
		let mut value = util::Setting::default();
		value.insert(None, util::Resource::File { key, mime });

		// 更新
		setting.insert(name, value);
	}

	Ok(HttpResponse::Ok().finish())
}

#[derive(serde::Deserialize)]
#[serde(tag = "type")]
enum FormConfig {
	File,
	RandomFile {
		weights: Vec<u32>, // ファイルの数と一致させる
		cache: bool,
	},
	FontRender {
		width: u32,
		height: u32,
		line_length: u32,
	},
}

#[derive(MultipartForm)]
struct PathPost {
	fetch_dest: Option<form::text::Text<util::FetchDest>>,
	files: Vec<TempFile>,
	config: form::text::Text<FormConfig>,
}
async fn path_post(path: web::Path<String>, MultipartForm(info): MultipartForm<PathPost>, id: Identity, setting: web::Data<UserMap>) -> common::Result<impl Responder> {
	let username = "TODO".to_string();
	let target_path = path.into_inner();

	let mut setting = setting.entry(username).or_default();

	// 1. 既存チェック
	if setting.contains_key(&target_path) {
		return Err(ErrorBadRequest("既に同名のパスが設定されています").into());
	}

	// 3. ファイルの保存
	let mut saved_files = Vec::new();
	for file in info.files {
		saved_files.push(save_file(file)?);
	}

	// 4. Resource構築と保存
	let resource = build_resource(info.config.into_inner(), saved_files)?;

	let mut new_setting = util::Setting::default();
	new_setting.insert(info.fetch_dest.map(|x| x.into_inner()), resource);

	setting.insert(target_path, new_setting);

	Ok(HttpResponse::Ok().finish())
}

#[derive(MultipartForm)]
struct PathPatch {
	new_path: Option<form::text::Text<String>>,
	fetch_dest: Option<form::text::Text<util::FetchDest>>,
	add_files: Vec<TempFile>,
	rem_files: Vec<form::text::Text<Uuid>>,
	config: Option<form::text::Text<FormConfig>>,
}
async fn path_patch(path: web::Path<String>, MultipartForm(info): MultipartForm<PathPatch>, id: Identity, setting: web::Data<UserMap>) -> common::Result<impl Responder> {
	let username = "TODO".to_string();
	let current_path = path.into_inner();
	let target_path = info.new_path.map(|p| p.into_inner()).unwrap_or_else(|| current_path.clone());

	let mut setting = setting.entry(username).or_default();

	// 1. パス名変更時の競合チェック
	if current_path != target_path && setting.contains_key(&target_path) {
		return Err(actix_web::error::ErrorBadRequest("変更先のパスは既に存在します").into());
	}

	// 2. 既存の設定を取り出し（古いキーからは削除される）
	let mut current_setting = match setting.remove(&current_path) {
		Some(c) => c,
		None => return Err(actix_web::error::ErrorNotFound("指定されたパスが存在しません").into()),
	};

	// 3. 物理ファイルの削除 (rem_files)
	for rem_uuid in info.rem_files {
		let key = rem_uuid.into_inner();
		let _ = tokio::fs::remove_file(format!("upload/{}", key)).await; // 失敗しても進行させる
	}

	// 4. 新規ファイルの保存 (add_files)
	let mut added_files = Vec::new();
	for file in info.add_files {
		added_files.push(save_file(file)?);
	}

	// 5. Configの更新 (送信された場合のみ上書き)
	if let Some(config) = info.config {
		let config = config.into_inner();

		// 注意: ここで新しくResourceを作り直す場合、既存のファイル(UUID)も維持したいなら、
		// FormConfig 側に既存の UUID を受け取るためのフィールド（`existing_keys: Vec<Uuid>` 等）を追加し、
		// build_resource 内で added_files と結合するロジックが必要になります。
		// 今回は単純に add_files から新しい Resource を構築する想定としています。
		let new_resource = build_resource(config, added_files)?;

		// デフォルトのFetchDest(None)の設定を上書き
		current_setting.insert(info.fetch_dest.map(|x| x.into_inner()), new_resource);
	}

	// 6. 新しい（または同じ）パス名で再登録
	setting.insert(target_path, current_setting);

	Ok(HttpResponse::Ok().finish())
}

async fn path_delete(path: web::Path<String>) -> common::Result<impl Responder> {
	Ok("TODO")
}

// ヘルパー
/// アップロードされたファイルを保存し、その情報を返す
fn save_file(file: TempFile) -> actix_web::Result<(Uuid, mime::Mime)> {
	let key = Uuid::new_v4();
	let mime = file.content_type.clone().unwrap_or(mime::TEXT_PLAIN);

	file.file.persist(format!("upload/{}", key)).map_err(|e| actix_web::error::ErrorInternalServerError(format!("File save error: {}", e)))?;

	Ok((key, mime))
}

/// FormConfigと保存済みファイルの情報からResourceを構築する
fn build_resource(config: FormConfig, mut saved_files: Vec<(Uuid, mime::Mime)>) -> actix_web::Result<util::Resource> {
	match config {
		FormConfig::File => {
			if saved_files.len() != 1 {
				return Err(actix_web::error::ErrorBadRequest("1 file required").into());
			}
			let (key, mime) = saved_files.pop().unwrap();
			Ok(util::Resource::File { key, mime })
		}
		FormConfig::RandomFile { weights, cache } => {
			if saved_files.len() != weights.len() {
				return Err(actix_web::error::ErrorBadRequest("Weights length mismatch").into());
			}
			let items = saved_files.into_iter().zip(weights).map(|((key, mime), weight)| (key, mime, weight)).collect();
			Ok(util::Resource::RandomFile { items, cache })
		}
		FormConfig::FontRender { width, height, line_length } => {
			if saved_files.len() != 1 {
				return Err(actix_web::error::ErrorBadRequest("1 file required").into());
			}
			let (key, _) = saved_files.pop().unwrap();
			Ok(util::Resource::FontRender { key, width, height, line_length })
		}
	}
}
