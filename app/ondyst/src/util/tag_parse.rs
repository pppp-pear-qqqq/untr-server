use common::{HTMLEncode, Tag, TagFormat};
use log::debug;
use rand::seq::IndexedRandom as _;

#[derive(Clone, Copy)]
pub struct Ondyst;

impl TagFormat for Ondyst {
	#[cfg(not(target_arch = "wasm32"))]
	fn from_args(_args: &std::collections::HashMap<String, tera::Value>) -> Self {
		Self
	}

	fn format(&self, tag: Tag<'_>, link: bool) -> String {
		let mut rng = rand::rng();
		match tag.name {
			"b" | "i" | "u" | "s" | "large" | "small" | "em" | "rainbow" => {
				format!("<{0}>{1}</{0}>", tag.name, tag.content.to_html(self, link))
			}
			"ruby" => {
				let params = tag.content.splitn(2, '|').collect::<Vec<_>>();
				match params.len() {
					2 => format!("<ruby>{}<rp>(</rp><rt>{}</rt><rp>)</rp></ruby>", params[0].to_html(self, link), params[1].to_html(self, link)),
					_ => format!("<em>{}</em>", tag.content.to_html(self, link)),
				}
			}
			"image" => format!("<img src=\"{}\">", tag.content.replace("\"", "%22")),
			"random" => {
				let params = tag.content.split('|').collect::<Vec<_>>();
				debug!("{params:?}");
				format!("<random>{}</random>", params.choose(&mut rng).unwrap().to_html(self, link))
			}
			_ => format!("[{0}/{1}/{0}]", tag.name, tag.content.to_html(self, link)),
		}
	}
}

#[cfg(target_arch = "wasm32")]
pub mod wasm {
	use super::*;
	use wasm_bindgen::prelude::*;

	#[wasm_bindgen]
	pub fn to_html(input: &str, link: bool) -> String {
		input.to_html(&Ondyst, link)
	}
}
