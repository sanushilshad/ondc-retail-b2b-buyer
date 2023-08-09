-- Add migration script here
CREATE TABLE auth_users(
user_id uuid PRIMARY KEY,
username TEXT NOT NULL UNIQUE,
password TEXT NOT NULL
);