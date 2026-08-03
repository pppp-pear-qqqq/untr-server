use std::sync::OnceLock;

use tera::{Context, Tera};

static MINIFY_CFG: OnceLock<minify_html::Cfg> = OnceLock::new();

pub trait PageRender
where
	Self: serde::Serialize,
{
	fn ctx(&self) -> tera::Result<Context> {
		Context::from_serialize(self)
	}

	fn render(&self, tmpl_name: &str, engine: &Tera) -> tera::Result<Vec<u8>> {
		let cfg = MINIFY_CFG.get_or_init(|| {
			let mut cfg = minify_html::Cfg::new();
			cfg.minify_css = true;
			cfg.minify_js = true;
			cfg
		});
		let body = engine.render(tmpl_name, &self.ctx()?)?;
		Ok(minify_html::minify(body.as_bytes(), cfg))
	}

	fn render_with_ctx(&self, tmpl_name: &str, engine: &Tera, mut ctx: Context) -> tera::Result<Vec<u8>> {
		let cfg = MINIFY_CFG.get_or_init(|| {
			let mut cfg = minify_html::Cfg::new();
			cfg.minify_css = true;
			cfg.minify_js = true;
			cfg
		});
		ctx.extend(self.ctx()?);
		let body = engine.render(tmpl_name, &ctx)?;
		Ok(minify_html::minify(body.as_bytes(), cfg))
	}
}
