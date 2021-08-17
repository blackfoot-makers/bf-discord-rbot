CREATE TABLE messages_edits (
  id SERIAL PRIMARY KEY,
  author BIGINT NOT NULL,
  content VARCHAR NOT NULL,
  channel BIGINT NOT NULL,
  date TIMESTAMP,

  parrent_message_id BIGINT NOT NULL,
  FOREIGN KEY(parrent_message_id) REFERENCES messages(id)
)