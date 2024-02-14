
CREATE TYPE user_type AS ENUM (
  'guest',
  'user',
  'member',
  'agent',
  'superadmin',
  'admin'
);

CREATE TYPE status AS ENUM (
  'active',
  'inactive',
  'pending',
  'archived'
);

CREATE TABLE IF NOT EXISTS user_account(
    id uuid PRIMARY KEY,
    is_test_user boolean NOT NULL DEFAULT false,
    username TEXT NOT NULL UNIQUE,
    user_type user_type DEFAULT 'user'::user_type,
    international_dialing_code TEXT NOT NULL,
    mobile_no TEXT NOT NULL,
    email TEXT NOT NULL,
    display_name TEXT NOT NULL,
    user_account_number TEXT NOT NULL,
    alt_user_account_number TEXT NOT NULL,
    is_active status DEFAULT 'active'::status,
    created_by uuid not null,
    vectors jsonb not null,
    updated_by uuid,
    deleted_by uuid,
    created_on timestamptz NOT NULL,
    updated_on timestamptz,
    deleted_on timestamptz,
    is_deleted boolean not null DEFAULT false
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
  secret text,
  valid_upto timestamptz,
  is_active boolean not null DEFAULT false,
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



CREATE TYPE customer_type AS ENUM (
  'na',
  'buyer',
  'seller',
  'brand',
  'logistic_partner',
  'payment_aggregator',
  'virtual_operator',
  'external_partner'
);



CREATE TABLE role (
  id uuid PRIMARY KEY,
  role_name text NOT NULL,
  role_status status,
  created_at timestamptz,
  updated_at timestamptz,
  deleted_at timestamptz,
  created_by text not null,
  updated_by text,
  deleted_by text,
  is_deleted boolean DEFAULT false
);
