use crate::db::DbConnection;
use rocket::{
    fairing::{Fairing, Info, Kind},
    Rocket,
};

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
            name: "DbInit - Initialise artifact table",
            kind: Kind::Launch,
        }
    }

    fn on_launch(&self, _rocket: &Rocket) {
        use rusqlite::Connection;

        let con = Connection::open(crate::config::DATABASE_FILE).expect("Could not establish connection to the database to initialise the table!");
        // Create table
        con.execute(
            &DbConnection::table(
                "CREATE TABLE IF NOT EXISTS {table} (
                        id SERIAL PRIMARY KEY,
                        date timestamp without time zone NOT NULL,
                        hash varchar NOT NULL,
                        platform varchar NOT NULL,
                        channel varchar NOT NULL,
                        download_uri varchar NOT NULL
                    );",
            ),
            &[],
        )
        .expect("failed to create table!");
    }
}
