# 私がやりたいこと
oxというパッケージマネージャーを作りたいです。

ox install <pkgname>← パッケージインストール
ox update ← アップデート
ox upgrade ← アップグレード

# ソフト保存ディレクトリ
ユーザー環境限定で、unixは~/.ox、windowsはC:\Users\<ユーザーネーム>\.oxでパッケージを保存できるようにしたいです
パッケージの参照は、リモートレポジトリ内(現時点ではhttps://github.com/nazozokc/ox/tree/main/packages)packages/から参照するようにしたいです。

# 使うツール
cargo
rustc

# 言語
rust