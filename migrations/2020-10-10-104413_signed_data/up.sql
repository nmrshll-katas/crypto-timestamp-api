CREATE TABLE signed_data (
  id BIGSERIAL PRIMARY KEY,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  data_hash_b64 VARCHAR(128) NOT NULL UNIQUE
);

CREATE INDEX idx_signed_data_hash ON signed_data (data_hash_b64);