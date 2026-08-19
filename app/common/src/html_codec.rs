use std::{borrow::Cow, sync::OnceLock};

use regex::Regex;

pub trait HTMLEncode<'a> {
	fn parse<T: TagFormat>(self, format: T, link_to_tag: bool) -> Cow<'a, str>;

	fn br(self) -> Cow<'a, str>;
	fn escape(self, quot: bool) -> Cow<'a, str>;
	fn escape_and_link(self) -> Cow<'a, str>;
	fn tag<T: TagFormat>(self, format: T) -> Cow<'a, str>;
}
impl<'a, T: Into<Cow<'a, str>>> HTMLEncode<'a> for T {
	fn parse<F: TagFormat>(self, format: F, link_to_tag: bool) -> Cow<'a, str> {
		let text = self.into();
		// ここでエスケープ・タグ処理・リンク処理など全てまとめて実行する
		todo!()
	}

	fn br(self) -> Cow<'a, str> {
		let text = self.into();
		const BR: &str = "<br>";

		let bytes = text.as_bytes();
		for i in 0..bytes.len() {
			if bytes[i] == b'\n' || bytes[i] == b'\r' {
				let mut owned = String::with_capacity(text.len() + 32);
				owned.push_str(&text[..i]);
				let mut p = i;
				while p < bytes.len() {
					match bytes[p] {
						b'\n' => {
							owned.push_str(BR);
							p += 1;
						}
						b'\r' => {
							owned.push_str(BR);
							p += 1;
							if p < bytes.len() && bytes[p] == b'\n' {
								p += 1;
							}
						}
						_ => {
							let start = p;
							while p < bytes.len() && bytes[p] != b'\r' && bytes[p] != b'\n' {
								p += 1;
							}
							owned.push_str(&text[start..p]);
						}
					}
				}
				owned.push_str(&text[p..]);
				return Cow::Owned(owned);
			}
		}
		text
	}

	fn escape(self, quot: bool) -> Cow<'a, str> {
		let text = self.into();
		let repl = |b: u8| match b {
			b'<' => Some("&lt;"),
			b'>' => Some("&gt;"),
			b'&' => Some("&amp;"),
			b'"' if quot => Some("&quot;"),
			b'\'' if quot => Some("&apos;"),
			_ => None,
		};

		let bytes = text.as_bytes();
		for i in 0..bytes.len() {
			if let Some(r) = repl(bytes[i]) {
				let mut owned = String::with_capacity(text.len() + 32);
				owned.push_str(&text[..i]);
				owned.push_str(r);
				let mut p = i + 1;
				for j in p..bytes.len() {
					if let Some(r) = repl(bytes[j]) {
						owned.push_str(&text[p..j]);
						owned.push_str(r);
						p = j + 1;
					}
				}
				owned.push_str(&text[p..]);
				return Cow::Owned(owned);
			}
		}
		text
	}

	fn escape_and_link(self) -> Cow<'a, str> {
		let text = self.into();
		static RE: OnceLock<Regex> = OnceLock::new();
		let re = RE.get_or_init(|| {
			let regs = [r#"(?<url>https?://[^\s<>"']+)"#, r"(?<misskey>@[\w_\-]+@[\w_\-]+(?:\.[\w_\-]+)+)", r"(?<bsky>@[\w_\-]+(?:\.[\w_\-]+)+)", r"(?<twitter>@[\w_\-]{4,15})"];
			Regex::new(&format!(r"(^|\s)(?:{})", regs.join("|"))).unwrap()
		});

		// Regexでのマッチがなければ、単純に escape して返す（アロケーション回避）
		if !re.is_match(&text) {
			return text.escape(false);
		}

		let mut out = String::with_capacity(text.len() * 2);
		let mut end = 0;
		for caps in re.captures_iter(&text) {
			let m = caps.get(0).unwrap();
			out.push_str(&text[end..m.start()].escape(false));

			// `(^|\s)` でキャプチャされる可能性のある先頭の空白部分を保持
			if let Some(space) = caps.get(1) {
				out.push_str(space.as_str());
			}

			end = m.end();
			let (href, body) = if let Some(m) = caps.name("url") {
				let m_str = m.as_str();
				(m_str.replace('"', "%22"), m_str.escape(false))
			} else if let Some(m) = caps.name("misskey") {
				let m_str = m.as_str();
				let (user, domain) = m_str.rsplit_once('@').unwrap();
				(format!("https://{domain}/{user}"), m_str.escape(false))
			} else if let Some(m) = caps.name("bsky") {
				let m_str = m.as_str();
				(format!("https://bsky.app/profile/{}", &m_str[1..]), m_str.escape(false))
			} else if let Some(m) = caps.name("twitter") {
				let m_str = m.as_str();
				(format!("https://x.com/{}", &m_str[1..]), m_str.escape(false))
			} else {
				unreachable!();
			};
			out.push_str(&format!("<a target=\"_blank\" href=\"{href}\">{body}</a>"));
		}

		if end > 0 {
			out.push_str(&text[end..].escape(false));
			Cow::Owned(out)
		} else {
			text.escape(false)
		}
	}

	fn tag<F: TagFormat>(self, format: F) -> Cow<'a, str> {
		match self.into() {
			Cow::Borrowed(s) => format.parse(s),
			Cow::Owned(s) => Cow::Owned(format.parse(&s).into_owned()),
		}
	}
}

