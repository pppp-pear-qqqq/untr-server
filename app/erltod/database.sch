table setting
	key text pk
	value text

# キャラクター
table actor
	id int pk
	user uuid
	name text
	comment text default('')
	profile text default('')
	icons text default('')
	portraits text default('')

# 装備品マスター
table equipment
	id int pk
	name text
	lore text
	role ref(role).update(cascade).delete(setnull)
	cost int
	trigger blob
	effect blob

# キャラクターの装備品所持状況
table actor_equipment
	actor ref(actor.id).update(cascade).delete(cascade)
	equipment ref(equipment.id).update(cascade).delete(cascade)
	@pk(actor, equipment)


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
