# API設計

## その他
| パス | 内容 | 
| --- | --- |
| "/api/health" | ヘルスチェック。ほぼ開発用用途 |

## 認証関連
| パス | 内容 | 
| --- | --- |
| "/api/auth/login" | ログイン |
| "/api/auth/refresh" | トークンのリフレッシュ |
| "/api/auth/logout" | ログアウト |
| "/api/auth/me" | 再訪問時の自動ログイン用 |

## 管理者ユーザ用
| パス | 内容 |
| --- | --- |
| "/api/悩み中" | ログのn件目～n+m件目を取得 |
| "/api/悩み中" | ログをm件の分解能で表示させる時のページ数取得 |
| "/api/悩み中" | ユーザ一覧のn件目～n+m件目を取得 |
| "/api/悩み中" | ユーザ一覧をm件の分解能で表示させる時のページ数取得 |

他にも、各ユーザが何グループ、何フィルタしてるか、どの媒体への送信をしているかくらいは見れるようにしたい。(フィルタ内容とかは一応個人情報な気がするのでフロントエンドで簡単に見ることはできないようにする？)

何なら別にSQLインジェクションができるような場所を作り、その結果を表示するだけでもいい気がするが、他のとこでもインジェクション出来てしまう気がするのでやめる？

ログやユーザは、フロントエンドで一度に何件表示(取得)するかを選ばせ、ページ遷移などで順々に見れるようにすることを想定

## 全ユーザ共通
| パス | 内容 |
| --- | --- |
|  |  |

ユーザ設定、グループ設定、グループ設定等の一括設定、CSVでの一括インポート、CSVでのインポート、CSVでのエクスポート、決算ログの確認、決算ログのエクスポート、グループでのフィルタの追加/削除/無効化/有効化など、たくさんありそう。


# 参考例(YaakoDrive)
```rust
/// サーバのRouter型を返す
pub fn create_router(state: AppState) -> Router {
  Router::new()
    // health
    .route("/api/health", get(health_handler))
    // auth
    .route("/api/auth/login", post(login_handler))
    .route("/api/auth/refresh", post(refresh_handler))
    .route("/api/auth/logout", post(logout_handler))
    .route("/api/auth/me", get(me_handler))
    // nodeとfileで先に登録するやつら(※注意)
    .route("/api/nodes", get(list_root_handler))
    .route("/api/nodes/folders", post(create_root_folder_handler))
    .route("/api/nodes/upload", post(upload_root_handler))
    // node
    .route("/api/nodes/{id}", get(get_node_handler))
    .route("/api/nodes/{id}/children", get(list_children_handler))
    .route("/api/nodes/{id}/folders", post(create_folder_handler))
    .route("/api/nodes/{id}/rename", patch(rename_node_handler))
    .route("/api/nodes/{id}/move", patch(move_node_handler))
    .route("/api/nodes/{id}", delete(delete_node_handler))
    // file
    .route("/api/nodes/{id}/upload", post(upload_handler)) // 追加
    .route("/api/nodes/{id}/download-url", get(download_url_handler)) // 追加
    .route("/api/files/download/{token}", get(download_handler))
    // trash
    .route("/api/trash", get(list_trash_handler))
    .route("/api/trash/{id}/children", get(list_trash_children_handler))
    .route("/api/trash/{id}/restore", post(restore_node_handler))
    .route("/api/trash/{id}", delete(hard_delete_node_handler))
    // search
    .route("/api/search", get(search_handler))
    // dashboard
    .route("/api/dashboard", get(dashboard_handler))
    // デフォルトだと2MBまでしか送信できないので
    // 送信制限をconfigのmax_size_bytesにする
    .layer(axum::extract::DefaultBodyLimit::max(
      state.config.upload.max_size_bytes as usize,
    ))
    // State管理
    .with_state(state)
}
```





# メモ
## 個人設定項目系
- グループ設定などの初期化処理