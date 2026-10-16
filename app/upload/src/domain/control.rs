use super::*;
use crate::util::{self, Resource, ResourceConfig, RouteMap, UserMap};

use actix_multipart::form::{self, MultipartForm, tempfile::TempFile};
use actix_web::{HttpRequest, mime};
use common::ReqType;

pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.service(web::resource("").get(index).post(post));
	cfg.service(web::resource("{path:.+}").post(path_post).put(path_put).patch(path_patch).delete(path_delete));
}

// ==========================================
// ハンドラ群
// ==========================================

async fn index(req: HttpRequest, ty: ReqType, id: Identity, setting: web::Data<RouteMap>, users: web::Data<UserMap>) -> common::Result<impl Responder> {
	match ty {
		ReqType::Empty => {
			let username = users.get(&*id).ok_or_else(|| ErrorUnauthorized("ログインしてください"))?;
			if let Some(setting) = setting.get(&username) {
				let keys = setting.keys().collect::<Vec<_>>();
				Ok(HttpResponse::Ok().json(keys))
			} else {
				Ok(HttpResponse::Ok().body("[]"))
			}
		}
		_ => {
			// TODO 多分登録済みのデータとか諸々をテンプレートに入れなきゃいけない
			let file = actix_files::NamedFile::open(resource("control.html"))?.set_content_type(mime::TEXT_HTML_UTF_8);
			Ok(file.into_response(&req))
		}
	}
}

#[derive(MultipartForm)]
struct Post {
	files: Vec<TempFile>,
}
/// ファイルの一括アップロード・リソース設定
async fn post(MultipartForm(info): MultipartForm<Post>, id: Identity, state: StateHandle, setting: web::Data<RouteMap>, users: web::Data<UserMap>) -> common::Result<impl Responder> {
	state.get().only_active()?;
	debug!("{setting:?}");
	let username = users.get(&*id).ok_or_else(|| ErrorUnauthorized("ログインしてください"))?;
	let mut setting = setting.entry(username).or_default();

	let mut result = String::new();
	let mut prepared_files = Vec::new();
	let mut resources_to_insert = Vec::new();

	// バリデーションとファイルの準備（まだ保存しない）
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

		let prepared = PreparedFile::new(file);

		let mut value = util::Setting::default();
		value.insert(None, util::Resource::File { key: prepared.key, mime: prepared.mime.clone() });

		resources_to_insert.push((name, value));
		prepared_files.push(prepared);
	}

	if prepared_files.is_empty() {
		return Err(ErrorBadRequest("保存できるファイルがありません").into());
	}

	// ディスクへの一括保存（失敗時は自動ロールバック）
	commit_files(prepared_files).await?;

	// 全て成功したらマップを更新
	let mut res = Vec::new();
	for (name, value) in resources_to_insert {
		res.push(name.clone());
		setting.insert(name, value);
	}

	Ok(HttpResponse::Ok().json(res))
}

