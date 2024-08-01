
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

CREATE TYPE "data_source" AS ENUM (
  'place_order',
  'ondc',
  'rapidor'
);

CREATE TABLE IF NOT EXISTS user_account(
    id uuid PRIMARY KEY,
    is_test_user BOOLEAN NOT NULL DEFAULT false,
    username TEXT NOT NULL UNIQUE,
    international_dialing_code TEXT NOT NULL,
    mobile_no TEXT NOT NULL,
    source data_source NOT NULL,
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
    is_deleted BOOLEAN NOT NULL DEFAULT false,
    subscriber_id TEXT NOT NULL
);

ALTER TABLE user_account ADD CONSTRAINT user_mobile_uq UNIQUE (mobile_no);
ALTER TABLE user_account ADD CONSTRAINT user_username_uq UNIQUE (username);
ALTER TABLE user_account ADD CONSTRAINT user_email_uq UNIQUE (email);

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

CREATE TYPE auth_context_type AS ENUM (
  'user_account',
  'business_account'
);

CREATE TABLE IF NOT EXISTS auth_mechanism (
  id uuid PRIMARY KEY,
  user_id uuid NOT NULL,
  auth_scope user_auth_identifier_scope NOT NULL,
  auth_identifier text NOT NULL,
  auth_context auth_context_type NOT NULL, 
  secret TEXT,
  valid_upto TIMESTAMPTZ,
  is_active status DEFAULT 'active'::status NOT NULL,
  created_on TIMESTAMPTZ,
  updated_on TIMESTAMPTZ,
  deleted_on TIMESTAMPTZ,
  created_by TEXT,
  updated_by TEXT,
  deleted_by TEXT,
  is_deleted BOOLEAN DEFAULT false
);

ALTER TABLE auth_mechanism ADD CONSTRAINT fk_auth_user FOREIGN KEY (user_id) REFERENCES user_account (id) ON DELETE CASCADE;
ALTER TABLE auth_mechanism ADD CONSTRAINT fk_auth_user_id_auth_scope UNIQUE (user_id, auth_scope, auth_context);



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



CREATE TABLE IF NOT EXISTS role (
  id uuid PRIMARY KEY,
  role_name TEXT NOT NULL,
  role_status status  NOT NULL,
  created_on TIMESTAMPTZ NOT NULL,
  updated_on TIMESTAMPTZ,
  deleted_on TIMESTAMPTZ,
  created_by uuid NOT NULL,
  updated_by uuid,
  deleted_by uuid,
  is_deleted BOOLEAN  NOT NULL DEFAULT false
);

ALTER TABLE role ADD CONSTRAINT unique_role_name UNIQUE (role_name);

CREATE TABLE IF NOT EXISTS user_role (
  id uuid PRIMARY KEY,
  user_id uuid NOT NULL,
  role_id uuid NOT NULL,
  created_on TIMESTAMPTZ NOT NULL,
  updated_on TIMESTAMPTZ,
  deleted_on TIMESTAMPTZ,
  created_by uuid NOT NULL,
  updated_by uuid,
  deleted_by uuid,
  is_deleted BOOLEAN NOT NULL DEFAULT false
);

ALTER TABLE user_role ADD CONSTRAINT fk_role_id FOREIGN KEY ("role_id") REFERENCES role ("id") ON DELETE CASCADE;
ALTER TABLE user_role ADD CONSTRAINT fk_user_id FOREIGN KEY ("user_id") REFERENCES user_account ("id") ON DELETE CASCADE;
ALTER TABLE user_role ADD CONSTRAINT user_role_pk UNIQUE (user_id, role_id);

