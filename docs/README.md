# untroche-server
起動
`docker compose up`

停止
`docker compose down`

再ビルド
`docker compose build --no-cache`

Clone直後は、`app/*/resource/`以下に以下4フォルダがあることを確認（無いなら中身空でいいから作る）
`image` `script` `style` `ts`
特にscriptは.gitignoreに含まれているのでまず無いはず
また、Cargo.lockが必要なので`cargo update`を実行しておく

databaseも無いのでそれもやっておく
`../untr-sch.exe -s app/portal/database.sch -o app/portal/database.db`
こんな感じ

あと関係ないけど久々に開発環境起動したときは`rustup update`やっておくといい

Rustプロジェクトのビルドが必要なコンテナを一斉に起動したとき、たまにおかしくなるので、もう一回やるとよい
多分同時にtargetディレクトリを触ろうとしてダメになるんだと思う

git使うときaの共通事項なんだけど、core.autocrlfをinputにしておくといい
`%LocalAppData%\GitHubDesktop\app-*\resources\app\git\cmd\git.exe config --global core.autocrlf input`
こうなんだけど、変数埋め込みが使えないから手動でパス持ってくるといい　いやわかんない　できるのか？
あとバージョン部分（app-*）はちゃんと実体を持ってくること　結局ここで手動入るからもうエクスプローラーで辿った方が速いと思う
最終的にはUser/{username}/.gitconfigを弄れればいいらしいからもうここ手作業で触った方が良いと思う
