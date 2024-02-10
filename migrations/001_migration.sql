-- Add migration script here



-- CREATE TYPE masking_type AS ENUM ('na', 'encrypt', 'partialM_mask', 'full_mask');
-- CREATE TYPE vectors AS (
--     key VARCHAR(255),
--     value VARCHAR(255),
--     masking masking_type
-- );



CREATE TABLE IF NOT EXISTS user_account(
    id uuid PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    mobile_no TEXT NOT NULL,
    email TEXT NOT NULL,
    display_name TEXT NOT NULL,
    created_by uuid not null,
    updated_by uuid,
    deleted_by uuid,
    created_on timestamptz NOT NULL,
    updated_on timestamptz,
    deleted_on timestamptz,
    is_deleted boolean not null DEFAULT false,
    is_active boolean not null DEFAULT false,
    vectors jsonb not null
);
CREATE TYPE "user_auth_identifier_scope" AS ENUM (
  'otp',
  'password',
  'google',
  'facebook',
  'microsoft',
  'apple',
  'token',
  'auth_app',
  'qr',
  'email'
);

CREATE TABLE IF NOT EXISTS auth_mechanism (
  id uuid PRIMARY KEY,
  user_id uuid NOT NULL,
  auth_scope user_auth_identifier_scope NOT NULL,
  auth_identifier text NOT NULL,
  secret text NOT NULL,
  valid_upto timestamptz,
  verified boolean,
  created_at timestamptz,
  updated_at timestamptz,
  deleted_at timestamptz,
  created_by text,
  updated_by text,
  deleted_by text,
  is_deleted boolean DEFAULT false
);

ALTER TABLE auth_mechanism ADD CONSTRAINT fk_auth_user FOREIGN KEY (user_id) REFERENCES user_account (id) ON DELETE CASCADE;
ALTER TABLE auth_mechanism ADD CONSTRAINT fk_auth_user_id_auth_scope UNIQUE (user_id, auth_scope);