CREATE TABLE IF NOT EXISTS permission (
  id uuid PRIMARY KEY,
  permission_name TEXT,
  permission_description TEXT,
  created_on TIMESTAMPTZ,
  updated_on TIMESTAMPTZ,
  deleted_on TIMESTAMPTZ,
  created_by uuid,
  updated_by uuid,
  deleted_by uuid,
  is_deleted BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE IF NOT EXISTS role_permission (
  id uuid PRIMARY KEY,
  role_id uuid,
  permission_id uuid,
  created_on TIMESTAMPTZ,
  updated_on TIMESTAMPTZ,
  deleted_on TIMESTAMPTZ,
  created_by uuid,
  updated_by uuid,
  deleted_by uuid,
  is_deleted BOOLEAN NOT NULL DEFAULT false
);


ALTER TABLE role_permission ADD CONSTRAINT "fk_permission_id" FOREIGN KEY ("permission_id") REFERENCES permission ("id") ON DELETE CASCADE;
ALTER TABLE role_permission ADD CONSTRAINT "fk_role_id" FOREIGN KEY ("role_id") REFERENCES role ("id") ON DELETE CASCADE;
ALTER TABLE permission ADD CONSTRAINT permission_name UNIQUE (permission_name);
ALTER TABLE role_permission ADD CONSTRAINT permission_role_id UNIQUE (permission_id, role_id);


CREATE TABLE IF NOT EXISTS communication (
  id uuid PRIMARY KEY,
  message TEXT NOT NULL,
  created_on TIMESTAMPTZ NOT NULL,
  created_by TEXT NOT NULL,
  media_list TEXT[]
);

CREATE TYPE kyc_status AS ENUM (
  'pending',
  'on-hold',
  'rejected',
  'completed'
);

CREATE TYPE trade_type as ENUM (
  'domestic',
  'export'
);

CREATE TYPE merchant_type as ENUM (
  'fpo',
  'manufacturer',
  'restaurant',
  'grocery',
  'mall',
  'supermart',
  'store',
  'other'
);

CREATE TABLE IF NOT EXISTS business_account (
  id uuid PRIMARY KEY,
  business_account_number TEXT NOT NULL,
  alt_business_account_number TEXT NOT NULL,
  company_name TEXT NOT NULL,
  vectors jsonb NOT NULL,
  default_vector_type TEXT NOT NULL,
  proofs jsonb NOT NULL,
  customer_type customer_type NOT NULL,
  merchant_type merchant_type NOT NULL,
  trade trade_type[],
  tags TEXT[],
  is_active status DEFAULT 'inactive'::status NOT NULL,
  source data_source NOT NULL,
  opening_time TIME,
  closing_time TIME,
  kyc_status kyc_status DEFAULT 'pending'::kyc_status NOT NULL,
  kyc_completed_by uuid,
  metadata_json jsonb,
  created_by  uuid NOT NULL,
  created_on TIMESTAMPTZ NOT NULL,
  updated_by uuid,
  updated_on TIMESTAMPTZ,
  deleted_by uuid,
  deleted_on TIMESTAMPTZ,
  is_deleted BOOLEAN NOT NULL DEFAULT false,
  is_test_account BOOLEAN NOT NULL DEFAULT false,
  subscriber_id TEXT NOT NULL

);

CREATE TABLE IF NOT EXISTS business_user_relationship (
  id uuid PRIMARY KEY,
  user_id uuid NOT NULL,
  business_id uuid NOT NULL,
  role_id uuid NOT NULL,
  verified BOOLEAN NOT NULL DEFAULT false,
  created_on TIMESTAMPTZ NOT NULL,
  updated_on TIMESTAMPTZ,
  created_by uuid NOT NULL,
  updated_by uuid
);


ALTER TABLE business_user_relationship ADD CONSTRAINT "fk_user_id" FOREIGN KEY ("user_id") REFERENCES user_account ("id") ON DELETE CASCADE;
ALTER TABLE business_user_relationship ADD CONSTRAINT "fk_business_id" FOREIGN KEY ("business_id") REFERENCES business_account ("id") ON DELETE CASCADE;
ALTER TABLE business_user_relationship ADD CONSTRAINT "fk_role_id" FOREIGN KEY ("role_id") REFERENCES role ("id") ON DELETE CASCADE;
ALTER TABLE business_user_relationship ADD CONSTRAINT user_business_role UNIQUE (user_id, business_id, role_id);


CREATE TYPE network_participant_type AS ENUM (
  'buyer',
  'seller'
);

CREATE TYPE ondc_np_fee_type AS ENUM (
  'percent',
  'amount'
);


CREATE TABLE IF NOT EXISTS registered_network_participant (
  id uuid PRIMARY KEY,
  name TEXT NOT NULL,
  code TEXT NOT NULL,
  subscriber_id TEXT NOT NULL,
  subscriber_uri TEXT NOT NULL,
  signing_key TEXT NOT NULL,
  network_participant_type network_participant_type NOT NULL,
  logo TEXT NOT NULL,
  long_description TEXT NOT NULL,
  short_description TEXT NOT NULL,
  unique_key_id TEXT NOT NULL,
  created_on TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  created_by uuid NOT NULL,
  fee_type ondc_np_fee_type DEFAULT 'percent'::ondc_np_fee_type NOT NULL,
  fee_value DECIMAL(20, 3) NOT NULL DEFAULT 0.00
);

ALTER TABLE registered_network_participant ADD CONSTRAINT registered_network_participant_constraint UNIQUE (subscriber_id, network_participant_type);





CREATE TYPE payment_type AS ENUM (
  'pre_paid',
  'cash_on_delivery',
  'credit'
);

CREATE TYPE product_search_type AS ENUM (
  'item',
  'fulfillment',
  'category',
  'city'
);

CREATE TYPE fulfillment_type AS ENUM (
  'delivery',
  'self_pickup'
);

CREATE TABLE IF NOT EXISTS search_request (
  id SERIAL NOT NULL PRIMARY KEY,
  message_id uuid NOT NULL,
  transaction_id uuid NOT NULL,
  business_id uuid NOT NULL,
  user_id uuid NOT NULL,
  device_id TEXT NOT NULL,
  created_on TIMESTAMPTZ NOT NULL,
  update_cache BOOLEAN DEFAULT false NOT NULL,
  query TEXT NOT NULL,
  payment_type payment_type,
  domain_category_code TEXT NOT NULL,
  search_type product_search_type NOT NULL,
  fulfillment_type fulfillment_type
);

CREATE TYPE ondc_network_participant_type AS ENUM (
  'BAP',
  'BPP'
);


CREATE TABLE IF NOT EXISTS network_participant (
  id uuid PRIMARY KEY,
  subscriber_id TEXT NOT NULL,
  br_id TEXT NOT NULL,
  subscriber_url TEXT NOT NULL,
  signing_public_key TEXT NOT NULL,
  domain TEXT NOT NULL,
  encr_public_key TEXT NOT NULL,
  type ondc_network_participant_type NOT NULL,
  uk_id TEXT NOT NULL,
  created_on TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
ALTER TABLE network_participant ADD CONSTRAINT network_participant_constraint UNIQUE (subscriber_id, type);



CREATE TABLE IF NOT EXISTS ondc_seller_product_info (
    id SERIAL NOT NULL PRIMARY KEY,
    seller_subscriber_id TEXT NOT NULL,
    provider_id TEXT NOT NULL,
    provider_name TEXT,
    item_id TEXT NOT NULL,
    item_code TEXT NOT NULL,
    item_name TEXT NOT NULL,
    tax_rate DECIMAL(5, 2) NOT NULL,
    images JSONB NOT NULL,
    mrp DECIMAL(20, 3) NOT NULL DEFAULT 0.0,
    unit_price DECIMAL(20, 3) NOT NULL DEFAULT 0.0,
    created_on TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
ALTER TABLE ondc_seller_product_info ADD CONSTRAINT ondc_seller_product_info_constraint UNIQUE (seller_subscriber_id, provider_id, item_id);

CREATE TABLE IF NOT EXISTS ondc_buyer_order_req (
    id SERIAL NOT NULL PRIMARY KEY,
    message_id uuid NOT NULL,
    transaction_id uuid NOT NULL,
    user_id uuid,
    business_id uuid,
    device_id TEXT NULL,
    action_type TEXT NOT NULL,
    request_payload JSONB NOT NULL,
    created_on TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);


CREATE TYPE commerce_data_type AS ENUM (
  'sale_order',
  'purchase_order'
);

CREATE TYPE  tax_type AS ENUM(
  'gst'
);

CREATE TYPE currency_code_type AS ENUM (
  'INR',
  'SGD',
  'AED',
  'GHS'
);

CREATE TYPE buyer_commerce_status AS ENUM(
  'quote_requested',
  'quote_accepted',
  'quote_rejected',
  'initialized',
  'created',
  'accepted',
  'in_progess',
  'completed',
  'cancelled'
);


CREATE TABLE IF NOT EXISTS buyer_commerce_data(
  id uuid PRIMARY KEY,
  urn TEXT,
  external_urn uuid NOT NULL,
  record_type commerce_data_type NOT NULL,
  record_status buyer_commerce_status NOT NULL,
  domain_category_code TEXT NOT NULL,
  buyer_id uuid NOT NULL,
  seller_id TEXT NOT NULL,
  buyer_name TEXT NOT NULL,
  seller_name TEXT,
  payment_data_json JSONB,
  source data_source NOT NULL,
  created_on timestamptz NOT NULL,
  updated_on timestamptz,
  deleted_on timestamptz,
  is_deleted BOOLEAN DEFAULT false,
  created_by uuid NOT NULL,
  grand_total DECIMAL(20, 3),
  buyer_chat_link TEXT,
  seller_chat_link TEXT,
  bpp_id TEXT NOT NULL,
  bpp_uri TEXT NOT NULL,
  bap_id TEXT NOT NULL,
  bap_uri TEXT NOT NULL,
  is_import BOOLEAN NOT NULL,
  quote_ttl TEXT NOT NULL,
  currency_code currency_code_type
);

ALTER TABLE buyer_commerce_data ADD CONSTRAINT buyer_commerce_data_uq UNIQUE (external_urn);

CREATE TABLE IF NOT EXISTS buyer_commerce_data_line(
  id uuid PRIMARY KEY,
  commerce_data_id uuid NOT NULL,
  item_id TEXT NOT NULL,
  item_image TEXT NOT NULL,
  item_name TEXT NOT NULL,
  item_code TEXT,
  qty DECIMAL(20, 2) NOT NULL,
  tax_rate DECIMAL(5, 2) NOT NULL DEFAULT 0.0,
  location_ids JSONB,
  fulfillment_ids JSONB,
  mrp DECIMAL(20, 3) NOT NULL DEFAULT 0.0,
  tax_value DECIMAL(20, 3) NOT NULL DEFAULT 0.0,
  unit_price DECIMAL(20, 3) NOT NULL DEFAULT 0.0,
  gross_total DECIMAL(20, 3) NOT NULL DEFAULT 0.0,
  available_qty DECIMAL(20, 2),
  discount_amount DECIMAL(20, 2) NOT NULL DEFAULT 0.0

);

ALTER TABLE buyer_commerce_data_line ADD CONSTRAINT commerce_data_fk FOREIGN KEY ("commerce_data_id") REFERENCES buyer_commerce_data ("id") ON DELETE CASCADE;
ALTER TABLE buyer_commerce_data_line ADD CONSTRAINT buyer_commerce_raw_data_uq UNIQUE (commerce_data_id, item_code);


CREATE TYPE commerce_fulfillment_status_type AS ENUM(
  'agent_assigned',
  'packed',
  'out_for_delivery',
  'order_picked_up',
  'searching_for_agent',
  'pending',
  'order_delivered',
  'cancelled'
);

CREATE TYPE fulfillment_servicability_status_type as ENUM(
  'serviceable',
  'non_serviceable'
);

CREATE TYPE inco_term_type AS ENUM (
    'EXW',
    'CIF',
    'FOB',
    'DAP',
    'DDP'
);

CREATE TABLE IF NOT EXISTS buyer_commerce_fulfillment_data(
  id uuid PRIMARY KEY,
  commerce_data_id uuid NOT NULL,
  fulfillment_id TEXT NOT NULL,
  fulfillment_type fulfillment_type NOT NULL,
  fulfillment_status commerce_fulfillment_status_type DEFAULT 'pending'::commerce_fulfillment_status_type NOT NULL,
  inco_terms inco_term_type,
  place_of_delivery TEXT,
  vectors JSONB,
  remark TEXT,
  provider_name TEXT,
  tat TEXT,
  tracking BOOLEAN,
  category TEXT,
  servicable_status fulfillment_servicability_status_type,
  pickup_data JSONB,
  drop_off_data JSONB,
  created_on TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

ALTER TABLE buyer_commerce_fulfillment_data ADD CONSTRAINT commerce_fulfillment_fk FOREIGN KEY ("commerce_data_id") REFERENCES buyer_commerce_data ("id") ON DELETE CASCADE;
ALTER TABLE buyer_commerce_fulfillment_data ADD CONSTRAINT buyer_commerce_fulfillment_data_uq UNIQUE (commerce_data_id, fulfillment_id);


CREATE TABLE IF NOT EXISTS buyer_commerce_fulfillment_data_line(
  id uuid PRIMARY KEY,
  commerce_fulfillment_id uuid NOT NULL,
  seller_id TEXT NOT NULL,
  item_code TEXT NOT NULL,
  item_count int NOT NULL
);

ALTER TABLE buyer_commerce_fulfillment_data_line ADD CONSTRAINT commerce_fulfillment_raw_fk FOREIGN KEY ("commerce_fulfillment_id") REFERENCES buyer_commerce_fulfillment_data ("id") ON DELETE CASCADE;
ALTER TABLE buyer_commerce_fulfillment_data_line ADD CONSTRAINT commerce_fulfillment_raw_data_uq UNIQUE (commerce_fulfillment_id, seller_id, item_code);



CREATE TABLE IF NOT EXISTS buyer_commerce_payment(
  id uuid PRIMARY KEY,
  commerce_data_id uuid NOT NULL,
  collected_by ondc_network_participant_type,
  payment_type payment_type
);
ALTER TABLE buyer_commerce_payment ADD CONSTRAINT buyer_commerce_payment_fk FOREIGN KEY ("commerce_data_id") REFERENCES buyer_commerce_data ("id") ON DELETE CASCADE;


CREATE TABLE IF NOT EXISTS  buyer_order_status_history(
  id uuid PRIMARY KEY,
  order_id TEXT NOT NULL,
  seller_id TEXT NOT NULL,
  fulfillment_id TEXT,
  status TEXT NOT NULL,
  created_on TEXT NOT NULL
);