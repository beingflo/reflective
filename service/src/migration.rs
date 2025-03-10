use rusqlite::Connection;
use rusqlite_migration::{M, Migrations};
use tracing::{error, info};

#[tracing::instrument(skip_all)]
pub fn apply_migrations(connection: &mut Connection) {
    let migrations = Migrations::new(vec![
        M::up(
            "CREATE TABLE user (
                id INTEGER PRIMARY KEY AUTOINCREMENT, 
                username TEXT NOT NULL,
                password TEXT NOT NULL
            );",
        ),
        M::up(
            "CREATE TABLE token (
                token TEXT NOT NULL, 
                user_id INTEGER NOT NULL,
                FOREIGN KEY (user_id)
                    REFERENCES user (id) 
            );",
        ),
        M::up(
            "CREATE TABLE image (
                id INTEGER PRIMARY KEY AUTOINCREMENT, 
                filename TEXT NOT NULL,
                metadata TEXT,
                user_id INTEGER NOT NULL,
                FOREIGN KEY (user_id)
                    REFERENCES user (id) 
            );",
        ),
        M::up(
            "CREATE TABLE variant (
                id INTEGER PRIMARY KEY AUTOINCREMENT, 
                filename TEXT NOT NULL,
                width INTEGER NOT NULL,
                height INTEGER NOT NULL,
                compression_quality INTEGER NOT NULL,
                quality TEXT NOT NULL, 
                image_id INTEGER NOT NULL,
                FOREIGN KEY (image_id)
                    REFERENCES image (id) 
            );",
        ),
        M::up(
            "CREATE TABLE tag (
                id INTEGER PRIMARY KEY AUTOINCREMENT, 
                description TEXT NOT NULL,
                metadata TEXT,
                user_id INTEGER NOT NULL,
                FOREIGN KEY (user_id)
                    REFERENCES user (id) 
            );",
        ),
        M::up(
            "CREATE TABLE image_tag (
                tag_id INTEGER NOT NULL,
                image_id INTEGER NOT NULL,
                FOREIGN KEY (tag_id)
                    REFERENCES tag (id),
                FOREIGN KEY (image_id)
                    REFERENCES image (id) 
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
