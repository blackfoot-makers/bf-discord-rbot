-- This file should undo anything in `up.sql`
ALTER TABLE airtable
DROP COLUMN created_time;
ALTER TABLE airtable
 ADD created_time BIGINT NOT NULL;