/*
backend/crates/config/src/loader.rs
アプリ全体の設定(Setting)の型定義
*/

// 外部クレート
use serde::Deserialize;

/// 設定まとめ
#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
  pub server: ServerSettings,
  pub database: DatabaseSettings,
  pub jwt: JwtSettings,
  pub cookie: CookieSettings,
  pub security: SecuritySettings,
  pub logging: LoggingSettings,
  pub scraping: ScrapingSettings,
  pub notifier: NotifierSettings,
  pub import: ImportSettings,
  pub dashboard: DashboardSettings,
}

/// サーバ設定
#[derive(Debug, Clone, Deserialize)]
pub struct ServerSettings {
  /// ホストIP
  pub host: String,
  /// ポート番号
  pub port: u16,
}

/// DB設定
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseSettings {
  /// DBのURL
  pub url: String,
}

/// JWT設定
#[derive(Debug, Clone, Deserialize)]
pub struct JwtSettings {
  // JWT秘密鍵
  pub secret: String,
  /// アクセストークン期限 (既定:15分)
  pub access_token_ttl_minutes: i64,
  /// リフレッシュトークン期限 (既定:30日)
  pub refresh_token_ttl_days: i64,
}

/// Cookie設定
#[derive(Debug, Clone, Deserialize)]
pub struct CookieSettings {
  /// Secureを有効にするか (既定:現状は本番/開発ともfalse)
  pub secure: bool,
}

/// セキュリティ設定
#[derive(Debug, Clone, Deserialize)]
pub struct SecuritySettings {
  /// AES-256-GCM鍵 (base64エンコード・32byte) (webhook用)
  pub webhook_enc_key: String,
}

/// ロギング設定
#[derive(Debug, Clone, Deserialize)]
pub struct LoggingSettings {
  // serverのMemoryLayerのフラッシュ窓 (フラッシュ間隔) (秒)
  pub server_flush_window_seconds: u64,
}

/// スクレイピング設定
#[derive(Debug, Clone, Deserialize)]
pub struct ScrapingSettings {
  /// 直近fingerprint取得件数 (件数ベース1本化)(既定:100件)
  pub recent_fingerprint_limit: u32,
}

/// 送信設定
#[derive(Debug, Clone, Deserialize)]
pub struct NotifierSettings {
  /// discordのembedで使用する標準色 EmbedColor::DEFAULT (規定: 水色0x87CEEB)
  pub default_embed_color: String,
}

/// CSV/Excelインポートの設定
#[derive(Debug, Clone, Deserialize)]
pub struct ImportSettings {
  // 証券コードの最大文字数
  pub ticker_max_len: usize,
  // 銘柄名の最大文字数
  pub company_name_max_len: usize,
  // 備考の最大文字数
  pub notes_max_len: usize,
}

/// ダッシュボードの設定
#[derive(Debug, Clone, Deserialize)]
pub struct DashboardSettings {
  // 送信成功の直近n日の定義
  pub recent_sent_days: i64,
  // 送信成功の直近n送信の定義
  pub recent_sent_min_count: u32,
  // 送信失敗の直近n日の定義
  pub recent_failed_days: i64,
  // 送信失敗の直近n送信の定義
  pub recent_failed_min_count: u32,
  // 管理者で実行履歴の直近n回の定義
  pub admin_recent_runs_count: u32,
}
