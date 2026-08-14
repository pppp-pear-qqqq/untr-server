use super::*;

/// リソース
pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.route("", web::to(index));
	cfg.service(web::resource("setting").get(view_setting).patch(patch_setting));
}

async fn index(id: Identity, pool: web::Data<SqlitePool>, tmpl: web::Data<tera::Tera>) -> common::Result<impl Responder> {
	let pool = pool.as_ref();

	// favはlocalStorageに持っているので、クライアント側で諸々のAPIを叩く　バックエンドではあんまり何もしない

	let body = Page::default().actor_data(ActorData::load(&id, pool).await?).render("home.html", &tmpl)?;
	Ok(HttpResponse::Ok().body(body))
}

async fn view_setting(id: Identity, pool: web::Data<SqlitePool>, tmpl: web::Data<tera::Tera>) -> common::Result<impl Responder> {
	let pool = pool.as_ref();

	// id,user,name,comment,profile,icon_list,portrait_list
	// webhookはportalへのリンクを設置する

	let body = Page::default().actor_data(ActorData::load(&id, pool).await?).render("setting.html", &tmpl)?;
	Ok(HttpResponse::Ok().body(body))
}

#[derive(serde::Deserialize, Validate)]
#[validate(schema(function = "any_some"))]
struct Setting {
	#[validate(length(max = 16, message = "名前は16文字以内で入力してください"))]
	name: Option<String>,
	#[validate(length(max = 48, message = "コメントは48文字以内で入力してください"))]
	comment: Option<String>,
	#[validate(length(max = 8192, message = "プロフィールは8192文字以内で入力してください"))]
	profile: Option<String>,
	#[validate(length(max = 8192, message = "アイコンURLは合計8192文字以内にしてください"))]
	icon_list: Option<String>,
	#[validate(length(max = 4096, message = "ポートレートURLは合計4096文字以内にしてください"))]
	portrait_list: Option<String>,
}
async fn patch_setting(web::Json(info): web::Json<Setting>, id: Identity, pool: web::Data<SqlitePool>) -> common::Result<impl Responder> {
	info.validate()?;
	let mut builder = sqlx::QueryBuilder::new("UPDATE actor SET ");
	let mut sep = builder.separated(',');

	if let Some(v) = &info.name {
		sep.push("name=");
		sep.push_bind_unseparated(v);
	}
	if let Some(v) = &info.comment {
		sep.push("comment=");
		sep.push_bind_unseparated(v);
	}
	if let Some(v) = &info.profile {
		sep.push("profile=");
		sep.push_bind_unseparated(v);
	}
	if let Some(v) = &info.icon_list {
		sep.push("icon_list=");
		sep.push_bind_unseparated(v);
	}
	if let Some(v) = &info.portrait_list {
		sep.push("portrait_list=");
		sep.push_bind_unseparated(v);
	}
	builder.push(" WHERE id=");
	builder.push_bind(*id);

	let pool = pool.as_ref();
	builder.build().execute(pool).await?;
	Ok(HttpResponse::NoContent().finish())
}

fn any_some(v: &Setting) -> Result<(), validator::ValidationError> {
	if v.name.is_some() || v.comment.is_some() || v.profile.is_some() || v.icon_list.is_some() || v.portrait_list.is_some() {
		Ok(())
	} else {
		let mut err = validator::ValidationError::new("empty_setting");
		err.message = Some(std::borrow::Cow::Borrowed("少なくとも1つの項目を変更してください"));
		Err(err)
	}
}
