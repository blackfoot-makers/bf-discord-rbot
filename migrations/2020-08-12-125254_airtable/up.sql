-- Your SQL goes here
CREATE TABLE airtable (
  id SERIAL PRIMARY KEY,
  aid VARCHAR NOT NULL,
  created_time BIGINT NOT NULL,
  content VARCHAR NOT NULL,
  triggered BOOLEAN NOT NULL
)