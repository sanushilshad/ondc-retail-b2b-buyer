use std::collections::HashMap;
use std::str::FromStr;

use bigdecimal::BigDecimal;
use serde_json::Value;
use sqlx::{PgPool, Postgres, Transaction, Executor};
use uuid::Uuid;
use crate::routes::product::schemas::{FulfillmentType, PaymentType, ProductSearchType};
use crate::schemas::RequestMetaData;
use super::models::{ProductVariantAttributeModel, WSSearchProviderContactModel, WSSearchProviderCredentialModel, WSSearchProviderTermsModel};
use super::schemas::{BulkCountryServicabilityCache, BulkGeoServicabilityCache, BulkHyperlocalServicabilityCache, BulkInterCityServicabilityCache, BulkItemVariantCache, BulkProviderCache, BulkProviderLocationCache, CategoryDomain, ProductSearchRequest, WSSearchBPP, WSSearchData, WSSearchProvider};
use chrono::{DateTime, Utc};
use crate::user_client::{BusinessAccount, UserAccount};
use crate::schemas::CountryCode;
use anyhow::anyhow;
#[tracing::instrument(name = "Save Product Search Request", skip(pool))]
pub async fn save_search_request(
    pool: &PgPool,
    user_account: &UserAccount,
    business_account: &BusinessAccount,
    meta_data: &RequestMetaData,
    search_request: &ProductSearchRequest,
) -> Result<(), anyhow::Error> {
    sqlx::query!(
        r#"
        INSERT INTO search_request (message_id, transaction_id, device_id, business_id,  user_id, created_on, update_cache, query, payment_type, domain_category_code, search_type, fulfillment_type)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        "#,
        &search_request.message_id,
        &search_request.transaction_id,
        &meta_data.device_id,
        &business_account.id,
        &user_account.id,
        Utc::now(),
        &search_request.update_cache,
        &search_request.query,
        &search_request.payment_type as &Option<PaymentType>, 
        &search_request.domain_category_code.to_string(),
        &search_request.search_type as &ProductSearchType,
        &search_request.fulfillment_type as &Option<FulfillmentType>

    )
    .execute(pool).await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e).context("A database failure occurred while saving ONDC search request")
    })?;
    Ok(())
}

#[tracing::instrument(name = "save ondc seller info", skip())]
fn create_bulk_provider_cache_objs<'a>(body: &'a Vec<WSSearchProvider>, network_participant_cache_id: Uuid, created_on: DateTime<Utc>) -> BulkProviderCache<'a> {
    let mut provider_ids: Vec<&str>= vec![];
    let mut network_participant_cache_ids = vec![];
    let mut names = vec![];
    let mut codes= vec![];
    let mut short_descs = vec![];
    let mut long_descs = vec![];
    let mut ttls = vec![];
    let mut ratings = vec![];
    let mut images= vec![];
    let mut identifications = vec![];
    let mut contacts = vec![];
    let mut terms = vec![];
    let mut credentials= vec![];
    let mut ids = vec![];
    let mut created_ons =vec![];
    for provider in body.iter() {
        ids.push(Uuid::new_v4());
        provider_ids.push(provider.description.id.as_ref());
        network_participant_cache_ids.push(network_participant_cache_id);
        names.push(provider.description.name.as_ref());
        codes.push(provider.description.code.as_ref());
        short_descs.push(provider.description.short_desc.as_ref());
        long_descs.push(provider.description.long_desc.as_ref());
        ttls.push(provider.description.ttl.as_ref());
        ratings.push(provider.description.rating);
        images.push(serde_json::to_value(&provider.description.images).unwrap());
        created_ons.push(created_on);
        contacts.push(
            serde_json::to_value(&WSSearchProviderContactModel{ 
                mobile_no: provider.description.contact.mobile_no.to_owned(),
                email: provider.description.contact.email.to_owned(),
                
            }
        ).unwrap());
        identifications.push(serde_json::to_value(&provider.description.identification).unwrap());
        terms.push(
            serde_json::to_value(
                &WSSearchProviderTermsModel{
                     gst_credit_invoice: provider.description.terms.gst_credit_invoice 
                }
            ).unwrap());
        credentials.push(serde_json::to_value(
            provider.description.credentials
                .iter()
                .map(|f| WSSearchProviderCredentialModel {
                    id: f.id.clone(),
                    r#type: f.r#type.clone(),
                    desc: f.desc.clone(),
                    url: f.url.clone(),
                })
                .collect::<Vec<_>>()
        ).unwrap())
        
    }

    return BulkProviderCache {
         provider_ids, 
         network_participant_cache_ids,
         names,
         codes, 
         short_descs, 
         long_descs, 
         images, 
         ratings, 
         ttls, 
         credentials, 
         contacts, 
         terms, 
         identifications,
         ids,
         created_ons
    };
}