pub trait HTMLDecode<'a> {
	fn unescape(self) -> Cow<'a, str>;
	fn rm_br(self) -> Cow<'a, str>;
}
impl<'a, T: Into<Cow<'a, str>>> HTMLDecode<'a> for T {
	fn unescape(self) -> Cow<'a, str> {
		let text = self.into();
		static SPECIALS: [(&str, char); 7] = [("&lt;", '<'), ("&gt;", '>'), ("&amp;", '&'), ("&quot;", '"'), ("&apos;", '\''), ("&#39;", '\''), ("&nbsp;", ' ')];

		if !text.contains('&') {
			return text;
		}

		let mut result = String::with_capacity(text.len());
		let mut i = 0;
		let s_bytes = text.as_bytes();
		while i < text.len() {
			if s_bytes[i] == b'&' {
				let rest = &text[i..];
				let mut b: Option<(char, usize)> = None;
				for (ent, repl) in SPECIALS {
					if rest.starts_with(ent) {
						b = Some((repl, ent.len()));
						break;
					}
				}
				if let Some((repl, skip)) = b {
					result.push(repl);
					i += skip;
				} else {
					result.push('&');
					i += 1;
				}
			} else {
				let next_amp = text[i..].find('&').unwrap_or(text.len() - i);
				result.push_str(&text[i..i + next_amp]);
				i += next_amp;
			}
		}
		Cow::Owned(result)
	}

	fn rm_br(self) -> Cow<'a, str> {
		let text = self.into();
		if text.contains('\n') || text.contains('\r') { Cow::Owned(text.replace(&['\n', '\r'][..], "")) } else { text }
	}
}

