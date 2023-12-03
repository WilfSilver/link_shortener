CREATE TABLE prefixes (
  user_id VARCHAR NOT NULL,
  prefix TEXT NOT NULL,
  PRIMARY KEY(user_id, prefix)
);