#[tracing::instrument(name = "Save Seller Cache", skip(transaction))]
async fn save_provider_cache(
    transaction: &mut Transaction<'_, Postgres>,
    provider_data: &Vec<WSSearchProvider>,
    network_participant_cache_id: Uuid,
    created_on: DateTime<Utc>,
) -> Result<HashMap<String, Uuid>, anyhow::Error> {

    let data = create_bulk_provider_cache_objs(provider_data, network_participant_cache_id, created_on);
   
    let query = sqlx::query!(
        r#"
        INSERT INTO provider_cache  (provider_id, network_participant_cache_id, name, code, short_desc, long_desc, images, rating,
        ttl, credentials, contact, terms, identification, created_on, updated_on, id)
        SELECT *
        FROM UNNEST(
            $1::text[], 
            $2::uuid[], 
            $3::text[], 
            $4::text[], 
            $5::text[], 
            $6::text[], 
            $7::jsonb[],
            $8::real[],
            $9::text[],
            $10::jsonb[],
            $11::jsonb[],
            $12::jsonb[],
            $13::jsonb[],
            $14::timestamptz[],
            $15::timestamptz[],
            $16::uuid[]
        )
        ON CONFLICT (network_participant_cache_id, provider_id) 
        DO UPDATE SET 
        updated_on = EXCLUDED.updated_on,
        credentials = COALESCE((
            SELECT jsonb_agg(DISTINCT c) 
            FROM (
                SELECT DISTINCT ON (c->>'id') c
                FROM jsonb_array_elements(provider_cache.credentials || EXCLUDED.credentials) AS c
                ORDER BY c->>'id'
            ) AS unique_credentials
        ), '[]'::jsonb)
        RETURNING id, provider_id
        "#,
        &data.provider_ids[..] as &[&str],
        &data.network_participant_cache_ids[..] as &[Uuid],
        &data.names[..] as &[&str],
        &data.codes[..] as &[&str],
        &data.short_descs[..] as &[&str],
        &data.long_descs[..] as &[&str],
        &data.images[..],
        &data.ratings[..] as &[Option<f32>],
        &data.ttls[..] as &[&str],
        &data.credentials[..],
        &data.contacts[..],
        &data.terms[..],
        &data.identifications[..],
        &data.created_ons[..] as &[DateTime<Utc>],
        &data.created_ons[..] as &[DateTime<Utc>],
        &data.ids,
    );

    let result = query
        .fetch_all(&mut **transaction)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            println!("sanu{:?}",  &data.credentials);
            anyhow::Error::new(e)
                .context("A database failure occurred while saving provider cache")
        })?;

    let mut provider_map: HashMap<String, Uuid> = HashMap::new();

    for row in result {
        let provider_id: String = row.provider_id;
        let id: Uuid = row.id;
        provider_map.insert(provider_id, id);
    }

    Ok(provider_map)
}



