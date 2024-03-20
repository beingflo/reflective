use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};
use tracing::{error, info};

#[tracing::instrument(skip_all)]
pub fn apply_migrations(connection: &mut Connection) {
    let migrations = Migrations::new(vec![
        M::up(
            "CREATE TABLE users (
                id INTEGER PRIMARY KEY AUTOINCREMENT, 
                username TEXT NOT NULL,
                password TEXT NOT NULL,
                config TEXT
            );",
        ),
        M::up(
            "CREATE TABLE tokens (
                token TEXT NOT NULL, 
                user_id INTEGER NOT NULL,
                FOREIGN KEY (user_id)
                    REFERENCES users (id) 
            );",
        ),
        M::up(
            "CREATE TABLE images (
                id INTEGER PRIMARY KEY AUTOINCREMENT, 
                filename TEXT NOT NULL,
                metadata TEXT,
                user_id INTEGER NOT NULL,
                FOREIGN KEY (user_id)
                    REFERENCES users (id) 
            );",
        ),
    ]);

    match migrations.to_latest(connection) {
        Ok(_) => {
            info!(message = "Applied migrations successfully");
        }
        Err(error) => {
            error!(message = "Error applying migrations", error = %error);
            panic!();
        }
    }
}
