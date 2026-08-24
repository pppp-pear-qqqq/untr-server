use std::{collections::HashMap, sync::OnceLock};

use regex::Regex;

pub trait TagFormat {
	const START: u8 = b'[';

	fn from_args(args: &HashMap<String, tera::Value>) -> Self;

	fn parse<'a>(&self, text: &'a str) -> Option<Tag<'a>> {
		if text.as_bytes().first() != Some(&Self::START) {
			return None;
		}

		// 最初の '/' を探してタグ名(name)を取得
		let first_slash = text.find('/')?;
		let name = &text[1..first_slash];

		// 中身(body)以降のスライスを取得
		let body_start = first_slash + 1;
		let rest = &text[body_start..];

		// 終了マーカー "/name]" をアロケーション無しで探す
		let mut search_offset = 0;
		let mut end_pos = 0;
		let mut found = false;

		// `rest` の中から '/' を順番に探す
		while let Some(idx) = rest[search_offset..].find('/') {
			let pos = search_offset + idx;
			let remaining = rest[pos..].as_bytes();

			// remainingが "/name]" を含む十分な長さを持っているか
			if remaining.len() > name.len() + 1 {
				let end_idx = 1 + name.len();
				// '/' の次からが name と一致し、その直後が ']' かどうかを直接判定
				if &remaining[1..end_idx] == name.as_bytes() && remaining[end_idx] == b']' {
					end_pos = pos;
					found = true;
					break;
				}
			}
			search_offset = pos + 1; // 見つからなかったら次の '/' へ
		}

		if !found {
			return None;
		}

		Some(Tag {
			name,
			content: &rest[..end_pos],
			size: body_start + end_pos + name.len() + 2,
		})
	}

	fn format(&self, tag: Tag<'_>, link: bool) -> String;
}
#[allow(dead_code)]
pub struct Tag<'a> {
	pub name: &'a str,
	pub content: &'a str,
	pub size: usize,
}

pub trait HTMLEncode {
	fn to_html<T: TagFormat>(&self, tag_format: &T, link: bool) -> String;
}

impl HTMLEncode for str {
	fn to_html<T: TagFormat>(&self, tag_format: &T, link: bool) -> String {
		let bytes = self.as_bytes();
		let mut output = String::with_capacity(self.len() + 32);

		let mut i = 0;
		let mut p = 0; // 最後に処理したチャンクの終わり位置

		while i < bytes.len() {
			if bytes[i] == T::START {
				if let Some(tag) = tag_format.parse(&self[i..]) {
					// パース成功：これまでのプレーンテキストをリンク・エスケープ処理に回す
					if link {
						process_plain_text(&self[p..i], &mut output);
					} else {
						escape_and_br(&self[p..i], &mut output);
					}

					// インデックスをタグの終わりまで進める
					i += tag.size;
					p = i;

					// フォーマットされた独自タグHTMLを追加
					output.push_str(&tag_format.format(tag, link));

					continue;
				}
			}
			i += 1;
		}

		// 最後に残ったプレーンテキストを処理
		if p < self.len() {
			if link {
				process_plain_text(&self[p..], &mut output);
			} else {
				escape_and_br(&self[p..], &mut output);
			}
		}
		output
	}
}

static LINK_RE: OnceLock<Regex> = OnceLock::new();
fn link_re() -> &'static Regex {
	LINK_RE.get_or_init(|| {
		let regs = [r#"(?<url>https?://[^\s<>"']+)"#, r"(?<misskey>@[\w_\-]+@[\w_\-]+(?:\.[\w_\-]+)+)", r"(?<bsky>@[\w_\-]+(?:\.[\w_\-]+)+)", r"(?<twitter>@[\w_\-]{4,15})"];
		Regex::new(&format!(r"(^|\s)(?:{})", regs.join("|"))).unwrap()
	})
}

/// テキストチャンクを受け取り、リンク変換とエスケープを行ってoutputに追加する
fn process_plain_text(chunk: &str, output: &mut String) {
	let mut end = 0;
	for caps in link_re().captures_iter(chunk) {
		let m = caps.get(0).unwrap();

		// リンクより前のテキストをエスケープ＆改行変換して追加
		escape_and_br(&chunk[end..m.start()], output);

		// (^|\s) でキャプチャされた先頭の空白部分を保持
		if let Some(space) = caps.get(1) {
			escape_and_br(space.as_str(), output);
		}

		end = m.end();

		// URLタグの生成
		let (href, body) = if let Some(m) = caps.name("url") {
			let m_str = m.as_str();
			(m_str.replace('"', "%22"), escape_str(m_str))
		} else if let Some(m) = caps.name("misskey") {
			let m_str = m.as_str();
			let (user, domain) = m_str.rsplit_once('@').unwrap();
			(format!("https://{domain}/{user}"), escape_str(m_str))
		} else if let Some(m) = caps.name("bsky") {
			let m_str = m.as_str();
			(format!("https://bsky.app/profile/{}", &m_str[1..]), escape_str(m_str))
		} else if let Some(m) = caps.name("twitter") {
			let m_str = m.as_str();
			(format!("https://x.com/{}", &m_str[1..]), escape_str(m_str))
		} else {
			unreachable!();
		};

		output.push_str(&format!("<a target=\"_blank\" href=\"{href}\">{body}</a>"));
	}

	// 最後に残ったテキストを処理
	if end < chunk.len() {
		escape_and_br(&chunk[end..], output);
	}
}
/// エスケープと改行(<br>)の処理を行う
fn escape_and_br(text: &str, output: &mut String) {
	const BR: &str = "<br>";
	let repl = |b: u8| match b {
		b'"' => Some("&quot;"),
		b'&' => Some("&amp;"),
		b'\'' => Some("&apos;"),
		b'<' => Some("&lt;"),
		b'>' => Some("&gt;"),
		b'/' => Some("&frasl;"),
		_ => None,
	};
	let bytes = text.as_bytes();

	let mut p = 0;
	let mut i = 0;
	while i < bytes.len() {
		if bytes[i] == b'\n' {
			output.push_str(&text[p..i]);
			output.push_str(BR);
			i += 1;
			p = i;
		} else if bytes[i] == b'\r' {
			output.push_str(&text[p..i]);
			output.push_str(BR);
			i += 1;
			if i < bytes.len() && bytes[i] == b'\n' {
				i += 1;
			}
			p = i;
		} else if let Some(r) = repl(bytes[i]) {
			output.push_str(&text[p..i]);
			output.push_str(r);
			i += 1;
			p = i;
		} else {
			i += 1;
		}
	}
	if p < text.len() {
		output.push_str(&text[p..]);
	}
}
/// 単なる文字列（エスケープ用）を返すヘルパー
fn escape_str(text: &str) -> String {
	let mut out = String::with_capacity(text.len());
	escape_and_br(text, &mut out);
	out
}
