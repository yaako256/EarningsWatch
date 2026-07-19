/*
backend/crates/config/src/loader.rs
configのロード
*/

// 外部クレート
// config用
use config::{Config, Environment, File};

// 自クレート
use crate::error::{ConfigLoadError, ConfigLoadResult};
use crate::setting::Settings;

/// configのロード
/// EARNINGSWATCH_ENV(dev/prod)を起点に
/// config/.env.{env} と config/config.{env}.toml を読み込み、
/// EARNINGSWATCH__... 形式の環境変数で上書きする。
pub fn load() -> ConfigLoadResult<Settings> {
  // 環境変数から環境ラベルを読み込む
  let env = std::env::var("EARNINGSWATCH_ENV")?;

  // 環境変数から設定ファイルがあるディレクトリパスを読み込む
  let config_dir = std::env::var("EARNINGSWATCH_CONFIG_DIR")?;

  // 環境ラベル別env読み込み
  // 環境変数への登録/上書きをする
  // .envファイルは存在しなくてもエラーにしない
  let _ = dotenvy::from_path(format!("{config_dir}/.env.{env}"));

  // 設定ファイルをロードする
  let raw = Config::builder()
    // 環境ラベル別のconfig取得
    .add_source(File::with_name(&format!("{config_dir}/config.{env}.toml")))
    // 環境変数上書き（EARNINGSWATCH__AAAAA__BBBB_BBB_BBBB形式）
    .add_source(
      Environment::with_prefix("EARNINGSWATCH")
        .prefix_separator("__")
        .separator("__"),
    )
    .build()
    .map_err(ConfigLoadError::Build)?;

  // Settingsにデシリアライズ
  raw.try_deserialize().map_err(ConfigLoadError::Deserialize)
}
