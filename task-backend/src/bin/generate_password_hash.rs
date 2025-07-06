use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use std::env;
use std::io::{self, Write};

fn main() {
    let args: Vec<String> = env::args().collect();

    let password = if args.len() > 1 {
        // コマンドライン引数からパスワードを取得
        args[1].clone()
    } else {
        // インタラクティブにパスワードを入力
        print!("Enter password to hash: ");
        io::stdout().flush().unwrap();

        let mut password = String::new();
        io::stdin().read_line(&mut password).unwrap();
        password.trim().to_string()
    };

    // パスワードが空でないかチェック
    if password.is_empty() {
        eprintln!("Error: Password cannot be empty");
        std::process::exit(1);
    }

    // Argon2でハッシュ生成
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(password_hash) => {
            println!("\n=== Password Hash Generated ===");
            println!("Password: {}", password);
            println!("Hash: {}", password_hash);
            println!("\nTo use in .env file:");
            println!("INITIAL_ADMIN_PASSWORD_HASH={}", password_hash);
        }
        Err(e) => {
            eprintln!("Error generating password hash: {}", e);
            std::process::exit(1);
        }
    }
}
