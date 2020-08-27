-- Your SQL goes here
CREATE TABLE projects (
  id SERIAL PRIMARY KEY,
  message_id BIGINT NOT NULL,
  channel_id BIGINT NOT NULL,
  codex VARCHAR NOT NULL DEFAULT '',
  client VARCHAR NOT NULL DEFAULT '',
  lead VARCHAR NOT NULL DEFAULT '',
  deadline VARCHAR NOT NULL DEFAULT '',
  description VARCHAR NOT NULL DEFAULT '',
  contexte VARCHAR NOT NULL DEFAULT '',
  created_at timestamp NOT NULL DEFAULT NOW()
);