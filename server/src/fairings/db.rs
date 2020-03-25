use rocket::{
    fairing::{Fairing, Info, Kind},
    Rocket,
};

embed_migrations!();

/// Will initialise the database if necessary.
pub struct DbInit;

impl Default for DbInit {
    fn default() -> Self {
        Self {}
    }
}

impl Fairing for DbInit {
    fn info(&self) -> Info {
        Info {
            name: "DbInit - Run migrations",
            kind: Kind::Launch,
        }
    }

    fn on_launch(&self, _: &Rocket) {
        use crate::diesel::Connection;
        let con = diesel::SqliteConnection::establish(crate::config::DATABASE_FILE)
            .expect("Could not establish connection to the database to initialise the table!");
        // Run migrations
        embedded_migrations::run(&con).expect("Failed to run migrations!");
    }
}
