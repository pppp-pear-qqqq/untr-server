use std::{borrow::Cow, sync::OnceLock};

use regex::Regex;

pub trait HTMLEncode {
	fn br(&self) -> String;
	fn escape(&self, quot: bool) -> Cow<'_, str>;
	fn escape_and_link(&self) -> Cow<'_, str>;
	fn tag<T: TagFormat>(&self, format: T) -> Cow<'_, str>;
}
impl HTMLEncode for str {
	fn br(&self) -> String {
		const BR: &str = "<br>";
		let mut result = String::with_capacity(self.len() + 32);
		let mut chars = self.chars().peekable();
		while let Some(c) = chars.next() {
			match c {
				'\r' => {
					if chars.peek() == Some(&'\n') {
						chars.next();
					}
					result.push_str(BR);
				}
				'\n' => {
					result.push_str(BR);
				}
				_ => result.push(c),
			}
		}
		result
	}
	fn escape(&self, quot: bool) -> Cow<'_, str> {
		let repl = |b: u8| match b {
			b'<' => Some("&lt;"),
			b'>' => Some("&gt;"),
			b'&' => Some("&amp;"),
			b'"' if quot => Some("&quot;"),
			b'\'' if quot => Some("&apos;"),
			_ => None,
		};
		let bytes = self.as_bytes();
		for (i, &b) in bytes.iter().enumerate() {
			if let Some(r) = repl(b) {
				let mut owned = String::with_capacity(self.len() + 32);
				owned.push_str(&self[..i]);
				owned.push_str(r);
				let mut p = i + 1;
				for (i, &b) in bytes.iter().enumerate().skip(p) {
					if let Some(r) = repl(b) {
						owned.push_str(&self[p..i]);
						owned.push_str(r);
						p = i + 1;
					}
				}
				owned.push_str(&self[p..]);
				return Cow::Owned(owned);
			}
		}
		Cow::Borrowed(self)
	}
	fn escape_and_link(&self) -> Cow<'_, str> {
		static RE: OnceLock<Regex> = OnceLock::new();
		let re = RE.get_or_init(|| {
			let regs = [
				r#"(?<url>https?://[^\s<>"']+)"#,
				r"(?<misskey>@[\w_\-]+@[\w_\-]+(?:\.[\w_\-]+)+)",
				r"(?<bsky>@[\w_\-]+(?:\.[\w_\-]+)+)",
				r"(?<twitter>@[\w_\-]{4,15})",
			];
			Regex::new(&format!(r"(^|\s)(?:{})", regs.join("|"))).unwrap()
		});
		let mut out = String::with_capacity(self.len() * 2);
		let mut end = 0;
		for caps in re.captures_iter(self) {
			let m = caps.get(0).unwrap();
			out.push_str(&self[end..m.start()].escape(false));
			out.push_str(&caps[1]);
			end = m.end();
			let (href, body) = if let Some(m) = caps.name("url") {
				let m = m.as_str();
				(m.replace('"', "%22"), m.escape(false))
			} else if let Some(m) = caps.name("misskey") {
				let m = m.as_str();
				let (user, domain) = m.rsplit_once('@').unwrap();
				(format!("https://{domain}/{user}"), m.escape(false))
			} else if let Some(m) = caps.name("bsky") {
				let m = m.as_str();
				(format!("https://bsky.app/profile/{}", &m[1..]), m.escape(false))
			} else if let Some(m) = caps.name("twitter") {
				let m = m.as_str();
				(format!("https://x.com/{}", &m[1..]), m.escape(false))
			} else {
				out.push_str(&m.as_str().escape(false));
				continue;
			};
			out.push_str(&format!("<a target=\"_blank\" href=\"{href}\">{body}</a>"));
		}
		if end > 0 {
			out.push_str(&self[end..]);
			Cow::Owned(out)
		} else {
			self.escape(false)
		}
	}
	fn tag<T: TagFormat>(&self, format: T) -> Cow<'_, str> {
		format.parse(&self)
	}
}

pub trait HTMLDecode {
	fn unescape(&self) -> Cow<'_, str>;
	fn rm_br(&self) -> String;
}
impl HTMLDecode for str {
	fn unescape(&self) -> Cow<'_, str> {
		static SPECIALS: [(&str, char); 7] = [("&lt;", '<'), ("&gt;", '>'), ("&amp;", '&'), ("&quot;", '"'), ("&apos;", '\''), ("&#39;", '\''), ("&nbsp;", ' ')];
		if !self.contains('&') {
			return Cow::Borrowed(self);
		}
		let mut result = String::with_capacity(self.len());
		let mut i = 0;
		let s_bytes = self.as_bytes();
		while i < self.len() {
			if s_bytes[i] == b'&' {
				// エンティティの候補を探す
				let rest = &self[i..];
				// 代表的なエンティティとのマッチング
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
					// 有効なエンティティでなければ、ただの '&' として扱う
					result.push('&');
					i += 1;
				}
			} else {
				// & 以外の文字はそのまま追加
				// 安全のため、ここでもスライスを使って次の & まで一気に飛ばすとより高速
				let next_amp = self[i..].find('&').unwrap_or(self.len() - i);
				result.push_str(&self[i..i + next_amp]);
				i += next_amp;
			}
		}
		Cow::Owned(result)
	}
	fn rm_br(&self) -> String {
		self.replace(&['\n', '\r'], "")
	}
}

/// # 実装例
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
	fn parse(self, raw: &str) -> Cow<'_, str>;
}
