use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};

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
                filename_small TEXT NOT NULL,
                filename_medium TEXT NOT NULL,
                filename_original TEXT NOT NULL,
                metadata TEXT,
                user_id INTEGER NOT NULL,
                FOREIGN KEY (user_id)
                    REFERENCES users (id) 
            );",
        ),
    ]);

    match migrations.to_latest(connection) {
        Ok(_) => {}
        Err(error) => {
            println!("Error applying migrations: {}", error);
            panic!();
        }
    }
}
