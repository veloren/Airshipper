-- The table already got created by diesel before the migration to sqlx. Therefore we skip the creation if already existing.
CREATE TABLE IF NOT EXISTS artifacts (
    id integer PRIMARY KEY AUTOINCREMENT NOT NULL,
    build_id integer NOT NULL,
    date timestamp NOT NULL,
    hash varchar NOT NULL,
    author varchar NOT NULL,
    merged_by varchar NOT NULL,
    platform varchar NOT NULL,
    channel varchar NOT NULL,
    file_name varchar NOT NULL UNIQUE,
    download_uri varchar NOT NULL UNIQUE
);

DROP TABLE IF EXISTS __diesel_schema_migrations;