#[tracing::instrument(name = "Save NP Cache", skip(transaction))]
async fn save_np_cache(
    transaction: &mut Transaction<'_, Postgres>,
    bpp: &WSSearchBPP,
    created_on: DateTime<Utc>
) -> Result<Uuid, anyhow::Error> {
    let query = sqlx::query!(
        r#"
        WITH ins AS(INSERT INTO network_participant_cache (id, subscriber_id, name, short_desc, long_desc, images, created_on)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (subscriber_id) 
        DO NOTHING
        RETURNING id)
        SELECT id FROM ins
        UNION ALL
        SELECT id FROM network_participant_cache WHERE subscriber_id = $2
        LIMIT 1;
        "#,
        Uuid::new_v4(),
        bpp.subscriber_id,
        bpp.name,
        bpp.short_desc,
        bpp.long_desc,
        serde_json::to_value(&bpp.images).unwrap(),
        &created_on

    );
    let result = query.fetch_one(&mut **transaction).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e)
            .context("A database failure occurred while saving np cache")
    })?;
    result.id.ok_or_else(|| anyhow::Error::msg("Failed to retrieve ID after insert"))
}




#[tracing::instrument(name = "save provider cache location info", skip())]
fn create_bulk_provider_location_cache_objs<'a>(
    providers: &'a Vec<WSSearchProvider>,
    provider_map: &'a HashMap<String, Uuid>,
    updated_on: DateTime<Utc>,
) -> BulkProviderLocationCache<'a> {
    let mut provider_ids = vec![];
    let mut location_ids = vec![];
    let mut latitudes = vec![];
    let mut longitudes = vec![];
    let mut addresses = vec![];
    let mut city_codes = vec![];
    let mut city_names = vec![];
    let mut state_codes = vec![];
    let mut state_names = vec![];
    let mut country_names = vec![];
    let mut country_codes = vec![];
    let mut area_codes = vec![];
    let mut updated_ons = vec![];
    let mut ids = vec![];
    for provider in providers {
        if let Some(provider_id) = provider_map.get(&provider.description.id){
            for (key, location) in &provider.locations {
                let gps_data = location
                    .gps
                    .split(',')
                    .map(|s| BigDecimal::from_str(s).unwrap_or_else(|_| BigDecimal::from(0).clone()))
                    .collect::<Vec<_>>();
                provider_ids.push(provider_id);
                location_ids.push(key.as_str());
                latitudes.push(gps_data.first().cloned().unwrap_or(BigDecimal::from(0)));
                longitudes.push(gps_data.get(1).cloned().unwrap_or(BigDecimal::from(0)));
                addresses.push(location.address.as_str());
                city_codes.push(location.city.code.as_str());
                city_names.push(location.city.name.as_str());
                state_codes.push(location.state.code.as_str());
                state_names.push(location.state.name.as_deref());
                country_names.push(location.country.name.as_deref());
                country_codes.push(&location.country.code);
                area_codes.push(location.area_code.as_str());
                updated_ons.push(updated_on);
                ids.push(Uuid::new_v4());
            };
        }

    }

    return BulkProviderLocationCache {

        provider_ids,
        location_ids,
        longitudes,
        latitudes,
        addresses,
        city_codes,
        city_names,
        state_codes,
        state_names,
        country_names,
        country_codes,
        area_codes,
        updated_ons,
        ids,
    };
}




