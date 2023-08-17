-- Add migration script here
CREATE TABLE IF NOT EXISTS auth_users(
user_id uuid PRIMARY KEY,
username TEXT NOT NULL UNIQUE,
password TEXT NOT NULL
);
ALTER TABLE auth_users ADD COLUMN email TEXT;
ALTER TABLE auth_users ADD COLUMN email2 TEXT;
-- DROP TABLE auth_users;