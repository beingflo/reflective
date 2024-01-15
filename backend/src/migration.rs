use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};

pub fn apply_migrations(connection: &mut Connection) {
    let migrations = Migrations::new(vec![M::up(
        "CREATE TABLE users(username TEXT NOT NULL, password TEXT NOT NULL);",
    )]);

    migrations.to_latest(connection).unwrap();
}
