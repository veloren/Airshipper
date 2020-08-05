use rocket::{
    fairing::{Fairing, Info, Kind},
    Cargo,
};

embed_migrations!();

/// Will initialise the database if necessary.
#[derive(Debug, Default)]
pub struct DbInit;

impl Fairing for DbInit {
    fn info(&self) -> Info {
        Info {
            name: "DbInit - Run migrations",
            kind: Kind::Launch,
        }
    }

    fn on_launch(&self, _: &Cargo) {
        use crate::diesel::Connection;
        let con = diesel::SqliteConnection::establish(crate::config::DATABASE_FILE)
            .expect("Could not establish connection to the database to initialise the table!");
        // Run migrations
        embedded_migrations::run(&con).expect("Failed to run migrations!");
    }
}
