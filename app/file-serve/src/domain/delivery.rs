use std::io::Cursor;

use super::*;
use crate::util::{FetchDest, Resource, RouteMap};

use ab_glyph::{FontRef, PxScale};
use actix_web::HttpRequest;
use base64::prelude::*;
use fxhash::FxHashMap as HashMap;
use image::{Rgba, RgbaImage};
use rand::seq::IndexedRandom;

#[derive(serde::Deserialize)]
pub struct Addr {
	username: String,
	path: String,
}
/// スライスからユーザー名、パス、およびマージ済みのクエリを抽出する
fn parse_addr(slice: &[u8], req_query: HashMap<String, String>) -> Option<(Addr, HashMap<String, String>)> {
	let text = String::from_utf8_lossy(slice);
	let (username, rest) = text.split_once('/')?;
	let (path, query_str) = rest.split_once('?').unwrap_or((rest, ""));

	// パス側のクエリをパース
	let mut query: HashMap<String, String> = query_str
		.split('&')
		.filter(|p| !p.is_empty()) // 空文字(&の連続など)を除外
		.map(|p| {
			let (k, v) = p.split_once('=').unwrap_or((p, ""));
			(k.to_string(), v.to_string())
		})
		.collect();

	// リクエストのクエリ(req_query)を優先して上書きする
	query.extend(req_query);
	Some((
		Addr {
			username: username.to_string(),
			path: path.to_string(),
		},
		query,
	))
}

pub async fn plane(req: HttpRequest, path: web::Path<Addr>, web::Query(query): web::Query<HashMap<String, String>>, dest: Option<FetchDest>, setting: web::Data<RouteMap>) -> common::Result<impl Responder> {
	file(req, path.into_inner(), query, dest, &setting).await
}
pub async fn b64(req: HttpRequest, path: web::Path<String>, web::Query(query): web::Query<HashMap<String, String>>, dest: Option<FetchDest>, setting: web::Data<RouteMap>) -> common::Result<impl Responder> {
	let decoded = BASE64_URL_SAFE.decode(&path.into_inner())?;
	let (addr, query) = parse_addr(&decoded, query).ok_or_else(|| ErrorBadRequest("パスの形式が正しくありません"))?;
	file(req, addr, query, dest, &setting).await
}
pub async fn encrypt(req: HttpRequest, path: web::Path<String>, web::Query(query): web::Query<HashMap<String, String>>, dest: Option<FetchDest>, setting: web::Data<RouteMap>) -> common::Result<impl Responder> {
	let decoded = BASE64_URL_SAFE.decode(&path.into_inner())?; // TODO
	let (addr, query) = parse_addr(&decoded, query).ok_or_else(|| ErrorBadRequest("パスの形式が正しくありません"))?;
	file(req, addr, query, dest, &setting).await
}