#[tracing::instrument(name = "save ondc seller location info", skip(transaction, providers))]
async fn save_provider_location_cache<'a>(
    transaction: &mut Transaction<'_, Postgres>,
    providers: &Vec<WSSearchProvider>,
    provider_map: &HashMap<String, Uuid>,
    updated_on: DateTime<Utc>,
) -> Result<HashMap<String, Uuid>, anyhow::Error> {
    let seller_data = create_bulk_provider_location_cache_objs(providers, provider_map, updated_on);
    let query = sqlx::query!(
        r#"
        INSERT INTO provider_location_cache (
            provider_cache_id,
            location_id,
            latitude,
            longitude,
            location,
            address,
            city_code,
            city_name,
            state_code,
            state_name,
            country_code,
            country_name,
            area_code,
            created_on,
            updated_on,
            id
        )
        SELECT 
            provider_cache_id, 
            location_id, 
            latitude, 
            longitude, 
            ST_Transform(ST_SetSRID(ST_MakePoint(latitude, longitude), 4326), 3857),
            address, 
            city_code, 
            city_name, 
            state_code, 
            state_name, 
            country_code, 
            country_name, 
            area_code, 
            created_on, 
            updated_on, 
            id
        FROM UNNEST(
            $1::uuid[], 
            $2::text[], 
            $3::decimal[], 
            $4::decimal[], 
            $5::text[], 
            $6::text[],
            $7::text[],
            $8::text[],
            $9::text[],
            $10::country_code[],
            $11::text[],
            $12::text[],
            $13::timestamptz[],
            $14::timestamptz[],
            $15::uuid[]
        ) AS t(
            provider_cache_id, 
            location_id, 
            latitude, 
            longitude, 
            address, 
            city_code, 
            city_name, 
            state_code, 
            state_name, 
            country_code, 
            country_name, 
            area_code, 
            created_on, 
            updated_on, 
            id
        )
        ON CONFLICT (provider_cache_id, location_id) 
        DO UPDATE SET 
            latitude = EXCLUDED.latitude,
            longitude = EXCLUDED.longitude,
            location = EXCLUDED.location,
            address = EXCLUDED.address,
            city_code = EXCLUDED.city_code,
            city_name =  EXCLUDED.city_name,
            state_code =  EXCLUDED.state_code,
            state_name =  EXCLUDED.state_name,
            country_code =  EXCLUDED.country_code,
            country_name =  EXCLUDED.country_name,
            updated_on = EXCLUDED.updated_on,
            area_code = EXCLUDED.area_code
        RETURNING id, provider_cache_id, location_id
        "#,
        &seller_data.provider_ids[..] as &[&Uuid],
        &seller_data.location_ids[..] as &[&str],
        &seller_data.latitudes[..] as &[BigDecimal],
        &seller_data.longitudes[..] as &[BigDecimal],
        &seller_data.addresses[..] as &[&str],
        &seller_data.city_codes[..] as &[&str],
        &seller_data.city_names[..] as &[&str],
        &seller_data.state_codes[..] as &[&str],
        &seller_data.state_names[..] as &[Option<&str>],
        &seller_data.country_codes[..] as &[&CountryCode],
        &seller_data.country_names[..] as &[Option<&str>],
        &seller_data.area_codes[..] as &[&str],
        &seller_data.updated_ons[..] as &[DateTime<Utc>],
        &seller_data.updated_ons[..] as &[DateTime<Utc>],
        seller_data.ids as Vec<Uuid>,
    );
    let result = query
        .fetch_all(&mut **transaction)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            anyhow::Error::new(e)
                .context("A database failure occurred while saving provider cache")
        })?;

    let mut provider_map: HashMap<String, Uuid> = HashMap::new();

    for row in result {
        let provider_cache_id: Uuid = row.provider_cache_id;
        let location_id = row.location_id;
        let id: Uuid = row.id;
        provider_map.insert(format!("{}-{}",provider_cache_id, location_id), id);
    }

    Ok(provider_map)
}

fn create_bulk_geo_json_servicability<'a>(providers: &'a [WSSearchProvider], domain: &'a CategoryDomain, location_map: &'a HashMap<String, Uuid>, provider_map: &'a HashMap<String, Uuid>, created_on: DateTime<Utc>) -> BulkGeoServicabilityCache<'a>{
    let mut ids =  vec![];
    let mut category_codes =  vec![];
    let mut domain_codes =  vec![];
    let mut created_ons =  vec![];
    let mut cordinates =  vec![];
    let mut location_cache_ids =  vec![];




    for provider in providers.iter(){
        for (location_id, servicability_data) in provider.servicability.iter(){
        for geo_data in servicability_data.geo_json.iter(){
            let location_cache_id = provider_map.get(&provider.description.id).and_then(|f: &Uuid|location_map.get(&format!("{}-{}", f, location_id)));
                if let Some(location_cache_id) = location_cache_id{
                    ids.push(Uuid::new_v4());
                    category_codes.push(&geo_data.category_code);
                    domain_codes.push(domain);
                    created_ons.push(created_on);
                    cordinates.push(&geo_data.value);
                    location_cache_ids.push(location_cache_id);
                }
            }
        }



    }
    BulkGeoServicabilityCache{
        ids,
        location_cache_ids,
        cordinates,
        category_codes,
        created_ons,
        domain_codes
    }
}

