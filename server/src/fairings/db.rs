use rocket::{
    fairing::{Fairing, Info, Kind},
    Rocket,
};

embed_migrations!();

/// Will initialise the database if necessary.
#[derive(Debug, Default)]
pub struct DbInit;

#[crate::async_trait]
impl Fairing for DbInit {
    fn info(&self) -> Info {
        Info {
            name: "DbInit - Run migrations",
            kind: Kind::Liftoff,
        }
    }

    async fn on_liftoff(&self, _: &Rocket<rocket::Orbit>) {
        use crate::diesel::Connection;
        let con = diesel::SqliteConnection::establish(
            crate::CONFIG
                .get_db_file_path()
                .as_path()
                .to_str()
                .expect("non-UTF8 path"),
        )
        .expect(
            "Could not establish connection to the database to initialise the table!",
        );
        // Run migrations
        tracing::debug!("running migrations");
        embedded_migrations::run(&con).expect("Failed to run migrations!");
    }
}