async fn file(req: HttpRequest, addr: Addr, query: HashMap<String, String>, dest: Option<FetchDest>, setting: &RouteMap) -> common::Result<HttpResponse> {
	let setting = setting.get(&addr.username).ok_or_else(|| ErrorNotFound("ファイルがありません"))?;
	let setting = setting.get(&addr.path).ok_or_else(|| ErrorNotFound("ファイルがありません"))?;
	let r = dest.and_then(|d| setting.get(&Some(d))).or_else(|| setting.get(&None)).ok_or_else(|| ErrorNotFound("ファイルがありません"))?;
	let (mut res, cache) = match r {
		Resource::File { key, mime } => {
			let f = actix_files::NamedFile::open(resource(format!("upload/{key}")))?;
			(f.set_content_type(mime.clone()).into_response(&req), true)
		}
		Resource::RandomFile { items, cache } => {
			let (key, mime, _) = items.choose_weighted(&mut rand::rng(), |(_, _, w)| *w)?;
			let f = actix_files::NamedFile::open(resource(format!("upload/{key}")))?;
			(f.set_content_type(mime.clone()).into_response(&req), *cache)
		}
		Resource::FontRender { key, width, height, line_length } => {
			// クエリから描画する文字と文字色を取得
			let word = query.get("word").or_else(|| query.get("w")).ok_or_else(|| ErrorBadRequest("クエリパラメータ 'word' が必要です"))?;
			if word.is_empty() {
				return Err(ErrorBadRequest("wordが空です").into());
			}
			let color = query.get("color").or_else(|| query.get("c"));
			let color = color.and_then(|c| parse_color(c).ok()).unwrap_or_else(|| Rgba([0, 0, 0, 255]));

			// フォントファイルの読み込みと検証
			let font_data = tokio::fs::read(resource(format!("upload/{key}"))).await.map_err(|_| ErrorInternalServerError("フォントファイルの読み込みに失敗しました"))?;
			let font = FontRef::try_from_slice(&font_data).map_err(|_| ErrorInternalServerError("無効なフォントファイルです"))?;

			// 自動改行（折り返し）の計算
			let chars: Vec<_> = word.chars().collect();
			let mut chars = chars.into_iter();
			let mut lines = Vec::new();
			let mut line = Vec::new();
			let line_length = *line_length as usize;

			while let Some(ch) = chars.next() {
				if (line_length != 0 && line.len() >= line_length) || ch == '\n' {
					lines.push(line);
					line = Vec::new();
				}
				line.push(ch);
			}
			if !line.is_empty() {
				lines.push(line);
			}

			// 画像サイズの計算
			let cols = lines.iter().map(|l| l.len()).max().unwrap_or(0) as u32;
			let rows = lines.len() as u32;

			// 画像の作成（デフォルトは完全透過）
			let mut img = RgbaImage::new(cols * *width, rows * *height);
			let scale = PxScale { x: *width as f32, y: *height as f32 };

			// 各文字をグリッドに沿って描画
			for (r, line) in lines.iter().enumerate() {
				for (c, &ch) in line.iter().enumerate() {
					let x = (c as u32 * *width) as i32;
					let y = (r as u32 * *height) as i32;

					imageproc::drawing::draw_text_mut(&mut img, color, x, y, scale, &font, &ch.to_string());
				}
			}

			// メモリ上でWEBP形式にエンコード
			let mut buffer = Cursor::new(Vec::new());
			img.write_to(&mut buffer, image::ImageFormat::WebP).map_err(|e| ErrorInternalServerError(format!("画像の生成に失敗しました: {}", e)))?;

			// HttpResponseを構築して返す
			(HttpResponse::Ok().content_type("image/webp").body(buffer.into_inner()), true)
		}
	};

	let headers = res.headers_mut();
	if cache {
		headers.insert(header::CACHE_CONTROL, header::HeaderValue::from_static("public,max-age=604800"));
	} else {
		headers.insert(header::CACHE_CONTROL, header::HeaderValue::from_static("no-store,no-cache,must-revalidate,max-age=0"));
	}
	Ok(res)
}

/// 柔軟な16進数カラーコードのパース
fn parse_color(hex: &str) -> Result<Rgba<u8>, ()> {
	let hex = hex.trim_start_matches('#');
	let parse_hex = |s: &str| u8::from_str_radix(s, 16).map_err(|_| ());

	match hex.len() {
		3 => {
			// #RGB
			let r = parse_hex(&hex[0..1])? * 17;
			let g = parse_hex(&hex[1..2])? * 17;
			let b = parse_hex(&hex[2..3])? * 17;
			Ok(Rgba([r, g, b, 255]))
		}
		4 => {
			// #RGBA
			let r = parse_hex(&hex[0..1])? * 17;
			let g = parse_hex(&hex[1..2])? * 17;
			let b = parse_hex(&hex[2..3])? * 17;
			let a = parse_hex(&hex[3..4])? * 17;
			Ok(Rgba([r, g, b, a]))
		}
		6 => {
			// #RRGGBB
			let r = parse_hex(&hex[0..2])?;
			let g = parse_hex(&hex[2..4])?;
			let b = parse_hex(&hex[4..6])?;
			Ok(Rgba([r, g, b, 255]))
		}
		8 => {
			// #RRGGBBAA
			let r = parse_hex(&hex[0..2])?;
			let g = parse_hex(&hex[2..4])?;
			let b = parse_hex(&hex[4..6])?;
			let a = parse_hex(&hex[6..8])?;
			Ok(Rgba([r, g, b, a]))
		}
		// 解析不能な場合はデフォルト（黒・完全不透明）
		_ => Err(()),
	}
}
