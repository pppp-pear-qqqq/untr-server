use super::*;
use crate::util::{self, RouteMap, UserMap};

use actix_multipart::form::{self, MultipartForm, tempfile::TempFile};
use actix_web::mime;

pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.service(web::resource("").get(index).post(post));
	cfg.service(web::resource("{path:.+}").post(path_post).put(path_put).patch(path_patch).delete(path_delete));
}

async fn index() -> common::Result<impl Responder> {
	Ok("TODO")
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
struct Post {
	files: Vec<TempFile>,
}
/// ファイルの一括アップロード・リソース設定
async fn post(MultipartForm(info): MultipartForm<Post>, id: Identity, setting: web::Data<RouteMap>, users: web::Data<UserMap>) -> common::Result<impl Responder> {
	let username = users.get_or_load(&*id).await?;

	let mut setting = setting.entry(username).or_default();

	let mut result = String::new();
	let mut saved = Vec::new();
	for file in info.files {
		if file.file_name.is_none() {
			result.push_str("[ERROR] ファイル名を読み込めません\n");
			continue;
		}

		let name = file.file_name.as_ref().cloned().unwrap();
		if setting.contains_key(&name) {
			result.push_str(&format!("[ERROR] {}: 既に同名のパスが設定されています\n", name));
			continue;
		}

		// ファイルの保存処理
		let (key, mime) = save_file(file)?;
		saved.push((name, key, mime));
	}

	// 1つもファイルが保存されなかった場合のエラーハンドリング
	if saved.is_empty() {
		return Err(ErrorBadRequest("ファイルがありません").into());
	}

	for (name, key, mime) in saved {
		// デフォルトの設定作成
		let mut value = util::Setting::default();
		value.insert(None, util::Resource::File { key, mime });

		// 更新
		setting.insert(name, value);
	}

	Ok(HttpResponse::Ok().finish())
}

#[derive(MultipartForm)]
struct NewPath {
	fetch_dest: Option<form::text::Text<util::FetchDest>>,
	files: Vec<TempFile>,
	config: form::text::Text<FormConfig>,
}
/// リソースの新規作成
async fn path_post(path: web::Path<String>, MultipartForm(info): MultipartForm<NewPath>, id: Identity, setting: web::Data<RouteMap>, users: web::Data<UserMap>) -> common::Result<impl Responder> {
	let path = path.into_inner();
	let username = users.get_or_load(&*id).await?;
	let fetch_dest = info.fetch_dest.map(|x| x.into_inner());

	let mut setting = setting.entry(username).or_default();
	let setting = setting.entry(path).or_default();
	if setting.contains_key(&fetch_dest) {
		return Err(ErrorBadRequest("既に同名・同条件のパスが設定されています").into());
	}

	let mut saved = Vec::new();
	for file in info.files {
		saved.push(save_file(file)?);
	}

	let resource = build_resource(info.config.into_inner(), saved)?;

	setting.insert(fetch_dest, resource);

	Ok(HttpResponse::Ok().finish())
}

/// リソースの置き換え 実装としては既存条件があるときにエラーではなく上書きするPOST
async fn path_put(path: web::Path<String>, MultipartForm(info): MultipartForm<NewPath>, id: Identity, setting: web::Data<RouteMap>, users: web::Data<UserMap>) -> common::Result<impl Responder> {
	let path = path.into_inner();
	let username = users.get_or_load(&*id).await?;
	let fetch_dest = info.fetch_dest.map(|x| x.into_inner());

	let mut setting = setting.entry(username).or_default();
	let setting = setting.entry(path).or_default();
	if let Some(cur) = setting.remove(&fetch_dest) {
		todo!("curを削除（リソース解放）");
	}

	let mut saved = Vec::new();
	for file in info.files {
		saved.push(save_file(file)?);
	}

	let resource = build_resource(info.config.into_inner(), saved)?;

	setting.insert(fetch_dest, resource);

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
/// 既存リソースの設定変更
// new_pathを指定した場合はfetch_destより上位を対象とするのに対して、add_filesやrem_files,configはfetch_dest以下と役割が異なるので分離すべきか
async fn path_patch(path: web::Path<String>, MultipartForm(info): MultipartForm<PathPatch>, id: Identity, setting: web::Data<RouteMap>, users: web::Data<UserMap>) -> common::Result<impl Responder> {
	let mut path = path.into_inner();
	let username = users.get_or_load(&*id).await?;

	let mut setting = setting.entry(username).or_default();

	// # パス名変更
	if let Some(new_path) = info.new_path {
		let new_path = new_path.into_inner();
		if path != new_path && setting.contains_key(&new_path) {
			return Err(ErrorBadRequest("変更先のパスは既に存在します").into());
		}

		// 既存の設定を取り出し（古いキーからは削除される）
		let cur = setting.remove(&path).ok_or_else(|| ErrorNotFound("指定されたパスが存在しません"))?;

		// 新しいパスで設定を保存
		setting.insert(new_path.clone(), cur);
		path = new_path;
	}

	// # フェッチ先変更
	let fetch_dest = info.fetch_dest.map(|x| x.into_inner());
	let setting = setting.entry(path).or_default();
	if let Some(cur) = setting.get_mut(&fetch_dest) {
		// ## 既存設定の更新
		// 物理ファイルの削除 (rem_files)
		let rem_files = info.rem_files.into_iter().map(|k| k.into_inner()).collect::<Vec<_>>();
		let (rem_files, saved): (Vec<_>, Vec<_>) = cur.key_and_mimes().into_iter().partition(|(key, _)| rem_files.contains(key));
		for key in rem_files.into_iter().map(|(key, _)| key) {
			let _ = tokio::fs::remove_file(format!("upload/{}", key)).await; // 失敗しても進行させる
		}

		let mut saved: Vec<_> = saved.into_iter().map(|(key, mime)| (*key, mime.to_owned())).collect();
		// 新規ファイルの保存 (add_files)
		for file in info.add_files {
			saved.push(save_file(file)?);
		}

		// Configの更新
		if let Some(config) = info.config {
			let config = config.into_inner();

			let resource = build_resource(config, saved)?;

			// 新しい設定を保存
			*cur = resource;
		} else {
			// 既存の設定を更新
			let count = cur.key_count();
			if count != saved.len() {
				return Err(ErrorBadRequest("ファイル数が一致しません").into());
			}
			cur.change_files(&saved);
		}
	} else {
		// ## 新規作成
		if !info.rem_files.is_empty() {
			return Err(ErrorBadRequest("条件を追加する場合はrem_filesは空になります").into());
		}
		let config = info.config.ok_or_else(|| ErrorBadRequest("条件を追加する場合はconfigが必須です"))?.into_inner();
		let mut saved = Vec::new();
		for file in info.add_files {
			saved.push(save_file(file)?);
		}
		let resource = build_resource(config, saved)?;
		setting.insert(fetch_dest, resource);
	}

	Ok(HttpResponse::Ok().finish())
}

async fn path_delete(path: web::Path<String>) -> common::Result<impl Responder> {
	Ok("TODO")
}

// ヘルパー
/// アップロードされたファイルを保存し、その情報を返す
fn save_file(file: TempFile) -> actix_web::Result<(Uuid, mime::Mime)> {
	let key = Uuid::new_v4();
	let mime = file.content_type.unwrap_or(mime::TEXT_PLAIN);

	file.file.persist(resource(format!("upload/{}", key))).map_err(|e| ErrorInternalServerError(format!("File save error: {}", e)))?;

	Ok((key, mime))
}

/// FormConfigと保存済みファイルの情報からResourceを構築する
fn build_resource(config: FormConfig, mut saved_files: Vec<(Uuid, mime::Mime)>) -> actix_web::Result<util::Resource> {
	match config {
		FormConfig::File => {
			if saved_files.len() != 1 {
				return Err(ErrorBadRequest("1 file required").into());
			}
			let (key, mime) = saved_files.pop().unwrap();
			Ok(util::Resource::File { key, mime })
		}
		FormConfig::RandomFile { weights, cache } => {
			if saved_files.len() != weights.len() {
				return Err(ErrorBadRequest("Weights length mismatch").into());
			}
			let items = saved_files.into_iter().zip(weights).map(|((key, mime), weight)| (key, mime, weight)).collect();
			Ok(util::Resource::RandomFile { items, cache })
		}
		FormConfig::FontRender { width, height, line_length } => {
			if saved_files.len() != 1 {
				return Err(ErrorBadRequest("1 file required").into());
			}
			let (key, _) = saved_files.pop().unwrap();
			Ok(util::Resource::FontRender { key, width, height, line_length })
		}
	}
}