#[tracing::instrument(name = "save_geo_json_servicability_cache", skip(transaction))]
async fn save_geo_json_servicability_cache(
    transaction: &mut Transaction<'_, Postgres>,
    providers:    &Vec<WSSearchProvider>,
    location_map: &HashMap<String, Uuid>,
    provider_map: &HashMap<String, Uuid>,
    created_on: DateTime<Utc>,
    domain: &CategoryDomain
) -> Result<(), anyhow::Error> {
    let data = create_bulk_geo_json_servicability(providers, domain, provider_map, provider_map, created_on);
    if !data.ids.is_empty(){
        let query = sqlx::query!(
            r#"
            INSERT INTO servicability_geo_json_cache (
                id,
                provider_location_cache_id,
                domain_code,
                geom,
                category_code,
                cordinates,
                created_on
            )
            SELECT 
                unnest($1::uuid[]), 
                unnest($2::uuid[]), 
                unnest($3::domain_category[]), 
                ST_SetSRID(ST_GeomFromGeoJSON(unnest($5::jsonb[])), 4326),
                unnest($4::text[]), 
                unnest($5::jsonb[]), 
                unnest($6::timestamptz[])
            ON CONFLICT (provider_location_cache_id, domain_code, category_code, geom) 
            DO UPDATE SET
                created_on = EXCLUDED.created_on;
            "#,
            &data.ids[..] as &[Uuid], 
            &data.location_cache_ids[..] as &[&Uuid], 
            &data.domain_codes[..] as &[&CategoryDomain], 
            &data.category_codes[..] as &[&Option<String>],
            &data.cordinates[..] as &[&Value],
            &data.created_ons, 
        );

        transaction
            .execute(query)
            .await
            .map_err(|e| {
                tracing::error!("Failed to execute query: {:?}", e);
                anyhow::Error::new(e)
                    .context("A database failure occurred while saving ONDC seller geojosn servicability cache info")
            })?;

    }

    Ok(())
}


fn create_bulk_hyperlocal_servicability<'a>(providers: &'a [WSSearchProvider], domain: &'a CategoryDomain, location_map: &'a HashMap<String, Uuid>, provider_map: &'a HashMap<String, Uuid>, created_on: DateTime<Utc>) -> BulkHyperlocalServicabilityCache<'a>{
    let mut ids =  vec![];
    let mut category_codes =  vec![];
    let mut domain_codes =  vec![];
    let mut created_ons =  vec![];
    let mut radii  =  vec![];
    let mut location_cache_ids =  vec![];




    for provider in providers.iter(){
        for (location_id, servicability_data) in provider.servicability.iter(){
        for geo_data in servicability_data.hyperlocal.iter(){
            let location_cache_id = provider_map.get(&provider.description.id).and_then(|f: &Uuid|location_map.get(&format!("{}-{}", f, location_id)));
                if let Some(location_cache_id) = location_cache_id{
                    ids.push(Uuid::new_v4());
                    category_codes.push(&geo_data.category_code);
                    domain_codes.push(domain);
                    created_ons.push(created_on);
                    radii.push(geo_data.value);
                    location_cache_ids.push(location_cache_id);
                }
            }
        }



    }
    BulkHyperlocalServicabilityCache{ ids, location_cache_ids, radii, category_codes, created_ons, domain_codes }
}

