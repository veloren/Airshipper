ALTER TABLE artifacts RENAME COLUMN os TO platform;
ALTER TABLE artifacts DROP COLUMN arch;