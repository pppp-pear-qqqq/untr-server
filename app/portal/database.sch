table setting
	key text pk
	value text

table report
	id int pk
	timestamp timestamp
	user ref(user.id).update(cascade).delete(setnull)
	tag text
	body text
	checked boolean default(FALSE)

table user
	id uuid pk
	name text unique index(user_name)
	password text
	profile text default('')		# プロフィール、プレイヤーのSNSアカウントやキャラクターなど　全部ひとまとめにする　未エスケープ
	webhook text?		# 共通のウェブフックURL
	mutes blob			# ユーザーミュートのリスト(Vec<id>)

table auth
	code uuid pk
	timestamp timestamp	# 期限となるタイミングを保持する
	user ref(user.id).update(cascade).delete(cascade)

table bbs
	id int pk
	timestamp timestamp
	user ref(user.id).update(cascade).delete(setnull)
	address text		# ポータル側で各アプリから参照するBBSを管理してもいい（どのみち読み込みは動的なので）リクエスト返却時にCORSを良くする
	body text			# 後から編集とかやらないので投稿時にエスケープ処理する
