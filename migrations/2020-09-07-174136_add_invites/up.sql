CREATE TABLE invites (
  id SERIAL PRIMARY KEY,
  code VARCHAR,
  actionrole INT NOT NULL DEFAULT 0,
  actionchannel INT NOT NULL DEFAULT 0,
  used_count INT NOT NULL DEFAULT 0
);