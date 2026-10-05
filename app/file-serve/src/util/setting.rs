use actix_web::mime;
use dashmap::DashMap;
use fxhash::FxHashMap as HashMap;
use uuid::Uuid;

pub type UserMap = DashMap<String, PathMap>;
pub type PathMap = HashMap<String, Setting>;
pub type Setting = HashMap<Option<FetchDest>, Resource>;

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Hash, serde::Deserialize)]
pub enum FetchDest {
	audio,
	audioworklet,
	document,
	embed,
	empty,
	fencedframe,
	font,
	frame,
	iframe,
	image,
	manifest,
	object,
	paintworklet,
	report,
	script,
	serviceworker,
	sharedworker,
	style,
	track,
	video,
	webidentity,
	worker,
	xslt,
}

// トレイト作って保持側はBox<dyn Resource>としてそれぞれ実装する形でもいい どうしよう
// RandomFileをVec<(u32, File)>と実装できるのは多少綺麗かもしれないが、そのくらいなら個別実装でも良いような気もする
// レスポンス作成関数の規模を確認してから？　もしくは設定アップロードの実装次第かも
pub enum Resource {
	File {
		key: Uuid,
		mime: mime::Mime,
	},
	RandomFile {
		items: Vec<(Uuid, mime::Mime, u32)>, // key,mime,weight
		cache: bool,
	},
	FontRender {
		key: Uuid,
		width: u32,
		height: u32,
		line_length: u32,
	},
}