#[tracing::instrument(name = "save_hyperlocal_servicability_cache", skip(transaction))]
async fn save_hyperlocal_servicability_cache(
    transaction: &mut Transaction<'_, Postgres>,
    providers: & Vec<WSSearchProvider>,
    location_map: &HashMap<String, Uuid>,
    provider_map: &HashMap<String, Uuid>,
    created_on: DateTime<Utc>,
    domain: &CategoryDomain
) -> Result<(), anyhow::Error> {
    let data = create_bulk_hyperlocal_servicability(providers, domain, location_map, provider_map, created_on);
    if !data.ids.is_empty(){
    let query = sqlx::query!(
        r#"
        INSERT INTO servicability_hyperlocal_cache (
            id,
            provider_location_cache_id,
            domain_code,
            category_code,
            radius,
            created_on
        )
        SELECT 
            unnest($1::uuid[]), 
            unnest($2::uuid[]), 
            unnest($3::domain_category[]), 
            unnest($4::text[]), 
            unnest($5::double precision[]), 
            unnest($6::timestamptz[])
        ON CONFLICT (provider_location_cache_id, domain_code, category_code) 
        DO NOTHING
        "#,
        &data.ids[..] as &[Uuid], 
        &data.location_cache_ids[..] as &[&Uuid], 
        &data.domain_codes[..] as &[&CategoryDomain], 
        &data.category_codes[..] as &[&Option<String>],
        &data.radii[..] as &[f64],
        &data.created_ons, 
    );

    transaction
        .execute(query)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            anyhow::Error::new(e)
                .context("A database failure occurred while saving ONDC seller hyperlocal servicability cache info")
        })?;
    }


    Ok(())
}


fn create_bulk_country_servicability<'a>(providers: &'a Vec<WSSearchProvider>, domain: &'a CategoryDomain, location_map: &'a HashMap<String, Uuid>, provider_map: &'a HashMap<String, Uuid>, created_on: DateTime<Utc>) -> BulkCountryServicabilityCache<'a>{
    let mut ids =  vec![];
    let mut category_codes =  vec![];
    let mut domain_codes =  vec![];
    let mut created_ons =  vec![];
    let mut country_codes  =  vec![];
    let mut location_cache_ids =  vec![];




    for provider in providers{
        for (location_id, servicability_data) in provider.servicability.iter(){
        for geo_data in servicability_data.country.iter(){
            let location_cache_id = provider_map.get(&provider.description.id).and_then(|f: &Uuid|location_map.get(&format!("{}-{}", f, location_id)));
                if let Some(location_cache_id) = location_cache_id{
                    ids.push(Uuid::new_v4());
                    category_codes.push(&geo_data.category_code);
                    domain_codes.push(domain);
                    created_ons.push(created_on);
                    country_codes.push(&geo_data.value);
                    location_cache_ids.push(location_cache_id);
                }
            }
        }



    }
    BulkCountryServicabilityCache{ ids, location_cache_ids, country_codes, category_codes, created_ons, domain_codes }
}

#[tracing::instrument(name = "save_country_servicability_cache", skip(transaction))]
 async fn save_country_servicability_cache(
    transaction: &mut Transaction<'_, Postgres>,
    providers: &Vec<WSSearchProvider>, 
    location_map: &HashMap<String, Uuid>,
    provider_map: &HashMap<String, Uuid>,
    created_on: DateTime<Utc>,
    domain: &CategoryDomain
) -> Result<(), anyhow::Error> {
    let data = create_bulk_country_servicability(providers, domain, location_map, provider_map, created_on);
    if !data.ids.is_empty(){
    let query = sqlx::query!(
        r#"
        INSERT INTO servicability_country_cache (
            id,
            provider_location_cache_id,
            domain_code,
            category_code,
            country_code,
            created_on
        )
        SELECT 
            unnest($1::uuid[]), 
            unnest($2::uuid[]), 
            unnest($3::domain_category[]), 
            unnest($4::text[]), 
            unnest($5::country_code[]), 
            unnest($6::timestamptz[])
        ON CONFLICT (provider_location_cache_id, domain_code, category_code, country_code) 
        DO NOTHING
        "#,
        &data.ids[..] as &[Uuid], 
        &data.location_cache_ids[..] as &[&Uuid], 
        &data.domain_codes[..] as &[&CategoryDomain], 
        &data.category_codes[..] as &[&Option<String>],
        &data.country_codes[..] as &[&CountryCode],
        &data.created_ons, 
    );

    transaction
        .execute(query)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            anyhow::Error::new(e)
                .context("A database failure occurred while saving ONDC seller hyperlocal servicability cache info")
        })?;
    }


    Ok(())
}



