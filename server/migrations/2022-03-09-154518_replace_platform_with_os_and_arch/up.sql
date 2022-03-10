ALTER TABLE artifacts RENAME COLUMN platform TO os;
ALTER TABLE artifacts ADD COLUMN arch varchar NOT NULL DEFAULT "";