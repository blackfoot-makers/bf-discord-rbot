-- Your SQL goes here
CREATE TABLE events (
  id SERIAL PRIMARY KEY,
  author BIGINT NOT NULL,
  content VARCHAR NOT NULL,
  channel BIGINT NOT NULL,
  triger_date TIMESTAMP NOT NULL
)