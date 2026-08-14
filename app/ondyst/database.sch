table setting
	key text pk
	value text

# ロケーション
table location
	key text pk
	name text
	lore text
	@init(master/location.csv)
# アイテム
table item
	id int pk
	name text
	lore text
	location ref(location.key)?.update(cascade).delete(cascade) index(item_location)
	message text
	@init(master/item.csv)

# キャラクター
table actor
	id int pk.auto
	user uuid unique
	name text
	comment text default('')
	profile text default('')
	icon_list text default('')
	portrait_list text default('')
	icon text expr(SUBSTR(icon_list||char(10),1,INSTR(icon_list||char(10),char(10))-1))
	portrait text expr(SUBSTR(portrait_list||char(10),1,INSTR(portrait_list||char(10),char(10))-1))

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
table chat_mention
	source ref(chat.id).update(cascade).delete(cascade)
	target ref(actor.id).update(cascade).delete(cascade) index(chat_mention_target)
	@pk(source, target)
table chat_anchor
	source ref(chat.id).update(cascade).delete(cascade)
	target ref(chat.id).update(cascade).delete(cascade) index(chat_anchor_target)
	@pk(source, target)