#[derive(MultipartForm)]
struct NewPath {
	fetch_dest: Option<form::text::Text<util::FetchDest>>,
	files: Vec<TempFile>,
	config: form::text::Text<ResourceConfig>,
}
/// リソースの新規作成
async fn path_post(path: web::Path<String>, MultipartForm(info): MultipartForm<NewPath>, id: Identity, state: StateHandle, setting: web::Data<RouteMap>, users: web::Data<UserMap>) -> common::Result<impl Responder> {
	state.get().only_active()?;
	let username = users.get(&*id).ok_or_else(|| ErrorUnauthorized("ログインしてください"))?;
	let value = path_set(username, path.into_inner(), info, setting, false).await?;
	Ok(HttpResponse::Ok().json(value))
}
/// リソースの置き換え
async fn path_put(path: web::Path<String>, MultipartForm(info): MultipartForm<NewPath>, id: Identity, state: StateHandle, setting: web::Data<RouteMap>, users: web::Data<UserMap>) -> common::Result<impl Responder> {
	state.get().only_active()?;
	let username = users.get(&*id).ok_or_else(|| ErrorUnauthorized("ログインしてください"))?;
	let value = path_set(username, path.into_inner(), info, setting, true).await?;
	Ok(HttpResponse::Ok().json(value))
}
/// リソース設定
async fn path_set(username: String, path: String, info: NewPath, setting: web::Data<RouteMap>, rewrite: bool) -> actix_web::Result<Vec<Uuid>> {
	let fetch_dest = info.fetch_dest.map(|x| x.into_inner());

	let mut setting = setting.entry(username).or_default();
	let setting = setting.entry(path).or_default();

	// 上書き不可かつ既に存在する場合は早期リターン
	if !rewrite && setting.contains_key(&fetch_dest) {
		return Err(ErrorBadRequest("既に同名・同条件のパスが設定されています").into());
	}

	// ファイルの準備
	let prepared_files: Vec<_> = info.files.into_iter().map(PreparedFile::new).collect();
	let file_infos: Vec<_> = prepared_files.iter().map(|p| (p.key, p.mime.clone())).collect();

	// リソースの構築とバリデーション
	let resource = Resource::build(info.config.into_inner(), file_infos)?;

	// 返却用のUUIDリストを抽出
	let saved_keys: Vec<Uuid> = resource.key_and_mimes().into_iter().map(|(key, _)| *key).collect();

	// 物理ファイルの保存
	commit_files(prepared_files).await?;

	// 設定の上書き ＆ 古いファイルのクリーンアップ
	if let Some(old) = setting.insert(fetch_dest, resource) {
		let old_keys = old.key_and_mimes().into_iter().map(|(key, _)| *key).collect();
		remove_files(old_keys).await;
	}

	Ok(saved_keys)
}

#[derive(MultipartForm)]
struct PathPatch {
	new_path: Option<form::text::Text<String>>,
	fetch_dest: Option<form::text::Text<util::FetchDest>>,
	add_files: Vec<TempFile>,
	rem_files: Vec<form::text::Text<Uuid>>,
	config: Option<form::text::Text<ResourceConfig>>,
}
/// 既存リソースの設定変更
async fn path_patch(path: web::Path<String>, MultipartForm(info): MultipartForm<PathPatch>, id: Identity, state: StateHandle, setting: web::Data<RouteMap>, users: web::Data<UserMap>) -> common::Result<impl Responder> {
	state.get().only_active()?;
	let mut path = path.into_inner();
	let username = users.get(&*id).ok_or_else(|| ErrorUnauthorized("ログインしてください"))?;

	let mut setting = setting.entry(username).or_default();

	// # パス名変更
	if let Some(new_path) = info.new_path {
		let new_path = new_path.into_inner();
		if path != new_path && setting.contains_key(&new_path) {
			return Err(ErrorBadRequest("変更先のパスは既に存在します").into());
		}

		let cur = setting.remove(&path).ok_or_else(|| ErrorNotFound("指定されたパスが存在しません"))?;
		setting.insert(new_path.clone(), cur);
		path = new_path;
	}

	// # フェッチ先変更
	let fetch_dest = info.fetch_dest.map(|x| x.into_inner());
	let setting = setting.entry(path).or_default();

	if let Some(cur) = setting.get_mut(&fetch_dest) {
		// ## 既存設定の更新
		let rem_files: Vec<Uuid> = info.rem_files.into_iter().map(|k| k.into_inner()).collect();
		let (rem_targets, saved_infos): (Vec<_>, Vec<_>) = cur.key_and_mimes().into_iter().partition(|(key, _)| rem_files.contains(key));

		// 存続するファイルの情報をクローンしておく
		let mut final_file_infos: Vec<(Uuid, mime::Mime)> = saved_infos.into_iter().map(|(k, m)| (*k, m.clone())).collect();

		// 追加ファイルの準備
		let mut prepared_files = Vec::new();
		for file in info.add_files {
			let prepared = PreparedFile::new(file);
			final_file_infos.push((prepared.key, prepared.mime.clone()));
			prepared_files.push(prepared);
		}

		if let Some(config) = info.config {
			let config = config.into_inner();
			let resource = Resource::build(config, final_file_infos)?; // バリデーション

			commit_files(prepared_files).await?; // 保存

			// 物理ファイルの削除 (不要になったもの)
			remove_files(rem_targets.into_iter().map(|(key, _)| *key).collect()).await;

			*cur = resource; // インプレース上書き
		} else {
			let count = cur.key_count();
			if count != final_file_infos.len() {
				return Err(ErrorBadRequest("ファイル数が一致しません").into());
			}

			commit_files(prepared_files).await?; // 保存

			// 物理ファイルの削除 (不要になったもの)
			remove_files(rem_targets.into_iter().map(|(key, _)| *key).collect()).await;

			cur.change_files(&final_file_infos); // メタデータを維持して差し替え
		}
	} else {
		// ## 新規作成
		if !info.rem_files.is_empty() {
			return Err(ErrorBadRequest("条件を追加する場合はrem_filesは空になります").into());
		}
		let config = info.config.ok_or_else(|| ErrorBadRequest("条件を追加する場合はconfigが必須です"))?.into_inner();

		let mut prepared_files = Vec::new();
		let mut file_infos = Vec::new();
		for file in info.add_files {
			let prepared = PreparedFile::new(file);
			file_infos.push((prepared.key, prepared.mime.clone()));
			prepared_files.push(prepared);
		}

		let resource = Resource::build(config, file_infos)?;
		commit_files(prepared_files).await?;

		setting.insert(fetch_dest, resource);
	}

	Ok(HttpResponse::Ok().finish())
}

