CREATE TABLE messages (
  id BIGINT PRIMARY KEY,
  author BIGINT NOT NULL,
  content VARCHAR NOT NULL,
  channel BIGINT NOT NULL
)