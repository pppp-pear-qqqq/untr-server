## 基本
https://ondyst.untroche.com
無駄なポータルログイン機能は排除（めんどうだから）
とはいえ試験的にやってもいいか

この辺昔の情報なので適宜変更する

## URL構造
```
/index
	:GET	リダイレクト
		未ログイン->entry
		ログイン済->home
/entry
	:GET	ログインページ
	/login	:POST
	/register:POST
/home
	:GET	ホーム画面
	/actor
		:GET	プロフィール編集画面
		:PATCH	更新
	/setting
		:GET	アカウントやローカルの設定
		:PATCH	設定反映
	/notice
		:GET	通知閲覧画面
		:POST	画面に埋め込む用のjsonデータ取得
/location
	:GET	全体マップ閲覧
	:PUT	ロケーション作成
	/{key}
		:GET	ロケーション閲覧
			?ページネーションとか表示数とか
		:POST	発言投稿
		/stream	SSE接続
		/item
			:POST	アイテム使用テキスト取得または投稿
			:PUT	アイテム登録
	/post
		:POST	発言をJSONデータで取得
		:DELETE	発言削除
/actor
	:GET	キャラクターリスト
	/{id}
		:GET	キャラクタープロフィール
```

## データベース
```
table user
	id uuid pk
	name text uniq
	password text

table actor
	id int pk
	user ref(user.id).update(cascade).delete(cascade) index
	name text index
	icon_list text # アイコンURLを改行区切り
	icon expr(icon_list.lines[0]) # 疑似コード、先頭1行
	comment text
	profile text
	portrait text
	badge int # ログ公開可、二次創作可、実績などのバッジフラグ
	visible bool

table timeline
	id int pk.auto
	timestamp timestamp
	location ref(location.id).update(cascade).delete(cascade)
	actor ref(actor.id).update(cascade).delete(cascade)
	name text
	icon text
	content text
	reply ref(timeline.id).update(cascade).delete(setnull) index
	@index(location,id)

table location
	id uuid pk
	key text uniq index # URL文字列
	owner ref(actor.id).update(cascade).delete(setnull)
	name text
	lore text
	visible bool
	item_permission bool

table item
	id int pk.auto
	location ref(location.id).update(cascade).delete(cascade)
	name text
	lore text
	effect text
	immediate bool # アイテム使用時、効果テキストを即座に投稿するか(true)、発言テキストエリアに挿入するか(false)
```

## localStorage
```
theme: string # サイトテーマ(light/dark)
fav_locations: [
	{
		key: string
		name: string
	}
]
```

## ロケーション
**空色広場** plaza
カラフルなレンガが敷き詰められた、町でいちばん大きな広場。広いだけで特に何もないけれど、いろんな人が集まってくる。

**真朱駅** station
町の外と中とを繋ぐ玄関口。線路がどこに繋がっているのかはよくわからないが、列車は『移動』の象徴であるので、パステルサイトの法則に従って様々な世界に行くことができる。
- きっぷ 白紙のきっぷ。別に持っていなくても電車には乗れるらしい。/きっぷを購入した。なにも書かれていない。

**深緑公園** park
さまざまな遊具やベンチなどが設置された、緑豊かな公園。澄んだ空気に満ちている気がする。
- すべり台 ゾウを模した結構大きめのすべり台。鼻の下には砂場が広がっている。/すべり台を滑った。楽しい！
- ブランコ シンプルなブランコ。あまり高く漕ぎすぎないように。/ブランコをこいだ。楽しい！
- 鉄棒 三段階の高さがある鉄棒。子供から大人まで挑戦することができる。/鉄棒で[跳び上がり回り|逆上がり|横とび越し下り|大車輪|ふとん干し]に挑戦した。

**喫茶店：錆色時計** coffee
店名どおりの丸時計が、少しだけ怪しいリズムで時を刻んでいる。コーヒー豆と抽出用の機械、テーブルとそれを囲む椅子。カウンター席も設けられている。
- コーヒー さまざまな豆を選んで注文することができる。/コーヒーを飲んだ。
- 紅茶 メインではないが一応置いているらしい。アイス・ホットしか選択肢がない。/紅茶を飲んだ。
- カレーライス 喫茶店の主力商品。コーヒーが隠し味に使われていて、少しすっぱめ。/カレーライスを食べた。

**やまぶき雑貨店** article
少し狭めの店内、謎のオブジェや小瓶などが空間を埋めている。大きく開けられた窓が陽の光を取り込んで、それらをきらきら輝かせている。
- 青い小瓶 なんらかの液体が詰められた小瓶。何に使うのかは分からない。/青い小瓶を購入した。中身は化粧水らしい。
- 赤い小瓶 なんらかの液体が詰められた小瓶。何に使うのかは分からない。/赤い小瓶を購入した。中身は傷薬らしい。

**青竹診療所** clinic
町中のけがや病気をすべて請け負う万能の診療所。パステルサイトの住人はあまり病気にかからないようで、仕事の幅に対して診療所の規模は小さい。

**白花学校** school
教育施設。小学校なのか中学校なのか高校なのかはよくわからない。たくさんの教室を含む校舎が、校庭をぐるっと取り囲むような構造。

**うるみ図書館** library
小説、実用書、雑誌など、いろんな本が納められた図書館。少し薄暗く、静かな空気が漂っている。
- 分厚い本 難しそうな厚い本。/分厚い本を読んでいる……
- 薄い本 雑誌など。別にセンシティブではない。/薄い本を読んでいる……

**若葉温室** garden
大きなガラスドームの中に、さまざまな植物が所狭しと伸びている。見たことのないようなカラフルな花なども多い。

**秘色の路地** path
町の隙間を縫うような細い路地。うまく歩けば近道になるだろうが、複雑さゆえに大抵は迷子を量産している。
