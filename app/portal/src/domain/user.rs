use super::*;

pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.route("", web::get().to(list));
	cfg.route("{user}", web::get().to(user));
}

async fn list(web::Query(page): web::Query<common::Pagination>, req_type: ReqType, id: Option<Identity>, _: StateHandle, pool: web::Data<Pool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	let offset = page.offset as i64;
	let limit = page.limit as i64;

	let pool = pool.as_ref();
	let records = sqlx::query_scalar!("SELECT name FROM user LIMIT ?,?", offset, limit).fetch_all(pool).await?;

	match req_type {
		ReqType::Empty => Ok(HttpResponse::Ok().json(records)),
		_ => {
			let mut ctx = tera::Context::new();
			ctx.insert("list", &records);
			let body = Page::default().user_data_opt(UserData::load_opt(&id, &pool).await?).render_with_ctx("user_list.html", &tmpl, ctx)?;
			Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
		}
	}
}

async fn user(user: web::Path<String>, id: Option<Identity>, _: StateHandle, pool: web::Data<Pool>, tmpl: web::Data<Tera>) -> common::Result<impl Responder> {
	#[derive(serde::Serialize)]
	struct Section<'a> {
		title: Option<&'a str>,
		content: &'a str,
	}

	let name = user.into_inner();

	let pool = pool.as_ref();
	let record = sqlx::query!("SELECT name,profile FROM user WHERE name=?", name).fetch_optional(pool).await?.ok_or(ErrorNotFound("対象のユーザーが存在しません"))?;

	let mut section_iter = record.profile.split("\n# ");
	let mut sections = Vec::new();
	if let Some(section) = section_iter.next() {
		if let Some(section) = section.strip_prefix("# ") {
			let (title, content) = section.split_once("\n").unwrap_or((section, ""));
			sections.push(Section { title: Some(title), content });
		} else {
			sections.push(Section { title: None, content: section });
		}
	}
	for section in section_iter {
		let (title, content) = section.split_once("\n").unwrap_or((section, ""));
		sections.push(Section { title: Some(title), content });
	}

	let mut ctx = tera::Context::new();
	ctx.insert("name", &record.name);
	ctx.insert("profile", &sections);
	let body = Page::default().title(&format!("{} - untroche.portal", record.name)).user_data_opt(UserData::load_opt(&id, &pool).await?).render_with_ctx("user.html", &tmpl, ctx)?;
	Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
}
