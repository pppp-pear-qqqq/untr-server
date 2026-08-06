table setting
	key text pk
	value text

# ロケーション
table location
	key text pk
	name text
	lore text
# アイテム
table item
	id int pk
	name text
	lore text
	location ref(location.key).update(cascade).delete(cascade) index(item_location)
	message text

# キャラクター
table actor
	id int pk
	user uuid
	name text
	comment text default('')
	profile text default('')
	icons text default('')
	portraits text default('')
	icon text expr(SUBSTR(icons||char(10),1,INSTR(icons||char(10),char(10))-1))
	portrait text expr(SUBSTR(portraits||char(10),1,INSTR(portraits||char(10),char(10))-1))

# キャラクター操作ログ
table log
	id int pk
	timestamp timestamp
	actor ref(actor.id).update(cascade).delete(setnull) index(log_actor)
	body text

# チャット関連
table chat
	id int pk
	timestamp timestamp
	location text index(chat_location)
	actor ref(actor.id).update(cascade).delete(setnull) index(chat_actor)
	name text
	icon text
	body text
table chat_anchor
	source ref(chat.id).update(cascade).delete(cascade)
	target ref(chat.id).update(cascade).delete(cascade) index(chat_anchor_target)
	@pk(source, target)
table chat_mention
	source ref(chat.id).update(cascade).delete(cascade)
	target ref(actor.id).update(cascade).delete(cascade) index(chat_mention_target)
	@pk(source, target)
