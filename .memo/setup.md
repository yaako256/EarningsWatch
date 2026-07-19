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

### タブ幅
`rustfmt.toml`を作り、以下を設定
```toml
tab_spaces = 2
```




# フロントエンド側(React)
## セットアップ前の準備について
1. フォルダ作成
前節でやったと思うが、フォルダだけ作っておく。
そしてDockerfileを入れておく。
```bash
mkdir frontend
```

2. dockerfileの編集
`frontend/Dockerfile`の行を以下のように編集する。
```dockerfile
#CMD ["npm", "run", "dev", "--", "--host", "0.0.0.0"]
CMD ["sleep", "infinity"]
```

3. 編集用にコンテナを起動
```
docker compose -f compose.yaml -f compose.dev.yaml up --build
```
もしかしたらこの時に、`frontend/node_module/`がないのにvolume化してるよと言われるかもしれない。
その時は`compose.dev.yaml`でその行を一旦コメントアウトする。

4. 後述のセットアップが終わったらDockerfileを元に戻す。

## セットアップ
カレントディレクトリにプロジェクトを作成する。
```bash
# 移動
cd frontend

# プロジェクト作成
npm create vite@latest .

# インストール
npm install
```
**選択オプション:**
| 項目 | 選択内容 |
| :--- | :--- |
| **Install required package (create-vite)**| Yes (`y`を入力) |
| **Current directory is not empty** | Ignore files and continue |
| **Select a framework** | React |
| **Select a variant** | TypeScript + React Compiler |
| **Which linter to use?** | ESLint |
| **Install with npm and start now?** | Yes |

1つ目の質問は、viteのパッケージがないからインストールするねってもの。
2つ目の質問はすでになんかファイルがあるけどいいの？ってやつ。
`frontend/`には`Dockerfile`しかなく、それは残しておきたいため、無視してプロジェクトを展開させる。
5つ目の質問は、どちらでもいいかもしれない。ESLintの方が安定するっぽいのでこっち。

**`.`（カレントディレクトリ）指定による挙動のメモ:**
```
・Project name: コマンドを実行したフォルダ名が自動採用された。(多分)
・Package name: 同上。
・展開場所: そのフォルダ直下にファイル群が展開された。
・補足: プロジェクト名を手動で設定するステップはスキップされた。
```

## 設定変更時
クローンをしたり、初回は`npm install`動作をしなきゃいけなそうである。
```bash
# 開発用コンテナを起動する
# docker compose -f compose.yaml -f compose.dev.yaml up --build

# フロントエンドのshellに入る
# docker compose -f compose.yaml -f compose.dev.yaml exec frontend bash

# インストール
npm install
```


# DB migrationファイル
以下のように作る。
```bash
# バックエンドコンテナに入る
# docker compose -f compose.yaml -f compose.dev.yaml exec backend bash
make backend-shell

# 対象場所に移動
# `migration/`の中ではなく、`migration/`と同じ階層
# cd backend

# migrationファイル作成
sqlx migrate add <"ファイル名">

```