CREATE TABLE invites (
  id SERIAL PRIMARY KEY,
  code VARCHAR NOT NULL,
  actionrole BIGINT,
  actionchannel BIGINT,
  used_count INT NOT NULL DEFAULT 0
);