# 私がやりたいこと
inxというパッケージマネージャーを作りたいです。

inx install <pkgname>← パッケージインストール
inx update ← アップデート
inx upgrade ← アップグレード

# ソフト保存ディレクトリ
ユーザー環境限定で、unixは~/.inx、windowsはC:\Users\<ユーザーネーム>\.inxでパッケージを保存できるようにしたいです
パッケージの参照は、リモートレポジトリ内(現時点ではhttps://github.com/nazozokc/inx/tree/main/packages)packages/から参照するようにしたいです。

# 使うツール
cargo
rustc

# 言語
rust