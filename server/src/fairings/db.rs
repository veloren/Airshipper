use rocket::fairing::{Fairing, Info, Kind};
use rocket::Rocket;

use crate::db::DbConnection;

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

    #[cfg(not(test))]
    fn on_launch(&self, _rocket: &Rocket) {
        use postgres::{Connection, TlsMode};

        let con = Connection::connect(&*crate::CONFIG.database_address, TlsMode::None)
            .expect("Could not establish connection to the database to initialise the table!");
        // Create table
        con.execute(
            &DbConnection::table(
                "CREATE TABLE IF NOT EXISTS {} (
                        id SERIAL PRIMARY KEY,
                        date timestamp without time zone NOT NULL,
                        hash varchar NOT NULL,
                        author varchar NOT NULL,
                        merged_by varchar NOT NULL,
                        platform varchar NOT NULL,
                        channel varchar NOT NULL,
                        download_path varchar NOT NULL
                    );",
            ),
            &[],
        )
        .expect("failed to create table!");
    }

    /// Will init the db and populate with test data.
    #[cfg(test)]
    fn on_launch(&self, _rocket: &Rocket) {
        use postgres::{Connection, TlsMode};

        let con = Connection::connect(&*crate::CONFIG.database_address, TlsMode::None)
            .expect("Could not establish connection to the database to initialise the table!");
        // Create table
        con.execute(
            &DbConnection::table(
                "CREATE TABLE IF NOT EXISTS {} (
                        id integer PRIMARY KEY,
                        date timestamp without time zone NOT NULL,
                        hash varchar NOT NULL,
                        author varchar NOT NULL,
                        merged_by varchar NOT NULL,
                        platform varchar NOT NULL,
                        channel varchar NOT NULL,
                        download_path varchar NOT NULL
                    );",
            ),
            &[],
        )
        .expect("failed to create table!");

        // TODO: Populate with dummy data
    }
}
