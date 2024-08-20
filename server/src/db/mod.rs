pub mod actions;
pub mod fs;

pub use self::fs::*;

use sqlx::{Executor, Pool};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseInitError {
    #[error("Failed to apply database migrations: {0:?}")]
    MigrationFailed(#[from] sqlx::migrate::MigrateError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DbType {
    Sqlite,
    Postgresql,
}

pub struct Db {
    pub(crate) pool: Pool<sqlx::any::Any>,
    #[allow(dead_code)]
    t: DbType,
}

impl Db {
    pub async fn new(database_file: &str) -> Result<Self, sqlx::migrate::MigrateError> {
        sqlx::any::install_default_drivers();

        let conection_args = format!("sqlite://{database_file}?mode=rwc");

        let t = if conection_args.starts_with("sqlite:") {
            DbType::Sqlite
        } else {
            DbType::Postgresql
        };

        let pool = sqlx::any::AnyPoolOptions::new()
            .max_connections(match t {
                DbType::Sqlite => 1,
                DbType::Postgresql => 10,
            })
            .connect(&conection_args)
            .await
            .expect("cannot connect to database, unable to start airshipper server");

        let x = pool.options();
        tracing::info!(?x, "pool options");
        if t == DbType::Sqlite {
            pool.execute("PRAGMA journal_mode = WAL;").await.unwrap();
            pool.execute("PRAGMA busy_timeout = 250;").await.unwrap();
        }

        sqlx::migrate!().run(&pool).await?;

        Ok(Self { pool, t })
    }
}
