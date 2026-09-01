# untroche-server

## 起動・停止・ビルド
* **起動:** `docker compose up`
* **停止:** `docker compose down`
* **再ビルド:** `docker compose build --no-cache`
* **デタッチ** `-d` オプションを付ける、または起動後に[d]を押す

特定コンテナのみを対象にする場合はサブコマンド以降にコンテナ名追記

Nginxの設定ファイル変更が反映されない場合、`--force-recreate`オプションを付ける

## 初回セットアップ（Clone直後）
リポジトリをクローンした直後は、以下のセットアップ作業を行う
* **Cargo.lock の生成:**
`cargo update` を実行する

* **データベースの作成:**
以下のコマンドを実行し、`.db` および `.db.txt` ファイルを生成する  
`../untr-sch.exe -s app/{app_name}/database.sch -o app/{app_name}/database.db`  
{app_name}は適切に書き換える  
untr-sch.exeが無い場合はそちらのプロジェクトもCloneおよびビルドしておく（ビルド手順はuntr-sch側のREADME.mdに記載）

## 開発時のTipsとトラブルシューティング
* **コンテナ起動時のエラー:**
Rustプロジェクトのビルドが必要なコンテナを一斉に起動した際、`target` ディレクトリへのアクセス競合によってエラー終了することがあるらしい  
その場合はもう一度起動コマンドを実行することで正常に立ち上がるはず

* **Rustのアップデート:**
久々に開発環境を起動した際は、`rustup update` を実行しておくと吉

## Gitの設定
リポジトリを扱う際の共通事項として、改行コードの問題を防ぐために `core.autocrlf` を `input` に設定したほうがよい

* **設定方法:**
ユーザーディレクトリ直下にある `C:/Users/{username}/.gitconfig` ファイルをテキストエディタで直接開き、手作業で編集するのが最も確実で手軽  
gitコマンドがインストールされているなら `git config --global core.autocrlf input` でもいいが、Github Desktopだけインストールしている環境だと使えないと思う

## 気になっていること
- Gitでディレクトリを維持するために`.dummy`ファイルを置くようにしたが、場所がサーバーリソース配信場所なので、多分ブラウザから`/style/.dummy`などでアクセスできる
	- 空ファイルなのでアクセスされたとてなのだが、綺麗ではない気がする
	- 仕様上多分仕方ないし、おまけ隠しコンテンツでも置いておくのが一番丸いのかもしれない　何を置くのかはわからないが　アクセス弾けるならそれでもいい
	- 一応アクセス弾くだけならNginxの設定調整すれば大丈夫か　まあ諸々検討
	- どのみち`.dummy`が必要なのは開発中の過渡期だけで、最終的にはすべてのディレクトリにちゃんと意味のあるファイルが入るわけだし、そうなったら消せばいいだけかも
	- でも消すのはちょっと面倒くさい

# 本番ビルド
`docker run --rm -v "${PWD}:/usr/src/app" -v cargo-cache:/usr/local/cargo/registry -w /usr/src/app rust:slim cargo build --bin portal --release`
実際どういう感じでやっていけばいいのかは手探り

本番サーバーでのディレクトリ構造を参考にしながら、etc/systemd/system以下に.serviceファイルを配置する
etc/nginx/nginx.confも配置する
データベースファイルとかを置きながら良い感じに