/// # Example
/// ```
/// #[derive(Clone, Copy)]
/// pub struct CommonTag;
/// impl TagFormat for CommonTag {
/// 	fn parse(self, raw: &str) -> Cow<'_, str> {
/// 		fn part(value: &str, limit: usize) -> Vec<&str> {
/// 			let mut parts = Vec::new();
/// 			let mut nest: usize = 0;
/// 			let mut bytes = value.bytes().enumerate().peekable();
/// 			while let Some((idx, b)) = bytes.next() {
/// 				match b {
/// 					b'[' => nest += 1,
/// 					b']' if nest > 0 => nest -= 1,
/// 					b'|' if nest == 0 => {
/// 						parts.push(idx);
/// 						if limit != 0 && limit <= parts.len() {
/// 							break;
/// 						}
/// 					}
/// 					b'\\' => {
/// 						bytes.next_if(|(_, b)| matches!(b, b'[' | b']' | b'|' | b'\\'));
/// 					}
/// 					_ => (),
/// 				}
/// 			}
/// 			let mut start = 0;
/// 			let mut params = Vec::with_capacity(parts.len() + 1);
/// 			for end in parts {
/// 				params.push(&value[start..end]);
/// 				start = end + 1;
/// 			}
/// 			params.push(&value[start..]);
/// 			params
/// 		}
/// 		let mut out = String::with_capacity(raw.len() * 2);
/// 		let mut rng = rand::rng();
/// 		let mut end = 0;
/// 		let mut stack = Vec::with_capacity(1);
/// 		let mut bytes = raw.bytes().enumerate().peekable();
/// 		while let Some((idx, b)) = bytes.next() {
/// 			match b {
/// 				b'[' => {
/// 					let start = idx + 1;
/// 					// ネストが無い時にはそれまでを出力
/// 					if stack.is_empty() {
/// 						out.push_str(&raw[end..idx]);
/// 						end = start;
/// 					}
/// 					// タグ名取得
/// 					if let Some(p) = raw[start..].find('/') {
/// 						let tag = &raw[start..start + p];
/// 						// タグかどうか
/// 						if tag.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
/// 							stack.push((tag, start + p + 1));
/// 							continue;
/// 						}
/// 					}
/// 					// 無名タグ
/// 					stack.push(("", start));
/// 				}
/// 				b']' if !stack.is_empty() => {
/// 					// タグ名取得
/// 					let (p, tag) = raw[end..idx]
/// 						.rfind('/')
/// 						.and_then(|p| {
/// 							let tag = &raw[end + p + 1..idx];
/// 							tag.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_').then_some((end + p, tag))
/// 						})
/// 						.unwrap_or((idx, ""));
/// 					// 現在のスタック先頭と一致するか
/// 					if let Some((_, start)) = stack.pop_if(|(x, _)| *x == tag) {
/// 						// スタックが空なら反映
/// 						if stack.is_empty() {
/// 							let content = &raw[start..p];
/// 							let content = match tag {
/// 								"b" | "i" | "u" | "s" | "large" | "small" | "rainbow" => format!("<{0}>{1}</{0}>", tag, self.parse(content)),
/// 								"ruby" => {
/// 									let params = part(content, 1);
/// 									match params.len() {
/// 										2 => format!("<ruby>{}<rp>(</rp><rt>{}</rt><rp>)</rp></ruby>", self.parse(params[0]), self.parse(params[1])),
/// 										_ => format!("<em>{}</em>", self.parse(content)),
/// 									}
/// 								}
/// 								"image" => format!("<img src=\"{content}\">"),
/// 								"" => {
/// 									let params = part(content, 0);
/// 									self.parse(params.choose(&mut rng).unwrap_or(&"")).into()
/// 								}
/// 								_ => format!("[{0}/{1}/{0}]", tag, self.parse(content)),
/// 							};
/// 							out.push_str(&content);
/// 							end = idx + 1;
/// 						}
/// 						// 空じゃないならなにもしない（スタックを解消したところで満足し、中身の処理は↑で再帰的に行う）
/// 					}
/// 					// 先頭以外も一致するかを確認する処理にすればタグの交差を処理できるけど、よくわからなかったので一旦保留
/// 				}
/// 				b'\\' => {
/// 					// エスケープ
/// 					if let Some((_, b)) = bytes.next_if(|(_, b)| matches!(b, b'[' | b']' | b'|' | b'/' | b'\\')) {
/// 						// とりあえず読み飛ばし、ネストが無いときのみ出力
/// 						if stack.is_empty() {
/// 							out.push_str(&raw[end..idx]);
/// 							out.push(b as char);
/// 							end = idx + 2;
/// 						}
/// 						// ネストがある場合は出力処理を最終的な再帰に任せる
/// 					}
/// 				}
/// 				_ => (),
/// 			}
/// 		}
/// 		if end > 0 {
/// 			out.push_str(&raw[end..]);
/// 			Cow::Owned(out)
/// 		} else {
/// 			Cow::Borrowed(raw)
/// 		}
/// 	}
/// }
/// ```
pub trait TagFormat {
	fn parse<'a>(self, raw: &'a str) -> Cow<'a, str>;
}
