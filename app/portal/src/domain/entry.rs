use actix_session::Session;
use argon2::{
	Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
	password_hash::{SaltString, rand_core::OsRng},
};

use super::*;

/// リソース
pub fn cfg(cfg: &mut web::ServiceConfig) {
	cfg.route("", web::get().to(index));
	cfg.service(web::resource("login").get(view_login).post(login));
	cfg.service(web::resource("register").post(register));
	cfg.route("logout", web::to(logout));
}

/// ログイン・登録画面の表示
async fn index(id: Option<Identity>, pool: web::Data<SqlitePool>, tmpl: web::Data<tera::Tera>) -> common::Result<impl Responder> {
	let body = Page::default().user_data_opt(UserData::load_opt(&id, &pool).await?).render("entry.html", &tmpl)?;
	Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
}

/// 最小化したログイン画面の表示
async fn view_login(tmpl: web::Data<tera::Tera>) -> common::Result<impl Responder> {
	let body = Page::default().min().render("login.html", &tmpl)?;
	Ok(HttpResponse::Ok().content_type(header::ContentType::html()).body(body))
}

#[derive(serde::Deserialize)]
struct Login {
	username: String,
	password: String,
}
/// ログイン処理
async fn login(web::Form(info): web::Form<Login>, session: Session, pool: web::Data<SqlitePool>) -> common::Result<impl Responder> {
	let pool = pool.as_ref();
	let record = sqlx::query!("SELECT id,password FROM user WHERE name=?", info.username).fetch_one(pool).await?;
	let parsed_hash = PasswordHash::new(&record.password)?;
	if Argon2::default().verify_password(info.password.as_bytes(), &parsed_hash).is_ok() {
		Identity::set(&session, record.id)?;
		Ok(HttpResponse::NoContent().finish())
	} else {
		Err(ErrorUnauthorized("ユーザー名またはパスワードが異なります").into())
	}
}

#[derive(serde::Deserialize)]
struct Register {
	username: String,
	password: String,
}
/// 新規登録処理
async fn register(web::Form(info): web::Form<Register>, session: Session, pool: web::Data<SqlitePool>) -> common::Result<impl Responder> {
	let id = Uuid::new_v4();
	let id = id.as_bytes().as_slice();
	let hashed = Argon2::default().hash_password(info.password.as_bytes(), &SaltString::generate(&mut OsRng))?.to_string();
	let mutes = rkyv::to_bytes::<rkyv::rancor::Error>(&Vec::<String>::new())?;
	let mutes = mutes.as_slice();

	let pool = pool.as_ref();
	let query = sqlx::query!("INSERT INTO user(id,name,password,mutes) VALUES(?,?,?,?)", id, info.username, hashed, mutes);
	match query.execute(pool).await {
		Ok(_) => {
			Identity::set(&session, id.to_vec())?;
			Ok(HttpResponse::NoContent().finish())
		}
		Err(sqlx::Error::Database(err)) if err.is_unique_violation() => Err(ErrorForbidden("ユーザー名が既に存在します").into()),
		Err(err) => Err(ErrorInternalServerError(err).into()),
	}
}

async fn logout(session: Session) -> common::Result<impl Responder> {
	Identity::remove(&session);
	Ok(HttpResponse::NoContent().finish())
}
