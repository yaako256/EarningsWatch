# セットアップ備忘録

# git
以下を実行
```bash
# git開始
git init

# ブランチ名変更
git branch -m main
```

# バックエンド側(Rust)
Cragoで専用コマンドを使わなくてもWorkspaceの親は作れる。

## Workspaceを作成する
1. ルートディレクトリを作成する  
```bash
mkdir backend
```
2. クレート用ディレクトリを作成する  
```bash
cd backend
mkdir crates
```
3. backendルートにCargo.tomlを作成する  
以下を作成する
```toml
[workspace]
resolver = "3"
members = ["crates/*"]

[workspace.dependencies]
```
4. migration用フォルダ作成
```bash
cd backend # 既に移動していたら不要
mkdir migrations
touch migrations/.gitkeep
```

## 子クレートを作成する
`--vcs none`を付けると、`.git/`および`gitignore`が生成されなくなる。
```bash
# クレート用ディレクトリに移動
cd backend/crates

# バイナリクレート作成
cargo new <"クレート名"> --bin --vcs none

# ライブラリクレート作成
cargo new <"クレート名"> --lib --vcs none
```

## メモ:子クレートのCargo.toml
`dependencies`の書き方がworkspace用になる。
```
[dependencies]
# workspace内の別クレート
aaa = { path = "../aaa" }
bbb = { path = "../bbb" }

# workspace共通外部クレート
ddd = { workspace = true }
eee = { workspace = true }

# 固有クレート
fff = "0.8"
```


# Docker周辺の整備




# フロントエンド側(React)
## セットアップ前の準備について
1. フォルダ作成
前節でやったと思うが、フォルダだけ作っておく。
そしてDockerfileを入れておく。
```bash
mkdir frontend
```