fn create_bulk_intercity_servicability<'a>(providers: &'a Vec<WSSearchProvider>, domain: &'a CategoryDomain, location_map: &'a HashMap<String, Uuid>, provider_map: &'a HashMap<String, Uuid>, created_on: DateTime<Utc>) -> BulkInterCityServicabilityCache<'a>{
    let mut ids =  vec![];
    let mut category_codes =  vec![];
    let mut domain_codes =  vec![];
    let mut created_ons =  vec![];
    let mut pincodes  =  vec![];
    let mut location_cache_ids =  vec![];

    for provider in providers{
        for (location_id, servicability_data) in provider.servicability.iter(){
        for geo_data in servicability_data.intercity.iter(){
            let location_cache_id = provider_map.get(&provider.description.id).and_then(|f: &Uuid|location_map.get(&format!("{}-{}", f, location_id)));
                if let Some(location_cache_id) = location_cache_id{
                    for pincode in geo_data.value.iter(){
                        ids.push(Uuid::new_v4());
                        category_codes.push(&geo_data.category_code);
                        domain_codes.push(domain);
                        created_ons.push(created_on);
                        pincodes.push(pincode.as_str());
                        location_cache_ids.push(location_cache_id);
                    }

                }
            }
        }



    }
    BulkInterCityServicabilityCache{ ids, location_cache_ids, pincodes, category_codes, created_ons, domain_codes }
}

#[tracing::instrument(name = "save_intecity_servicability_cache", skip(transaction))]
async fn save_intercity_servicability_cache(
    transaction: &mut Transaction<'_, Postgres>,
    providers: &Vec<WSSearchProvider>, 
    location_map: &HashMap<String, Uuid>,
    provider_map: &HashMap<String, Uuid>,
    created_on: DateTime<Utc>,
    domain: &CategoryDomain
) -> Result<(), anyhow::Error> {
    let data = create_bulk_intercity_servicability(providers, domain, location_map, provider_map, created_on);
    if !data.ids.is_empty(){
    let query = sqlx::query!(
        r#"
        INSERT INTO servicability_intercity_cache (
            id,
            provider_location_cache_id,
            domain_code,
            category_code,
            pincode,
            created_on
        )
        SELECT 
            unnest($1::uuid[]), 
            unnest($2::uuid[]), 
            unnest($3::domain_category[]), 
            unnest($4::text[]), 
            unnest($5::text[]), 
            unnest($6::timestamptz[])
        ON CONFLICT (provider_location_cache_id, domain_code, category_code, pincode) 
        DO NOTHING
        "#,
        &data.ids[..] as &[Uuid], 
        &data.location_cache_ids[..] as &[&Uuid], 
        &data.domain_codes[..] as &[&CategoryDomain], 
        &data.category_codes[..] as &[&Option<String>],
        &data.pincodes[..] as &[&str],
        &data.created_ons, 
    );

    transaction
        .execute(query)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            anyhow::Error::new(e)
                .context("A database failure occurred while saving ONDC seller hyperlocal servicability cache info")
        })?;
    }


    Ok(())
}



async fn save_location_servicability_cache(transaction: &mut Transaction<'_, Postgres>,  providers: &Vec<WSSearchProvider>, domain: &CategoryDomain,  location_map: &HashMap<String, Uuid>, provider_map:&HashMap<String, Uuid>,  created_on: DateTime<Utc>) -> Result<(), anyhow::Error> {
    save_geo_json_servicability_cache(transaction, providers, location_map, provider_map, created_on, domain).await?;
    save_hyperlocal_servicability_cache(transaction, providers, location_map, provider_map, created_on, domain).await?;
    save_country_servicability_cache(transaction, providers, location_map, provider_map, created_on, domain).await?;
    save_intercity_servicability_cache(transaction, providers, location_map, provider_map, created_on, domain).await?;
    Ok(())
} 