/// 指定パスのリソースをすべて削除
async fn path_delete(path: web::Path<String>, id: Identity, state: StateHandle, setting: web::Data<RouteMap>, users: web::Data<UserMap>) -> common::Result<impl Responder> {
	state.get().only_active()?;
	let path = path.into_inner();
	let username = users.get(&*id).ok_or_else(|| ErrorUnauthorized("ログインしてください"))?;

	let mut setting = setting.entry(username).or_default();

	// パスごと一括削除し、ぶら下がっていた全ファイルを消去
	if let Some(removed) = setting.remove(&path) {
		for (_, resource) in removed {
			remove_files(resource.key_and_mimes().into_iter().map(|(key, _)| *key).collect()).await;
		}
		Ok(HttpResponse::Ok().finish())
	} else {
		Err(ErrorNotFound("指定されたパスが存在しません").into())
	}
}

// ==========================================
// ヘルパー群
// ==========================================

/// 保存待ち状態のファイル情報
struct PreparedFile {
	key: Uuid,
	mime: mime::Mime,
	temp: TempFile,
}
impl PreparedFile {
	fn new(temp: TempFile) -> Self {
		let key = Uuid::new_v4();
		let mime = temp.content_type.clone().unwrap_or(mime::TEXT_PLAIN);
		Self { key, mime, temp }
	}
}

/// 準備されたファイル群を一括で保存し、エラーがあれば自動で削除（ロールバック）する
async fn commit_files(prepared_files: Vec<PreparedFile>) -> actix_web::Result<()> {
	let mut successfully_saved = Vec::new();

	for pf in prepared_files {
		let dest = resource(format!("upload/{}", pf.key)); // 既存のヘルパー関数を使用
		match pf.temp.file.persist(&dest) {
			Ok(_) => {
				successfully_saved.push(pf.key);
			}
			Err(e) => {
				// ロールバック: 既に成功したファイルのみ削除する
				remove_files(successfully_saved.into_iter().collect()).await;
				return Err(ErrorInternalServerError(format!("File save error: {}", e)).into());
			}
		}
	}
	Ok(())
}

/// ファイルを削除する
async fn remove_files(keys: Vec<Uuid>) {
	for key in keys {
		let _ = tokio::fs::remove_file(format!("upload/{}", key)).await;
	}
}
