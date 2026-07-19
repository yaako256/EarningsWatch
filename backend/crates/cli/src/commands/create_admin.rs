/*
backend/crates/cli/src/commands/create_admin.rs
管理者ユーザを作成するコマンド
*/
use sqlx::PgPool;

pub async fn run(pool: &PgPool, username: String) {
  let user_repo = infra::PgUserRepository::new(pool.clone());

  let password =
    rpassword::prompt_password("管理者パスワード: ").expect("パスワードの入力に失敗しました");
  let password_confirm =
    rpassword::prompt_password("管理者パスワード(確認): ").expect("パスワードの入力に失敗しました");

  if password != password_confirm {
    eprintln!("パスワードが一致しません");
    std::process::exit(1);
  }

  match app::create_admin_user(&user_repo, username, &password).await {
    Ok(user) => {
      println!("管理者ユーザを作成しました: {}", user.username);
    }
    Err(e) => {
      eprintln!("管理者ユーザの作成に失敗しました: {e}");
      std::process::exit(1);
    }
  }
}