fn create_bulk_variant<'a>(providers: &'a Vec<WSSearchProvider>,  provider_map: &'a HashMap<String, Uuid>, created_on: DateTime<Utc>) -> BulkItemVariantCache<'a>{
    let mut provider_ids = vec![];
    let mut ids = vec![];
    let mut variant_ids = vec![];
    let mut variant_names = vec![];
    let mut created_ons = vec![];
    let mut attributes = vec![];
    for provider  in providers{
        if let Some(provider_id) = provider_map.get(&provider.description.id){
            if let Some(variants) =  &provider.variants{
                for (variant_id, variant) in variants{
                    let attribute_data: Vec<ProductVariantAttributeModel> = variant.attributes.iter().map(|f| ProductVariantAttributeModel{ attribute_code: f.attribute_code.to_owned(), sequence: f.sequence.to_owned() }).collect();
                    ids.push(Uuid::new_v4());
                    variant_ids.push(variant_id.as_str());
                    variant_names.push(variant.name.as_str());
                    created_ons.push(created_on);
                    provider_ids.push(provider_id);
                    attributes.push(serde_json::to_value(attribute_data).unwrap());
                };
            }     
        }



    };
    BulkItemVariantCache{
        provider_ids,
        ids,
        variant_ids,
        variant_names,
        created_ons,
        attributes
    }
}


async fn save_variant_cache(
    transaction: &mut Transaction<'_, Postgres>, 
    providers: &Vec<WSSearchProvider>, 
    provider_map: &HashMap<String, Uuid>, 
    created_on: DateTime<Utc>
) -> Result<(), anyhow::Error> {
    let data = create_bulk_variant(providers, provider_map, created_on);
    
    if !data.ids.is_empty() {
        let query = sqlx::query!(
            r#"
            INSERT INTO cache_item_variant (
                id,
                provider_cache_id,
                variant_id,
                variant_name,
                attributes,
                created_on,
                updated_on
            )
            SELECT 
                unnest($1::uuid[]), 
                unnest($2::uuid[]), 
                unnest($3::text[]), 
                unnest($4::text[]), 
                unnest($5::jsonb[]), 
                unnest($6::timestamptz[]),
                unnest($7::timestamptz[])
            ON CONFLICT (provider_cache_id, variant_id) 
            DO UPDATE SET 
            updated_on = EXCLUDED.updated_on,
            attributes  = EXCLUDED.attributes
            "#,
            &data.ids[..] as &[Uuid], 
            &data.provider_ids[..] as &[&Uuid], 
            &data.variant_ids[..] as &[&str], 
            &data.variant_names[..] as &[&str],
            &data.attributes[..] as &[Value], 
            &data.created_ons[..] as &[DateTime<Utc>],
            &data.created_ons[..] as &[DateTime<Utc>]
        );

        transaction
            .execute(query)
            .await
            .map_err(|e| {
                tracing::error!("Failed to execute query: {:?}", e);
                anyhow::Error::new(e)
                    .context("A database failure occurred while saving ONDC seller hyperlocal servicability cache info")
            })?;
    }

    Ok(())
}


pub async fn save_cache_to_db(transaction: &mut Transaction<'_, Postgres>, domain: &CategoryDomain, product_objs: &WSSearchData,  created_on: DateTime<Utc>) -> Result<(),anyhow::Error>{
    
    let id = save_np_cache(transaction, &product_objs.bpp, created_on)
        .await
        .map_err(|e| anyhow!(e))?;

    let provider_map = save_provider_cache(
        transaction,
        &product_objs.providers,
        id,
        created_on,
    )
    .await
    .map_err(|e| anyhow!(e))?;

    let location_map = save_provider_location_cache(
        transaction,
        &product_objs.providers,
        &provider_map,
        created_on,
    )
    .await
    .map_err(|e| anyhow!(e))?;

    save_location_servicability_cache(transaction, &product_objs.providers, domain, &location_map, &provider_map, created_on)
        .await
        .map_err(|e| anyhow!(e))?;

    save_variant_cache(transaction, &product_objs.providers, &provider_map, created_on).await
        .map_err(|e| anyhow!(e))?;

    Ok(())

}