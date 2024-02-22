
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
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
    is_test_user BOOLEAN NOT NULL DEFAULT false,
    username TEXT NOT NULL UNIQUE,
    international_dialing_code TEXT NOT NULL,
    mobile_no TEXT NOT NULL,
    email TEXT NOT NULL,
    display_name TEXT NOT NULL,
    user_account_number TEXT NOT NULL,
    alt_user_account_number TEXT NOT NULL,
    is_active status DEFAULT 'active'::status,
    created_by uuid NOT NULL,
    vectors jsonb NOT NULL,
    updated_by uuid,
    deleted_by uuid,
    created_on TIMESTAMPTZ NOT NULL,
    updated_on TIMESTAMPTZ,
    deleted_on TIMESTAMPTZ,
    is_deleted BOOLEAN NOT NULL DEFAULT false
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
  secret TEXT,
  valid_upto TIMESTAMPTZ,
  is_active BOOLEAN NOT NULL DEFAULT false,
  created_at TIMESTAMPTZ,
  updated_at TIMESTAMPTZ,
  deleted_at TIMESTAMPTZ,
  created_by TEXT,
  updated_by TEXT,
  deleted_by TEXT,
  is_deleted BOOLEAN DEFAULT false
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
  role_name TEXT NOT NULL,
  role_status status,
  created_at TIMESTAMPTZ,
  updated_at TIMESTAMPTZ,
  deleted_at TIMESTAMPTZ,
  created_by uuid NOT NULL,
  updated_by uuid,
  deleted_by uuid,
  is_deleted BOOLEAN DEFAULT false
);

ALTER TABLE role ADD CONSTRAINT unique_role_name UNIQUE (role_name);

CREATE TABLE user_role (
  id uuid PRIMARY KEY,
  user_id uuid NOT NULL,
  role_id uuid NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ,
  deleted_at TIMESTAMPTZ,
  created_by uuid NOT NULL,
  updated_by uuid,
  deleted_by uuid,
  is_deleted BOOLEAN NOT NULL
);

ALTER TABLE user_role ADD CONSTRAINT fk_role_id FOREIGN KEY ("role_id") REFERENCES role ("id") ON DELETE CASCADE;
ALTER TABLE user_role ADD CONSTRAINT fk_user_id FOREIGN KEY ("user_id") REFERENCES user_account ("id") ON DELETE CASCADE;
ALTER TABLE user_role ADD CONSTRAINT user_role_pk UNIQUE (user_id, role_id);

CREATE TABLE permission (
  id uuid PRIMARY KEY,
  permission_name TEXT,
  permission_description TEXT,
  created_at TIMESTAMPTZ,
  updated_at TIMESTAMPTZ,
  deleted_at TIMESTAMPTZ,
  created_by uuid,
  updated_by uuid,
  deleted_by uuid,
  is_deleted BOOLEAN
);

CREATE TABLE role_permission (
  id uuid PRIMARY KEY,
  role_id uuid,
  permission_id uuid,
  created_at TIMESTAMPTZ,
  updated_at TIMESTAMPTZ,
  deleted_at TIMESTAMPTZ,
  created_by uuid,
  updated_by uuid,
  deleted_by uuid,
  is_deleted BOOLEAN
);


ALTER TABLE role_permission ADD CONSTRAINT "fk_permission_id" FOREIGN KEY ("permission_id") REFERENCES permission ("id") ON DELETE CASCADE;
ALTER TABLE role_permission ADD CONSTRAINT "fk_role_id" FOREIGN KEY ("role_id") REFERENCES role ("id") ON DELETE CASCADE;
ALTER TABLE permission ADD CONSTRAINT permission_name UNIQUE (permission_name);
ALTER TABLE role_permission ADD CONSTRAINT permission_role_id UNIQUE (permission_id, role_id);
