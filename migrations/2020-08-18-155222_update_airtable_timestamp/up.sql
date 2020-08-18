-- Your SQL goes here
ALTER TABLE airtable
DROP COLUMN created_time;
ALTER TABLE airtable
 ADD created_time timestamp; 
 