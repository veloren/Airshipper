use rocket::{
    fairing::{Fairing, Info, Kind},
    tokio::{self, fs::File},
    Build, Rocket,
};

use crate::{db::DbConnection, CONFIG};

/// Will initialise the database if necessary.
#[derive(Debug, Default)]
pub struct DbInit;

#[rocket::async_trait]
impl Fairing for DbInit {
    fn info(&self) -> Info {
        Info {
            name: "DbInit - migrations & pool",
            kind: Kind::Ignite,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> rocket::fairing::Result {
        // Create Db
        if tokio::fs::metadata(&CONFIG.db_path).await.is_err() {
            File::create(&CONFIG.db_path)
                .await
                .expect("Failed to create db!");
        }

        // Run migrations
        let pool = sqlx::SqlitePool::connect(&CONFIG.db_path)
            .await
            .expect("Failed to connect sqlite db.");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations!");

        // Add pool to managed state
        let rocket = rocket.manage(DbConnection::new(pool));

        Ok(rocket)
    }
}
