
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION postgis;


CREATE TYPE "data_source_type" AS ENUM (
  'place_order',
  'ondc',
  'rapidor'
);







CREATE TYPE network_participant_type AS ENUM (
  'buyer',
  'seller'
);



CREATE TYPE ondc_np_fee_type AS ENUM (
  'percent',
  'amount'
);

CREATE TYPE payment_settlement_phase AS ENUM (
  'sale_amount'
);

CREATE TYPE payment_settlement_type AS ENUM (
  'neft'
);

CREATE TYPE ondc_network_participant_type AS ENUM (
  'BAP',
  'BPP'
);


CREATE TABLE IF NOT EXISTS registered_network_participant (
  id SERIAL PRIMARY KEY,
  name TEXT NOT NULL,
  code TEXT NOT NULL,
  subscriber_id TEXT NOT NULL,
  subscriber_uri TEXT NOT NULL,
  signing_key TEXT NOT NULL,
  network_participant_type ondc_network_participant_type NOT NULL,
  logo TEXT NOT NULL,
  long_description TEXT NOT NULL,
  short_description TEXT NOT NULL,
  unique_key_id TEXT NOT NULL,
  created_on TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  created_by uuid NOT NULL,
  fee_type ondc_np_fee_type DEFAULT 'percent'::ondc_np_fee_type NOT NULL,
  fee_value DECIMAL(20, 3) NOT NULL DEFAULT 0.00,
  settlement_phase payment_settlement_phase NOT NULL,
  settlement_type payment_settlement_type NOT NULL,
  bank_account_no TEXT NOT NULL,
  bank_ifsc_code TEXT NOT NULL,
  bank_beneficiary_name TEXT NOT NULL,
  bank_name TEXT NOT NULL,
  observability_token TEXT,

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
CREATE INDEX idx_search_request_message_txn ON search_request(message_id, transaction_id);
CREATE INDEX search_request_created_on_idx ON search_request (created_on);


CREATE TYPE payment_collected_by_type AS ENUM(
    'BAP',
    'BPP',
    'buyer'
);


CREATE TABLE IF NOT EXISTS network_participant (
  id SERIAL NOT NULL PRIMARY KEY,
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


CREATE TABLE IF NOT EXISTS ondc_buyer_order_req (
    id SERIAL NOT NULL PRIMARY KEY,
    message_id uuid NOT NULL,
    transaction_id uuid NOT NULL,
    user_id uuid NOT NULL,
    business_id uuid NOT NULL,
    device_id TEXT NULL,
    action_type TEXT NOT NULL,
    request_payload JSONB NOT NULL,
    created_on TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_ondc_buyer_order_req_action_message_txn ON ondc_buyer_order_req(action_type, message_id, transaction_id);


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

CREATE TYPE commerce_status AS ENUM(
  'quote_requested',
  'quote_accepted',
  'quote_rejected',
  'initialized',
  'created',
  'accepted',
  'in_progress',
  'completed',
  'cancelled'
);
CREATE TYPE country_code_type AS ENUM (
    'AFG',
    'ALA',
    'ALB',
    'DZA',
    'ASM',
    'AND',
    'AGO',
    'AIA',
    'ATA',
    'ATG',
    'ARG',
    'ARM',
    'ABW',
    'AUS',
    'AUT',
    'AZE',
    'BHS',
    'BHR',
    'BGD',
    'BRB',
    'BLR',
    'BEL',
    'BLZ',
    'BEN',
    'BMU',
    'BTN',
    'BOL',
    'BES',
    'BIH',
    'BWA',
    'BVT',
    'BRA',
    'IOT',
    'BRN',
    'BGR',
    'BFA',
    'BDI',
    'CPV',
    'KHM',
    'CMR',
    'CAN',
    'CYM',
    'CAF',
    'TCD',
    'CHL',
    'CHN',
    'CXR',
    'CCK',
    'COL',
    'COM',
    'COG',
    'COD',
    'COK',
    'CRI',
    'CIV',
    'HRV',
    'CUB',
    'CUW',
    'CYP',
    'CZE',
    'DNK',
    'DJI',
    'DMA',
    'DOM',
    'ECU',
    'EGY',
    'SLV',
    'GNQ',
    'ERI',
    'EST',
    'SWZ',
    'ETH',
    'FLK',
    'FRO',
    'FJI',
    'FIN',
    'FRA',
    'GUF',
    'PYF',
    'ATF',
    'GAB',
    'GMB',
    'GEO',
    'DEU',
    'GHA',
    'GIB',
    'GRC',
    'GRL',
    'GRD',
    'GLP',
    'GUM',
    'GTM',
    'GGY',
    'GIN',
    'GNB',
    'GUY',
    'HTI',
    'HMD',
    'VAT',
    'HND',
    'HKG',
    'HUN',
    'ISL',
    'IND',
    'IDN',
    'IRN',
    'IRQ',
    'IRL',
    'IMN',
    'ISR',
    'ITA',
    'JAM',
    'JPN',
    'JEY',
    'JOR',
    'KAZ',
    'KEN',
    'KIR',
    'PRK',
    'KOR',
    'KWT',
    'KGZ',
    'LAO',
    'LVA',
    'LBN',
    'LSO',
    'LBR',
    'LBY',
    'LIE',
    'LTU',
    'LUX',
    'MAC',
    'MDG',
    'MWI',
    'MYS',
    'MDV',
    'MLI',
    'MLT',
    'MHL',
    'MTQ',
    'MRT',
    'MUS',
    'MYT',
    'MEX',
    'FSM',
    'MDA',
    'MCO',
    'MNG',
    'MNE',
    'MSR',
    'MAR',
    'MOZ',
    'MMR',
    'NAM',
    'NRU',
    'NPL',
    'NLD',
    'NCL',
    'NZL',
    'NIC',
    'NER',
    'NGA',
    'NIU',
    'NFK',
    'MKD',
    'MNP',
    'NOR',
    'OMN',
    'PAK',
    'PLW',
    'PSE',
    'PAN',
    'PNG',
    'PRY',
    'PER',
    'PHL',
    'PCN',
    'POL',
    'PRT',
    'PRI',
    'QAT',
    'ROU',
    'RUS',
    'RWA',
    'REU',
    'BLM',
    'SHN',
    'KNA',
    'LCA',
    'MAF',
    'SPM',
    'VCT',
    'WSM',
    'SMR',
    'STP',
    'SAU',
    'SEN',
    'SRB',
    'SYC',
    'SLE',
    'SGP',
    'SXM',
    'SVK',
    'SVN',
    'SLB',
    'SOM',
    'ZAF',
    'SGS',
    'SSD',
    'ESP',
    'LKA',
    'SDN',
    'SUR',
    'SJM',
    'SWE',
    'CHE',
    'SYR',
    'TWN',
    'TJK',
    'TZA',
    'THA',
    'TLS',
    'TGO',
    'TKL',
    'TON',
    'TTO',
    'TUN',
    'TUR',
    'TKM',
    'TCA',
    'TUV',
    'UGA',
    'UKR',
    'ARE',
    'GBR',
    'USA',
    'URY',
    'UZB',
    'VUT',
    'VEN',
    'VNM',
    'WLF',
    'ESH',
    'YEM',
    'ZMB',
    'ZWE'
);

CREATE TYPE domain_category_type AS ENUM (
    'RET10', 
    'RET12',
    'RET13',
    'RET14',
    'RET15',
    'RET16',
    'RET1A',
    'RET1B',
    'RET1C'
);


CREATE TABLE IF NOT EXISTS commerce_data(
  id uuid PRIMARY KEY,
  urn TEXT NOT NULL,
  external_urn uuid NOT NULL,
  record_type commerce_data_type NOT NULL,
  record_status commerce_status NOT NULL,
  domain_category_code domain_category_type NOT NULL,
  buyer_id uuid NOT NULL,
  seller_id TEXT NOT NULL,
  buyer_name TEXT NOT NULL,
  seller_name TEXT,
  source data_source_type NOT NULL,
  created_on timestamptz NOT NULL,
  updated_on timestamptz,
  deleted_on timestamptz,
  is_deleted BOOLEAN NOT NULL DEFAULT false,
  created_by uuid NOT NULL,
  updated_by TEXT,
  deleted_by uuid,
  refund_grand_total DECIMAL(20, 3),
  grand_total DECIMAL(20, 3),
  documents JSONB,
  buyer_chat_link TEXT,
  seller_chat_link TEXT,
  bpp_id TEXT NOT NULL,
  bpp_uri TEXT NOT NULL,
  bap_id TEXT NOT NULL,
  bap_uri TEXT NOT NULL,
  quote_ttl TEXT NOT NULL,
  currency_code currency_code_type NOT NULL,
  city_code TEXT NOT NULL,
  country_code country_code_type NOT NULL,
  billing JSONB,
  bpp_terms JSONB,
  cancellation_terms JSONB
);

ALTER TABLE commerce_data ADD CONSTRAINT commerce_data_uq UNIQUE (external_urn);
CREATE INDEX commerce_created_on_idx ON commerce_data (created_on);


CREATE TABLE IF NOT EXISTS commerce_data_line(
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
  refunded_tax_value DECIMAL(20, 3),
  refunded_discount_amount DECIMAL(20, 2),
  refunded_gross_total DECIMAL(20, 2),
  unit_price DECIMAL(20, 3) NOT NULL DEFAULT 0.0,
  gross_total DECIMAL(20, 3) NOT NULL DEFAULT 0.0,
  available_qty DECIMAL(20, 2),
  discount_amount DECIMAL(20, 2) NOT NULL DEFAULT 0.0,
  item_req TEXT,
  packaging_req TEXT
);

ALTER TABLE commerce_data_line ADD CONSTRAINT commerce_data_fk FOREIGN KEY ("commerce_data_id") REFERENCES commerce_data ("id") ON DELETE CASCADE;
ALTER TABLE commerce_data_line ADD CONSTRAINT commerce_raw_data_uq UNIQUE (commerce_data_id, item_code);
CREATE INDEX commerce_data_line_id_idx ON commerce_data_line (commerce_data_id);

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

CREATE TYPE fulfillment_servicability_status as ENUM(
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

CREATE TYPE fulfillment_category_type AS ENUM (
  'standard_delivery',
  'express_delivery',
  'self_pickup'
);

CREATE TYPE trade_type  AS ENUM (
  'domestic',
  'import'
);

CREATE TABLE IF NOT EXISTS commerce_fulfillment_data(
  id uuid PRIMARY KEY,
  commerce_data_id uuid NOT NULL,
  fulfillment_id TEXT NOT NULL,
  fulfillment_type fulfillment_type NOT NULL,
  packaging_charge DECIMAL(20, 3) NOT NULL DEFAULT 0.0,
  delivery_charge DECIMAL(20, 3) NOT NULL DEFAULT 0.0,
  convenience_fee DECIMAL(20, 3) NOT NULL DEFAULT 0.0,
  refunded_convenience_fee DECIMAL(20, 3),
  refunded_delivery_charge DECIMAL(20, 3),
  refunded_packaging_charge DECIMAL(20, 3),
  fulfillment_status commerce_fulfillment_status_type DEFAULT 'pending'::commerce_fulfillment_status_type NOT NULL,
  inco_terms inco_term_type,
  place_of_delivery TEXT,
  vectors JSONB,
  remark TEXT,
  provider_name TEXT,
  tat TEXT,
  tracking BOOLEAN,
  trade_type trade_type,
  category fulfillment_category_type,
  servicable_status fulfillment_servicability_status,
  pickup_data JSONB NOT NULL,
  drop_off_data JSONB,
  created_on TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP

);

ALTER TABLE commerce_fulfillment_data ADD CONSTRAINT commerce_fulfillment_fk FOREIGN KEY ("commerce_data_id") REFERENCES commerce_data ("id") ON DELETE CASCADE;
ALTER TABLE commerce_fulfillment_data ADD CONSTRAINT commerce_fulfillment_data_uq UNIQUE (commerce_data_id, fulfillment_id);
CREATE INDEX commerce_fulfillment_data_id_idx ON commerce_fulfillment_data (commerce_data_id);

CREATE TABLE IF NOT EXISTS commerce_fulfillment_data_line(
  id uuid PRIMARY KEY,
  commerce_fulfillment_id uuid NOT NULL,
  seller_id TEXT NOT NULL,
  item_code TEXT NOT NULL,
  item_count int NOT NULL
);

ALTER TABLE commerce_fulfillment_data_line ADD CONSTRAINT commerce_fulfillment_raw_fk FOREIGN KEY ("commerce_fulfillment_id") REFERENCES commerce_fulfillment_data ("id") ON DELETE CASCADE;
ALTER TABLE commerce_fulfillment_data_line ADD CONSTRAINT commerce_fulfillment_raw_data_uq UNIQUE (commerce_fulfillment_id, seller_id, item_code);


CREATE TYPE settlement_basis_type AS ENUM (
  'return_window_expiry',
  'shipment',
  'delivery'
);


CREATE TYPE payment_status AS ENUM (
  'paid',
  'not_paid',
  'pending'
);

CREATE TABLE IF NOT EXISTS commerce_payment_data(
  id uuid PRIMARY KEY,
  payment_id TEXT,
  commerce_data_id uuid NOT NULL,
  collected_by payment_collected_by_type,
  payment_type payment_type NOT NULL,
  payment_order_id TEXT,
  payment_status payment_status NOT NULL,
  payment_amount DECIMAL(20, 3),
  transaction_id TEXT,
  buyer_fee_type ondc_np_fee_type,
  buyer_fee_amount DECIMAL(20, 3),
  settlement_window TEXT,
  withholding_amount DECIMAL(20, 3),
  settlement_basis settlement_basis_type,
  settlement_details JSONB,
  seller_payment_detail JSONB
);
ALTER TABLE commerce_payment_data ADD CONSTRAINT commerce_payment_fk FOREIGN KEY ("commerce_data_id") REFERENCES commerce_data ("id") ON DELETE CASCADE;
CREATE INDEX commerce_payment_data_id_idx ON commerce_payment_data (commerce_data_id);


CREATE TABLE IF NOT EXISTS ondc_provider_info (
    id uuid PRIMARY KEY,
    seller_subscriber_id TEXT NOT NULL,
    provider_id TEXT NOT NULL,
    provider_name TEXT,
    created_on TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_on TIMESTAMPTZ
);
ALTER TABLE ondc_provider_info ADD CONSTRAINT ondc_provider_info_constraint UNIQUE (seller_subscriber_id, provider_id);

CREATE TABLE IF NOT EXISTS ondc_provider_location_info(
    id uuid PRIMARY KEY,
    seller_subscriber_id TEXT NOT NULL,
    provider_id TEXT NOT NULL,
    location_id TEXT NOT NULL,
    latitude DECIMAL(9, 6) NOT NULL,
    longitude DECIMAL(9, 6) NOT NULL,
    address TEXT NOT NULL,
    city_code TEXT NOT NULL,
    city_name TEXT NOT NULL,
    state_code TEXT NOT NULL,
    state_name TEXT,
    country_code country_code_type NOT NULL,
    country_name TEXT,
    area_code TEXT NOT NULL,
    created_on TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_on TIMESTAMPTZ
);

ALTER TABLE ondc_provider_location_info ADD CONSTRAINT ondc_seller_location_constraint UNIQUE (seller_subscriber_id, provider_id, location_id);

CREATE TABLE IF NOT EXISTS ondc_provider_product_info (
    id uuid PRIMARY KEY,
    seller_subscriber_id TEXT NOT NULL,
    currency_code currency_code_type NOT NULL,
    country_code country_code_type NOT NULL,
    provider_id TEXT NOT NULL,
    provider_name TEXT,
    item_id TEXT NOT NULL,
    item_code TEXT NOT NULL,
    item_name TEXT NOT NULL,
    tax_rate DECIMAL(5, 2) NOT NULL,
    images JSONB NOT NULL,
    mrp DECIMAL(20, 3) NOT NULL DEFAULT 0.0,
    unit_price_with_tax DECIMAL(20, 3) NOT NULL DEFAULT 0.0,
    unit_price_without_tax DECIMAL(20, 3) NOT NULL DEFAULT 0.0,
    price_slab JSONB,
    created_on TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_on TIMESTAMPTZ
);
ALTER TABLE ondc_provider_product_info ADD CONSTRAINT ondc_provider_product_info_constraint UNIQUE (seller_subscriber_id, country_code, provider_id, item_id);
ALTER TABLE ondc_provider_location_info ADD FOREIGN KEY (seller_subscriber_id, provider_id) REFERENCES ondc_provider_info (seller_subscriber_id, provider_id) ON DELETE CASCADE;
ALTER TABLE ondc_provider_product_info ADD FOREIGN KEY (seller_subscriber_id, provider_id) REFERENCES ondc_provider_info (seller_subscriber_id, provider_id) ON DELETE CASCADE;

CREATE TYPE series_type AS ENUM (
  'order'
);


CREATE TABLE IF NOT EXISTS series_no_generator(
  id SERIAL NOT NULL PRIMARY KEY,
  subscriber_id TEXT NOT NULL,
  series_type series_type NOT NULL,
  prefix TEXT NOT NULL,
  series_no BIGINT NOT NULL
);

ALTER TABLE series_no_generator ADD CONSTRAINT series_no_generator_constraint UNIQUE (subscriber_id, series_type, prefix);



CREATE TABLE IF NOT EXISTS network_participant_cache(
  id uuid PRIMARY KEY,
  subscriber_id TEXT NOT NULL,
  name TEXT NOT NULL,
  short_desc TEXT NOT NULL,
  long_desc TEXT NOT NULL,
  images JSONB NOT NULL,
  created_on TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

ALTER TABLE network_participant_cache ADD CONSTRAINT network_participant_cache_constraint UNIQUE (subscriber_id);


CREATE TABLE IF NOT EXISTS provider_cache(
  id uuid PRIMARY KEY,
  provider_id TEXT NOT NULL,
  network_participant_cache_id uuid NOT NULL,
  name TEXT NOT NULL,
  code TEXT NOT NULL,
  short_desc TEXT NOT NULL,
  long_desc TEXT NOT NULL,
  images JSONB NOT NULL,
  rating real,
  ttl TEXT NOT NULL,
  credentials JSONB NOT NULL,
  contact JSONB NOT NULL,
  terms JSONB,
  identifications JSONB,
  created_on TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_on TIMESTAMPTZ
);
ALTER TABLE provider_cache ADD CONSTRAINT provider_cache_constraint UNIQUE (network_participant_cache_id, provider_id);
ALTER TABLE provider_cache ADD CONSTRAINT provider_cache_fk FOREIGN KEY ("network_participant_cache_id") REFERENCES network_participant_cache("id") ON DELETE CASCADE;



CREATE TABLE IF NOT EXISTS provider_location_cache(
    id uuid PRIMARY KEY,
    provider_cache_id uuid NOT NULL,
    location_id TEXT NOT NULL,
    latitude DECIMAL(9, 6) NOT NULL,
    longitude DECIMAL(9, 6) NOT NULL,
    location GEOMETRY(POINT, 3857) NOT NULL,
    address TEXT NOT NULL,
    city_code TEXT NOT NULL,
    city_name TEXT NOT NULL,
    state_code TEXT NOT NULL,
    state_name TEXT,
    country_code country_code_type NOT NULL,
    country_name TEXT,
    area_code TEXT NOT NULL,
    created_on TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_on TIMESTAMPTZ
);
ALTER TABLE provider_location_cache ADD CONSTRAINT enforce_srid CHECK (ST_SRID(location) = 3857);
CREATE INDEX provider_location_cache_idx ON provider_location_cache USING GIST (location);

ALTER TABLE provider_location_cache ADD CONSTRAINT provider_location_cache_constraint UNIQUE (provider_cache_id, location_id);
ALTER TABLE provider_location_cache ADD CONSTRAINT provider_location_cache_constraint_fk FOREIGN KEY ("provider_cache_id") REFERENCES provider_cache("id") ON DELETE CASCADE;




CREATE TABLE IF NOT EXISTS provider_servicability_geo_json_cache(
    id uuid PRIMARY KEY,
    provider_location_cache_id uuid NOT NULL,
    domain_code domain_category_type NOT NULL,
    geom GEOMETRY(Geometry, 4326) NOT NULL,
    category_code TEXT,
    coordinates JSONB NOT NULL,
    created_on TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

ALTER TABLE provider_servicability_geo_json_cache ADD CONSTRAINT provider_servicability_geo_json_cache_cache_constraint UNIQUE NULLS NOT DISTINCT (provider_location_cache_id, domain_code, category_code, geom);
ALTER TABLE provider_servicability_geo_json_cache ADD CONSTRAINT provider_servicability_geo_json_cache_fk FOREIGN KEY ("provider_location_cache_id") REFERENCES provider_location_cache("id") ON DELETE CASCADE;
ALTER TABLE provider_servicability_geo_json_cache ADD CONSTRAINT enforce_srid CHECK (ST_SRID(geom) = 4326);
CREATE INDEX servicability_geo_json_cache ON provider_servicability_geo_json_cache USING GIST (geom);


CREATE TABLE IF NOT EXISTS provider_servicability_hyperlocal_cache (
    id uuid PRIMARY KEY,
    provider_location_cache_id uuid NOT NULL,
    domain_code domain_category_type NOT NULL,
    category_code TEXT,
    radius DOUBLE PRECISION NOT NULL,
    created_on TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

ALTER TABLE provider_servicability_hyperlocal_cache ADD CONSTRAINT provider_servicability_hyperlocal_cache_constraint UNIQUE NULLS NOT DISTINCT (provider_location_cache_id, domain_code, category_code);
ALTER TABLE provider_servicability_hyperlocal_cache ADD CONSTRAINT provider_servicability_hyperlocal_cache_fk FOREIGN KEY ("provider_location_cache_id") REFERENCES provider_location_cache("id") ON DELETE CASCADE;


CREATE TABLE IF NOT EXISTS provider_servicability_country_cache (
    id uuid PRIMARY KEY,
    provider_location_cache_id uuid NOT NULL,
    domain_code domain_category_type NOT NULL,
    category_code TEXT,
    country_code country_code_type NOT NULL,
    created_on TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

ALTER TABLE provider_servicability_country_cache ADD CONSTRAINT provider_servicability_country_cache_constraint UNIQUE NULLS NOT DISTINCT (provider_location_cache_id, domain_code, category_code, country_code);
ALTER TABLE provider_servicability_country_cache ADD CONSTRAINT provider_servicability_country_cache_fk FOREIGN KEY ("provider_location_cache_id") REFERENCES provider_location_cache("id") ON DELETE CASCADE;


CREATE TABLE IF NOT EXISTS provider_servicability_intercity_cache (
    id uuid PRIMARY KEY,
    provider_location_cache_id uuid NOT NULL,
    domain_code domain_category_type NOT NULL,
    category_code TEXT,
    pincode TEXT NOT NULL,
    created_on TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

ALTER TABLE provider_servicability_intercity_cache ADD CONSTRAINT provider_servicability_intercity_cache_constraint UNIQUE NULLS NOT DISTINCT (provider_location_cache_id, domain_code, category_code, pincode);
ALTER TABLE provider_servicability_intercity_cache ADD CONSTRAINT provider_servicability_intercity_cache_fk FOREIGN KEY ("provider_location_cache_id") REFERENCES provider_location_cache("id") ON DELETE CASCADE;


CREATE TABLE IF NOT EXISTS provider_offer_cache (
    id uuid PRIMARY KEY,
    provider_cache_id uuid NOT NULL,
    offer_id TEXT NOT NULL,
    name TEXT NOT NULL,
    offer_code TEXT NOT NULL,
    short_desc TEXT NOT NULL,
    long_desc TEXT NOT NULL,
    images JSONB NOT NULL,
    domain_code domain_category_type NOT NULL,
    location_ids JSONB NOT NULL,
    category_ids JSONB NOT NULL,
    item_ids JSONB NOT NULL,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    created_on TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_on TIMESTAMPTZ
);


ALTER TABLE provider_offer_cache ADD CONSTRAINT provider_offer_cache_constraint UNIQUE NULLS NOT DISTINCT (provider_cache_id, offer_id);
ALTER TABLE provider_offer_cache ADD CONSTRAINT provider_offer_cache_fk FOREIGN KEY ("provider_cache_id") REFERENCES provider_cache("id") ON DELETE CASCADE;


CREATE TABLE IF NOT EXISTS provider_item_variant_cache(
    id uuid PRIMARY KEY,
    provider_cache_id uuid NOT NULL,
    variant_id TEXT NOT NULL,
    variant_name TEXT NOT NULL,
    attributes JSONB NOT NULL,
    created_on TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_on TIMESTAMPTZ
);
ALTER TABLE provider_item_variant_cache ADD CONSTRAINT item_variant_cache_constraint UNIQUE NULLS NOT DISTINCT (provider_cache_id, variant_id);
ALTER TABLE provider_item_variant_cache ADD CONSTRAINT item_variant_cache_cache_fk FOREIGN KEY ("provider_cache_id") REFERENCES provider_cache("id") ON DELETE CASCADE;




CREATE TABLE IF NOT EXISTS provider_item_cache (
      id uuid PRIMARY KEY,
      country_code country_code_type NOT NULL,
      provider_cache_id uuid NOT NULL,
      long_desc TEXT NOT NULL,
      short_desc TEXT NOT NULL,
      category_code TEXT NOT NULL,
      domain_code domain_category_type NOT NULL,
      item_code TEXT NOT NULL,
      item_id TEXT NOT NULL,
      item_name TEXT NOT NULL,
      currency currency_code_type NOT NULL,
      price_with_tax DECIMAL(20, 3) NOT NULL,
      price_without_tax DECIMAL(20, 3) NOT NULL,
      offered_price DECIMAL(20, 3),
      maximum_price DECIMAL(20, 3) NOT NULL,
      tax_rate DECIMAL(20, 3) NOT NULL,
      variant_cache_id uuid,
      recommended BOOLEAN NOT NULL,
      payment_ids JSONB NOT NULL,
      attributes JSONB,
      images JSONB NOT NULL,
      videos JSONB,
      price_slabs JSONB,
      fulfillment_options JSONB NOT NULL,
      payment_options JSONB NOT NULL,
      categories JSONB,
      qty JSONB NOT NULL,
      creator JSONB NOT NULL,
      matched BOOLEAN NOT NULL,
      time_to_ship TEXT NOT NULL,
      country_of_origin TEXT,
      validity JSONB,
      replacement_terms JSONB NOT NULL,
      return_terms JSONB NOT NULL,
      cancellation_terms JSONB NOT NULL,
      created_on TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
      updated_on TIMESTAMPTZ
);

ALTER TABLE provider_item_cache ADD CONSTRAINT item_cache_constraint UNIQUE (provider_cache_id, country_code, domain_code, item_id);
ALTER TABLE provider_item_cache ADD CONSTRAINT item_cache_variant_fk FOREIGN KEY ("variant_cache_id") REFERENCES provider_item_variant_cache("id") ON DELETE CASCADE;
ALTER TABLE provider_item_cache ADD CONSTRAINT item_cache_constraint_fk FOREIGN KEY ("provider_cache_id") REFERENCES provider_cache("id") ON DELETE CASCADE;



CREATE TABLE IF NOT EXISTS item_location_cache_relationship(
      id uuid PRIMARY KEY,
      item_cache_id uuid NOT NULL,
      location_cache_id uuid NOT NULL,
      created_on TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

ALTER TABLE item_location_cache_relationship ADD CONSTRAINT product_location_cache_relationship_constraint UNIQUE (item_cache_id, location_cache_id);
ALTER TABLE item_location_cache_relationship ADD CONSTRAINT product_fk FOREIGN KEY ("item_cache_id") REFERENCES provider_item_cache("id") ON DELETE CASCADE;
ALTER TABLE item_location_cache_relationship ADD CONSTRAINT location_fk FOREIGN KEY ("location_cache_id") REFERENCES provider_location_cache("id") ON DELETE CASCADE;


CREATE TABLE IF NOT EXISTS subscribed_search_location (
    id SERIAL NOT NULL PRIMARY KEY,
    city_code TEXT NOT NULL,
    country_code country_code_type NOT NULL,
    created_on TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    domain_category_code domain_category_type NOT NULL
);

ALTER TABLE subscribed_search_location ADD CONSTRAINT sub_city_code_uq UNIQUE (country_code, city_code, domain_category_code);



-- CREATE TABLE IF NOT EXISTS  buyer_order_status_history(
--   id uuid PRIMARY KEY,
--   order_id TEXT NOT NULL,
--   seller_id TEXT NOT NULL,
--   fulfillment_id TEXT,
--   status TEXT NOT NULL,
--   created_on TEXT NOT NULL
-- );