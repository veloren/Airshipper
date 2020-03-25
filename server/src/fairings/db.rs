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

    fn on_launch(&self, rocket: &Rocket) {
        let con = DbConnection::get_one(&rocket)
            .expect("Could not establish connection to the database to initialise the table!");
        // Create table
        con.create_table().expect("Failed to create table!");
    }
}
