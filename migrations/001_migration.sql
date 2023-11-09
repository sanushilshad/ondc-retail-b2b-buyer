-- Add migration script here
CREATE TABLE IF NOT EXISTS user_account(
    id uuid PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL,
    created_by text,
    updated_by text,
    deleted_by text,
    is_deleted boolean,
    is_active boolean
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
  user_id uuid,
  auth_scope user_auth_identifier_scope,
  auth_identifier text,
  secret text,
  valid_upto timestamptz,
  verified boolean,
  created_at timestamptz,
  updated_at timestamptz,
  deleted_at timestamptz,
  created_by text,
  updated_by text,
  deleted_by text,
  is_deleted boolean
);

ALTER TABLE auth_mechanism ADD CONSTRAINT fk_auth_user FOREIGN KEY (user_id) REFERENCES user_account (id) ON DELETE CASCADE;
ALTER TABLE auth_mechanism ADD CONSTRAINT fk_auth_user_id_auth_scope UNIQUE (user_id, auth_scope);