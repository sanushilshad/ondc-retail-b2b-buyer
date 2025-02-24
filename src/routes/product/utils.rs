use std::collections::HashMap;
use std::str::FromStr;

use bigdecimal::{BigDecimal, ToPrimitive};
use elasticsearch::http::request::JsonBody;
use serde_json::{json, Value};
use sqlx::{PgPool, Postgres, Transaction, Executor};
use tokio::try_join;
use uuid::Uuid;
use crate::elastic_search_client::{ElasticSearchClient, ElasticSearchIndex};
use crate::routes::product::models::{ESHyperlocalServicabilityModel, ESProviderLocationModel, ESProviderModel};
use crate::routes::product::schemas::{FulfillmentType, PaymentType, ProductSearchType};
use crate::schemas::{CurrencyType, RequestMetaData};
use super::models::{ESCountryServicabilityModel, ESGeoJsonServicabilityModel, ESInterCityServicabilityModel, ESLocationModel, ESNetworkParticipantModel, ESProviderItemModel, ESProviderItemVariantModel, ProductVariantAttributeModel, SearchProviderCredentialModel, WSItemReplacementTermModel, WSItemReturnTermModel, WSPriceSlabModel, WSProductCategoryModel, WSSearchItemAttributeModel, WSSearchProviderContactModel, WSSearchProviderTermsModel};
use super::schemas::{BulkCountryServicabilityCache, BulkGeoServicabilityCache, BulkHyperlocalServicabilityCache, BulkInterCityServicabilityCache, BulkItemCache, BulkItemLocationCache, BulkItemVariantCache, BulkProviderCache, BulkProviderLocationCache, CategoryDomain, DBItemCacheData, ProductSearchRequest, ServicabilityIds, WSSearchBPP, WSSearchData, WSSearchProvider};
use chrono::{DateTime, Utc};
use crate::user_client::{BusinessAccount, UserAccount};
use crate::schemas::CountryCode;

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
        identifications.push(serde_json::to_value(&provider.description.identifications).unwrap());
        terms.push(
            serde_json::to_value(
                &WSSearchProviderTermsModel{
                     gst_credit_invoice: provider.description.terms.gst_credit_invoice 
                }
            ).unwrap());
        credentials.push(serde_json::to_value(
            provider.description.credentials
                .iter()
                .map(|f| SearchProviderCredentialModel {
                    id: f.id.clone(),
                    r#type: f.r#type.clone(),
                    desc: f.desc.clone(),
                    url: f.url.clone(),
                })
                .collect::<Vec<_>>()
        ).unwrap());
        // payment_options.push(
        //     serde_json::to_value(
        //         provider.payments.iter().map(|(id, payment_obj)|
        //         ProviderPaymentOptionModel{
        //             r#type: payment_obj.r#type.to_owned(),
        //             collected_by: payment_obj.collected_by.to_owned(),
        //             id: id.to_owned() 
        //         }
        //     ).collect::<Vec<_>>()
        // ).unwrap());
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
         created_ons,
         
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
        ttl, credentials, contact, terms, identifications, created_on, updated_on, id)
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
        &data.images,
        &data.ratings[..] as &[Option<f32>],
        &data.ttls[..] as &[&str],
        &data.credentials,
        &data.contacts,
        &data.terms,
        &data.identifications,
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
            $10::country_code_type[],
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
    let mut coordinates =  vec![];
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
                    coordinates.push(&geo_data.value);
                    location_cache_ids.push(location_cache_id);
                }
            }
        }



    }
    BulkGeoServicabilityCache{
        ids,
        location_cache_ids,
        coordinates,
        category_codes,
        created_ons,
        domain_codes
    }
}

#[tracing::instrument(name = "save_geo_json_servicability_cache", skip(transaction))]
async fn save_provider_geo_json_servicability_cache(
    transaction: &mut Transaction<'_, Postgres>,
    providers:    &Vec<WSSearchProvider>,
    location_map: &HashMap<String, Uuid>,
    provider_map: &HashMap<String, Uuid>,
    created_on: DateTime<Utc>,
    domain: &CategoryDomain
) -> Result<Vec<Uuid>, anyhow::Error> {
    let data = create_bulk_geo_json_servicability(providers, domain, location_map, provider_map, created_on);
    if !data.ids.is_empty(){
        let query = sqlx::query!(
            r#"
            INSERT INTO provider_servicability_geo_json_cache (
                id,
                provider_location_cache_id,
                domain_code,
                geom,
                category_code,
                coordinates,
                created_on
            )
            SELECT 
                unnest($1::uuid[]), 
                unnest($2::uuid[]), 
                unnest($3::domain_category_type[]), 
                ST_SetSRID(ST_GeomFromGeoJSON(unnest($5::jsonb[])), 4326),
                unnest($4::text[]), 
                unnest($5::jsonb[]), 
                unnest($6::timestamptz[])
            ON CONFLICT (provider_location_cache_id, domain_code, category_code, geom) 
            DO NOTHING
            RETURNING id
            "#,
            &data.ids, 
            &data.location_cache_ids[..] as &[&Uuid], 
            &data.domain_codes[..] as &[&CategoryDomain], 
            &data.category_codes[..] as &[&Option<String>],
            &data.coordinates[..] as &[&Value],
            &data.created_ons, 
        );

        let result = query
            .fetch_all(&mut **transaction)
            .await
            .map_err(|e| {
                tracing::error!("Failed to execute query: {:?}", e);

                anyhow::Error::new(e)
                    .context("A database failure occurred while saving ONDC seller geojosn servicability cache inf")
            })?;
        let ids: Vec<Uuid> = result.into_iter().map(|row| row.id).collect();
        return Ok(ids);

    }
    Ok(Vec::new())

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
async fn save_provider_hyperlocal_servicability_cache(
    transaction: &mut Transaction<'_, Postgres>,
    providers: & Vec<WSSearchProvider>,
    location_map: &HashMap<String, Uuid>,
    provider_map: &HashMap<String, Uuid>,
    created_on: DateTime<Utc>,
    domain: &CategoryDomain
) -> Result<Vec<Uuid>, anyhow::Error> {
    let data = create_bulk_hyperlocal_servicability(providers, domain, location_map, provider_map, created_on);
    if !data.ids.is_empty(){
    let query = sqlx::query!(
        r#"
        INSERT INTO provider_servicability_hyperlocal_cache (
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
            unnest($3::domain_category_type[]), 
            unnest($4::text[]), 
            unnest($5::double precision[]), 
            unnest($6::timestamptz[])
        ON CONFLICT (provider_location_cache_id, domain_code, category_code) 
        DO UPDATE SET
        radius = EXCLUDED.radius
        RETURNING id
        "#,
        &data.ids,
        &data.location_cache_ids[..] as &[&Uuid], 
        &data.domain_codes[..] as &[&CategoryDomain], 
        &data.category_codes[..] as &[&Option<String>],
        &data.radii,
        &data.created_ons, 
    );

    let result = query
            .fetch_all(&mut **transaction)
            .await
            .map_err(|e| {
                tracing::error!("Failed to execute query: {:?}", e);

                anyhow::Error::new(e)
                    .context("A database failure occurred while saving ONDC seller hyperlocal servicability cache inf")
            })?;
        let ids: Vec<Uuid> = result.into_iter().map(|row| row.id).collect();
        return Ok(ids);
    }


       Ok(Vec::new())
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
 async fn save_provider_country_servicability_cache(
    transaction: &mut Transaction<'_, Postgres>,
    providers: &Vec<WSSearchProvider>, 
    location_map: &HashMap<String, Uuid>,
    provider_map: &HashMap<String, Uuid>,
    created_on: DateTime<Utc>,
    domain: &CategoryDomain
) -> Result<Vec<Uuid>, anyhow::Error> {
    let data = create_bulk_country_servicability(providers, domain, location_map, provider_map, created_on);
    if !data.ids.is_empty(){
        let query = sqlx::query!(
            r#"
            INSERT INTO provider_servicability_country_cache (
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
                unnest($3::domain_category_type[]), 
                unnest($4::text[]), 
                unnest($5::country_code_type[]), 
                unnest($6::timestamptz[])
            ON CONFLICT (provider_location_cache_id, domain_code, category_code, country_code) 
            DO NOTHING
            RETURNING id
            "#,
            &data.ids,
            &data.location_cache_ids[..] as &[&Uuid], 
            &data.domain_codes[..] as &[&CategoryDomain], 
            &data.category_codes[..] as &[&Option<String>],
            &data.country_codes[..] as &[&CountryCode],
            &data.created_ons, 
        );

        // transaction
        //     .execute(query)
        //     .await
        //     .map_err(|e| {
        //         tracing::error!("Failed to execute query: {:?}", e);
        //         anyhow::Error::new(e)
        //             .context("A database failure occurred while saving ONDC seller hyperlocal servicability cache info")
        //     })?;
        let result = query
            .fetch_all(&mut **transaction)
            .await
            .map_err(|e| {
                tracing::error!("Failed to execute query: {:?}", e);

                anyhow::Error::new(e)
                    .context("A database failure occurred while saving ONDC seller hyperlocal servicability cache inf")
            })?;
        let ids: Vec<Uuid> = result.into_iter().map(|row| row.id).collect();
        return Ok(ids);
    }


    Ok(Vec::new())
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
async fn save_provider_intercity_servicability_cache(
    transaction: &mut Transaction<'_, Postgres>,
    providers: &Vec<WSSearchProvider>, 
    location_map: &HashMap<String, Uuid>,
    provider_map: &HashMap<String, Uuid>,
    created_on: DateTime<Utc>,
    domain: &CategoryDomain
) -> Result<Vec<Uuid>, anyhow::Error> {
    let data = create_bulk_intercity_servicability(providers, domain, location_map, provider_map, created_on);
    if !data.ids.is_empty(){
    let query = sqlx::query!(
        r#"
        INSERT INTO provider_servicability_intercity_cache (
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
            unnest($3::domain_category_type[]), 
            unnest($4::text[]), 
            unnest($5::text[]), 
            unnest($6::timestamptz[])
        ON CONFLICT (provider_location_cache_id, domain_code, category_code, pincode) 
        DO NOTHING
        RETURNING id
        "#,
        &data.ids,
        &data.location_cache_ids[..] as &[&Uuid], 
        &data.domain_codes[..] as &[&CategoryDomain], 
        &data.category_codes[..] as &[&Option<String>],
        &data.pincodes[..] as &[&str],
        &data.created_ons, 
    );

        let result = query
            .fetch_all(&mut **transaction)
            .await
            .map_err(|e| {
                tracing::error!("Failed to execute query: {:?}", e);

                anyhow::Error::new(e)
                    .context("A database failure occurred while saving ONDC seller intercity servicability cache")
            })?;
        let ids: Vec<Uuid> = result.into_iter().map(|row| row.id).collect();
        return Ok(ids);
    }


    Ok(Vec::new())
}






async fn save_provider_location_servicability_cache(transaction: &mut Transaction<'_, Postgres>,  providers: &Vec<WSSearchProvider>, domain: &CategoryDomain,  location_map: &HashMap<String, Uuid>, provider_map:&HashMap<String, Uuid>,  created_on: DateTime<Utc>) -> Result<ServicabilityIds, anyhow::Error> {
    let geo_json_ids = save_provider_geo_json_servicability_cache(transaction, providers, location_map, provider_map, created_on, domain).await?;
    let hyper_local_ids = save_provider_hyperlocal_servicability_cache(transaction, providers, location_map, provider_map, created_on, domain).await?;
    let country_ids = save_provider_country_servicability_cache(transaction, providers, location_map, provider_map, created_on, domain).await?;
    let inter_city_ids = save_provider_intercity_servicability_cache(transaction, providers, location_map, provider_map, created_on, domain).await?;
    Ok(ServicabilityIds{ hyperlocal: hyper_local_ids, country: country_ids, inter_city: inter_city_ids, geo_json: geo_json_ids})
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
) -> Result<HashMap<String, Uuid>, anyhow::Error> {
    let data = create_bulk_variant(providers, provider_map, created_on);
    let mut variant_map: HashMap<String, Uuid> = HashMap::new();
    if !data.ids.is_empty() {
        let query = sqlx::query!(
            r#"
            INSERT INTO item_variant_cache (
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
            RETURNING id, provider_cache_id, variant_id
            "#,
            &data.ids,
            &data.provider_ids[..] as &[&Uuid], 
            &data.variant_ids[..] as &[&str], 
            &data.variant_names[..] as &[&str],
            &data.attributes[..] as &[Value], 
            &data.created_ons[..] as &[DateTime<Utc>],
            &data.created_ons[..] as &[DateTime<Utc>]
        );

        let result = query
            .fetch_all(&mut **transaction)
            .await
            .map_err(|e| {
                tracing::error!("Failed to execute query: {:?}", e);

                anyhow::Error::new(e)
                    .context("A database failure occurred while saving variant cache")
            })?;

        for row in result {

            let key = format!("{}_{}", row.provider_cache_id,  row.variant_id);
            variant_map.insert(key, row.id);
        };
    }

    Ok(variant_map)
}

fn create_bulk_items<'a>(providers: &'a Vec<WSSearchProvider>, country_code: &'a CountryCode, provider_map: &'a HashMap<String, Uuid>, variant_map: &'a HashMap<String, Uuid>, created_on: DateTime<Utc>) -> BulkItemCache<'a>{
    let mut provider_ids = vec![];
    let mut ids = vec![];
    let mut item_names = vec![];
    let mut created_ons = vec![];
    let mut country_codes = vec![];
    let mut domain_codes = vec![];
    let mut short_descs= vec![];
        let mut long_descs= vec![];
    let mut item_ids = vec![];
    let mut item_codes = vec![];
    let mut currencies= vec![];
    let mut recommends= vec![];
    let mut matched= vec![];
    let mut country_of_origins= vec![];
    let mut tax_rates= vec![];
    let mut price_with_taxes= vec![];
    let mut price_without_taxes= vec![];
    let mut offered_prices= vec![];
    let mut maximum_prices= vec![];
    let mut variant_ids= vec![];
    let mut time_to_ships= vec![];
    let mut images= vec![];
    let mut videos= vec![];
    let mut payment_options = vec![];
    let mut attributes= vec![];
    let mut price_slabs= vec![];
    let mut fulfillment_options= vec![];
    let mut return_terms= vec![];
    let mut replacement_terms= vec![];
    let mut cancellation_terms= vec![];
    let mut creators= vec![];
    let mut categories= vec![];
    let mut validities= vec![];
    let mut qtys= vec![];

    for provider  in providers{
        if let Some(provider_id) = provider_map.get(&provider.description.id){

            for item in &provider.items{
                let attribute_models: Vec<WSSearchItemAttributeModel> = item.attributes.iter().map(|a|a.get_model()).collect();

                ids.push(Uuid::new_v4());
                created_ons.push(created_on);
                provider_ids.push(provider_id);
                country_codes.push(country_code);
                domain_codes.push(&item.domain_category);
                // category_codes.push(item.categories.first().map_or("NA", |f|&f.code));
                item_ids.push(item.id.as_str());
                item_codes.push(item.id.as_str());
                item_names.push(item.name.as_str());
                short_descs.push(item.short_desc.as_str());
                long_descs.push(item.long_desc.as_str());
                currencies.push(&item.price.currency);
                matched.push(item.matched);
                recommends.push(item.recommended);
                country_of_origins.push(item.country_of_origin.as_deref());
                tax_rates.push(&item.tax_rate);
                price_with_taxes.push(&item.price.price_with_tax);
                price_without_taxes.push(&item.price.price_without_tax);
                offered_prices.push(&item.price.offered_value);
                maximum_prices.push(&item.price.maximum_price);
                if let Some(parent_item_id) = &item.parent_item_id {
                    let key = format!("{}_{}", provider_id, parent_item_id);
                    variant_ids.push(variant_map.get(&key).copied());
                } else {
                    variant_ids.push(None);
                }

                
                time_to_ships.push(item.time_to_ship.as_ref());
                payment_options.push(json!(item.payment_options));
                fulfillment_options.push(json!(item.fullfillment_options));
                images.push(json!(item.images));
                videos.push(json!(item.videos));
                attributes.push(json!(attribute_models));
                if let Some(price_slab) = &item.price_slabs{
                    let slab_models: Vec<WSPriceSlabModel> = price_slab.iter().map(|a|a.get_model()).collect();
                    price_slabs.push(Some(json!(slab_models)));
                }
                else{
                    price_slabs.push(None);
                };
                let return_term_model: Vec<WSItemReturnTermModel> = item.return_terms.iter().map(|a|a.get_model()).collect();
                return_terms.push(json!(return_term_model));
                let replacement_term: Vec<WSItemReplacementTermModel> = item.replacement_terms.iter().map(|a|a.get_model()).collect();
                replacement_terms.push(json!(replacement_term));
                
                cancellation_terms.push(json!(item.cancellation_terms.get_model()));
                creators.push(json!(item.creator.get_model()));
                let category: Vec<WSProductCategoryModel> = item.categories.iter().map(|f|f.get_model()).collect();
                categories.push(json!(category));
                validities.push(json!(item.validity.as_ref().map(|f|f.get_model())));
                qtys.push(json!(item.quantity.get_model()));
            };
         
        }

    };
    BulkItemCache{
        provider_ids,
        ids,
        country_codes,
        domain_codes,
        // category_codes,
        short_descs,
        long_descs,
        item_ids,
        item_codes,
        created_ons,
        item_names,
        currencies,
        price_with_taxes,
        price_without_taxes,
        offered_prices,
        maximum_prices,
        tax_rates,
        variant_ids,
        recommends,
        matched,
        attributes,
        images,
        videos,
        price_slabs,
        fulfillment_options,
        categories,
        creators,
        time_to_ships,
        country_of_origins,
        validities,
        replacement_terms,
        return_terms,
        cancellation_terms,
        qtys,
        payment_options
    }
}



fn create_bulk_item_location_mapping<'a>(
    providers: &'a Vec<WSSearchProvider>,
    provider_map: &'a HashMap<String, Uuid>,
    location_map: &'a HashMap<String, Uuid>,
    item_map: &'a HashMap<String, Uuid>,
    created_on: DateTime<Utc>,
) -> BulkItemLocationCache<'a> {
    let mut item_cache_ids = Vec::new();
    let mut ids = Vec::new();
    let mut location_cache_ids = Vec::new();
    let mut created_ons = Vec::new();

    for provider in providers {
        if let Some(provider_id) = provider_map.get(&provider.description.id) {
            for item in provider.items.iter() {
                let item_map_key = format!("{}_{}", &provider_id, &item.id);
                if let Some(item_id) = item_map.get(&item_map_key) {
                    for location_id in item.location_ids.iter() {
                        let location_key = format!("{}-{}", provider_id, location_id);
                        if let Some(location_fk) = location_map.get(&location_key) {
                            ids.push(Uuid::new_v4());
                            created_ons.push(created_on);
                            item_cache_ids.push(item_id);        
                            location_cache_ids.push(location_fk);
                        } else {
                            tracing::warn!("Missing location mapping for key: {}", location_key);
                        }
                    }
                } else {
                    tracing::warn!("Missing item mapping for key: {}", item_map_key);
                }
            }
        }
    }
    BulkItemLocationCache {
        ids,
        item_cache_ids,
        location_cache_ids,
        created_ons,
    }
}

async fn save_item_location_relationship_cache(transaction: &mut Transaction<'_, Postgres>, providers:  &Vec<WSSearchProvider>,  provider_map: &HashMap<String, Uuid>, location_map: &HashMap<String, Uuid>,item_map: &HashMap<String, Uuid>, created_on: DateTime<Utc>) -> Result<(), anyhow::Error>{
    let data = create_bulk_item_location_mapping(providers, provider_map, location_map, item_map, created_on);
    if !data.ids.is_empty() {
        let query = sqlx::query!(
            r#"
            INSERT INTO item_location_cache_relationship (
                id,
                item_cache_id,
                location_cache_id,
                created_on
            )
            SELECT 
                unnest($1::uuid[]), 
                unnest($2::uuid[]), 
                unnest($3::uuid[]), 
                unnest($4::timestamptz[])
            ON CONFLICT (item_cache_id, location_cache_id) 
            DO NOTHING
            "#,
            &data.ids,
            &data.item_cache_ids[..] as &[&Uuid], 
            &data.location_cache_ids as &[&Uuid],
            &data.created_ons
        );

        transaction
            .execute(query)
            .await
            .map_err(|e| {
                tracing::error!("Failed to execute query: {:?}", e);
                anyhow::Error::new(e)
                    .context("A database failure occurred while saving ONDC item-location mapping  cache info")
            })?;
    }
    Ok(())
}

async fn save_item_cache(transaction: &mut Transaction<'_, Postgres>, country_code: &CountryCode, providers:  &Vec<WSSearchProvider>,  provider_map: &HashMap<String, Uuid>, variant_map: &HashMap<String, Uuid>, created_on: DateTime<Utc>) -> Result<HashMap<String, Uuid>, anyhow::Error>{
    let data: BulkItemCache<'_> = create_bulk_items(providers,country_code, provider_map,variant_map, created_on);
    let mut map: HashMap<String, Uuid> = HashMap::new();
    if !data.ids.is_empty() {
        let query = sqlx::query!(
            r#"
                INSERT INTO item_cache (
                id,
                country_code,
                provider_cache_id,
                domain_code,
                item_code,
                item_id,
                item_name,
                currency,
                price_with_tax,
                price_without_tax,
                offered_price,
                maximum_price,
                tax_rate,
                variant_cache_id,
                recommended,
                attributes,
                images,
                videos,
                price_slabs,
                fulfillment_options,
                categories,
                qty,
                creator,
                matched,
                time_to_ship,
                country_of_origin,
                validity,
                replacement_terms,
                return_terms,
                cancellation_terms,
                payment_options,
                created_on,
                updated_on,
                long_desc,
                short_desc
                )
                SELECT 
                unnest($1::uuid[]),                      
                unnest($2::country_code_type[]),          
                unnest($3::uuid[]),                                            
                unnest($4::domain_category_type[]),           
                unnest($5::text[]),                       
                unnest($6::text[]),                        
                unnest($7::text[]),                       
                unnest($8::currency_code_type[]),         
                unnest($9::decimal[]),                   
                unnest($10::decimal[]),                   
                unnest($11::decimal[]),                    
                unnest($12::decimal[]),                  
                unnest($13::decimal[]),                    
                unnest($14::uuid[]),                     
                unnest($15::bool[]),                                           
                unnest($16::jsonb[]),                      
                unnest($17::jsonb[]),                      
                unnest($18::jsonb[]),                     
                unnest($19::jsonb[]),                      
                unnest($20::jsonb[]),                       
                unnest($21::jsonb[]),                     
                unnest($22::jsonb[]),                    
                unnest($23::jsonb[]),                      
                unnest($24::bool[]),                     
                unnest($25::text[]),                      
                unnest($26::country_code_type[]),          
                unnest($27::jsonb[]),                     
                unnest($28::jsonb[]),                    
                unnest($29::jsonb[]),                      
                unnest($30::jsonb[]),             
                unnest($31::jsonb[]),                   
                unnest($32::timestamptz[]),                 
                unnest($33::timestamptz[]),
                unnest($34::text[]),
                unnest($35::text[])                    
                ON CONFLICT (provider_cache_id, country_code, domain_code, item_id)
                DO UPDATE SET 
                updated_on = EXCLUDED.updated_on,
                item_code = EXCLUDED.item_code,
                item_name = EXCLUDED.item_name,
                currency = EXCLUDED.currency,
                price_with_tax = EXCLUDED.price_with_tax,
                price_without_tax = EXCLUDED.price_without_tax,
                offered_price = EXCLUDED.offered_price,
                maximum_price = EXCLUDED.maximum_price,
                tax_rate = EXCLUDED.tax_rate,
                variant_cache_id = EXCLUDED.variant_cache_id,
                recommended = EXCLUDED.recommended,
                attributes = EXCLUDED.attributes,
                images = EXCLUDED.images,
                videos = EXCLUDED.videos,
                price_slabs = EXCLUDED.price_slabs,
                fulfillment_options = EXCLUDED.fulfillment_options,
                payment_options = EXCLUDED.payment_options,
                categories = EXCLUDED.categories,
                qty = EXCLUDED.qty,
                creator = EXCLUDED.creator,
                matched = EXCLUDED.matched,
                time_to_ship = EXCLUDED.time_to_ship,
                country_of_origin = EXCLUDED.country_of_origin,
                validity = EXCLUDED.validity,
                replacement_terms = EXCLUDED.replacement_terms,
                return_terms = EXCLUDED.return_terms,
                cancellation_terms = EXCLUDED.cancellation_terms,
                long_desc = EXCLUDED.long_desc,
                short_desc  = EXCLUDED.short_desc
                RETURNING id, provider_cache_id, item_id;

            "#,
            &data.ids,
            &data.country_codes[..] as &[&CountryCode], 
            &data.provider_ids[..] as &[&Uuid], 
            &data.domain_codes[..] as &[&CategoryDomain],
            &data.item_codes[..] as &[&str], 
            &data.item_ids[..] as &[&str], 
            &data.item_names[..] as &[&str], 
            &data.currencies[..] as &[&CurrencyType],
            &data.price_with_taxes[..] as &[&BigDecimal],
            &data.price_without_taxes[..] as &[&BigDecimal],
            &data.offered_prices[..] as &[&Option<BigDecimal>],
            &data.maximum_prices[..] as &[&BigDecimal],
            &data.tax_rates[..] as &[&BigDecimal],
            &data.variant_ids[..] as &[Option<Uuid>], 
            &data.recommends,
            &data.attributes,
            &data.images,
            &data.videos,
            &data.price_slabs [..] as &[Option<Value>],
            &data.fulfillment_options,
            &data.categories,
            &data.qtys,
            &data.creators,
            &data.matched,
            &data.time_to_ships[..] as &[&str], 
            &data.country_of_origins[..] as &[Option<&str>], 
            &data.validities,
            &data.replacement_terms,
            &data.return_terms,
            &data.cancellation_terms,
            &data.payment_options,
            &data.created_ons,
            &data.created_ons,
            &data.long_descs as &[&str], 
            &data.short_descs as &[&str], 

        );




        let result = query
            .fetch_all(&mut **transaction)
            .await
            .map_err(|e| {
                tracing::error!("Failed to execute query: {:?}", e);
                anyhow::Error::new(e)
                    .context("A database failure occurred while saving item cache")
            })?;
        for row in result {
            let key = format!("{}_{}", row.provider_cache_id, row.item_id);
            map.insert(key, row.id);
        };
    }
    Ok(map)
}

pub async fn save_cache_to_db(transaction: &mut Transaction<'_, Postgres>, country_code: &CountryCode, domain: &CategoryDomain, product_objs: &WSSearchData,  created_on: DateTime<Utc>) -> Result<DBItemCacheData,anyhow::Error>{
    
    let id = save_np_cache(transaction, &product_objs.bpp, created_on)
        .await?;

    let provider_map = save_provider_cache(
        transaction,
        &product_objs.providers,
        id,
        created_on,
    )
    .await?;

    let location_map = save_provider_location_cache(
        transaction,
        &product_objs.providers,
        &provider_map,
        created_on,
    )
    .await?;
    let location_ids = location_map.values().map(|&id| id).collect();
    let provider_ids = provider_map.values().map(|&id| id).collect();
    let sericability_data = save_provider_location_servicability_cache(transaction, &product_objs.providers, domain, &location_map, &provider_map, created_on)
        .await?;

    let variant_map = save_variant_cache(transaction, &product_objs.providers, &provider_map, created_on).await?;
    let variant_ids = variant_map.values().map(|&id| id).collect();
    let item_map = save_item_cache(transaction, country_code, &product_objs.providers, &provider_map, &variant_map, created_on).await?;
    let item_ids = item_map.values().map(|&id| id).collect();
    save_item_location_relationship_cache(transaction, &product_objs.providers, &provider_map, &location_map, &item_map, created_on).await?;

    Ok(DBItemCacheData{ servicability_ids: sericability_data, network_participant_ids: vec![id], location_ids, provider_ids, variant_ids, item_ids})

}

async fn get_hyperlocal_cache_data_from_db(
    pool: &PgPool,
    id_list: Vec<Uuid>,
) -> Result<Vec<ESHyperlocalServicabilityModel>, anyhow::Error> {
    let query = sqlx::query!(
        r#"
        SELECT 
            shc.id as id,
            shc.provider_location_cache_id as location_cache_id,
            shc.domain_code as "domain_code:CategoryDomain",
            shc.category_code as category_code,
            shc.radius as radius,
            shc.created_on as created_on,
            pc.id AS provider_cache_id,
            pc.network_participant_cache_id as network_participant_cache_id,
            plc.latitude as latitude,
            plc.longitude as longitude
        FROM provider_servicability_hyperlocal_cache AS shc
        JOIN provider_location_cache AS plc
        ON shc.provider_location_cache_id = plc.id
        JOIN provider_cache AS pc
        ON plc.provider_cache_id = pc.id
        WHERE shc.id = ANY($1)
        "#,
        &id_list[..]
    );

    let data = query.fetch_all(pool).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e).context("A database failure occurred while fetching hyperlocal")
    })?;

    let result = data
        .into_iter()
        .map(|record| ESHyperlocalServicabilityModel {
            id: record.id,
            location_cache_id: record.location_cache_id,
            domain_code: record.domain_code,
            category_code: record.category_code,
            radius: record.radius,
            created_on: record.created_on,
            provider_cache_id: record.provider_cache_id,
            network_participant_cache_id: record.network_participant_cache_id,
            location: ESLocationModel {
                lat: record.latitude.to_f64().unwrap(),
                lon: record.longitude.to_f64().unwrap(),
            },
        })
        .collect();

    Ok(result)
}

async fn save_provider_hyperlocal_servicability_cache_to_elastic_search(pool: &PgPool, es_client: &ElasticSearchClient, id_list: Vec<Uuid>)-> Result<(), anyhow::Error>{
    if !id_list.is_empty() {
        let data = get_hyperlocal_cache_data_from_db(pool, id_list).await?;
        let json_values: Vec<JsonBody<_>> = data
        .into_iter()
        .flat_map(|record| {
            let id = record.id.to_string(); 
            vec![
                json!({"index": { "_index": es_client.get_index(&ElasticSearchIndex::ProviderServicabilityHyperLocal.to_string()), "_id": id }}).into(),
                json!(record).into(),
            ]
        })
        .collect();
        es_client.send(&es_client.get_index(&ElasticSearchIndex::ProviderServicabilityHyperLocal.to_string()), json_values).await?;
    }

    Ok(())
}



async fn get_country_cache_data_from_db(
    pool: &PgPool,
    id_list: Vec<Uuid>,
) -> Result<Vec<ESCountryServicabilityModel>, anyhow::Error> {
    let query = sqlx::query_as!(
        ESCountryServicabilityModel,
        r#"
        SELECT 
            shc.id as id,
            shc.provider_location_cache_id as location_cache_id,
            shc.domain_code as "domain_code:CategoryDomain",
            shc.category_code as category_code,
            shc.country_code as "country_code:CountryCode",
            shc.created_on as created_on,
            pc.id AS provider_cache_id,
            pc.network_participant_cache_id as network_participant_cache_id
        FROM provider_servicability_country_cache AS shc
        JOIN provider_location_cache AS plc
        ON shc.provider_location_cache_id = plc.id
        JOIN provider_cache AS pc
        ON plc.provider_cache_id = pc.id
        WHERE shc.id = ANY($1)
        "#,
        &id_list[..]
    );

    let data = query.fetch_all(pool).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e).context("A database failure occurred while fetching country servicability data")
    })?;


    Ok(data)
}

async fn save_provider_country_servicability_cache_to_elastic_search(pool: &PgPool, es_client: &ElasticSearchClient, id_list: Vec<Uuid>)-> Result<(), anyhow::Error>{
    if !id_list.is_empty() {
        let data = get_country_cache_data_from_db(pool, id_list).await?;
        let json_values: Vec<JsonBody<_>> = data
        .into_iter()
        .flat_map(|record| {
            let id = record.id.to_string(); 
            vec![
                json!({"index": { "_index": es_client.get_index(&ElasticSearchIndex::ProviderServicabilityCountry.to_string()), "_id": id }}).into(),
                json!(record).into(),
            ]
        })
        .collect();
        es_client.send(&es_client.get_index(&ElasticSearchIndex::ProviderServicabilityCountry.to_string()), json_values).await?;
    }

    Ok(())
}


async fn get_intercity_cache_data_from_db(
    pool: &PgPool,
    id_list: Vec<Uuid>,
) -> Result<Vec<ESInterCityServicabilityModel>, anyhow::Error> {
    let query = sqlx::query_as!(
        ESInterCityServicabilityModel,
        r#"
        SELECT 
            shc.id as id,
            shc.provider_location_cache_id as location_cache_id,
            shc.domain_code as "domain_code:CategoryDomain",
            shc.category_code as category_code,
            shc.pincode,
            shc.created_on as created_on,
            pc.id AS provider_cache_id,
            pc.network_participant_cache_id as network_participant_cache_id
        FROM provider_servicability_intercity_cache AS shc
        JOIN provider_location_cache AS plc
        ON shc.provider_location_cache_id = plc.id
        JOIN provider_cache AS pc
        ON plc.provider_cache_id = pc.id
        WHERE shc.id = ANY($1)
        "#,
        &id_list[..]
    );

    let data = query.fetch_all(pool).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e).context("A database failure occurred while fetching intecity servicability data")
    })?;


    Ok(data)
}

async fn save_provider_intercity_servicability_cache_to_elastic_search(pool: &PgPool, es_client: &ElasticSearchClient, id_list: Vec<Uuid>)-> Result<(), anyhow::Error>{
    if !id_list.is_empty() {
        let data = get_intercity_cache_data_from_db(pool, id_list).await?;
        let json_values: Vec<JsonBody<_>> = data
        .into_iter()
        .flat_map(|record| {
            let id = record.id.to_string(); 
            vec![
                json!({"index": { "_index": es_client.get_index(&ElasticSearchIndex::ProviderServicabilityInterCity.to_string()), "_id": id }}).into(),
                json!(record).into(),
            ]
        })
        .collect();
        es_client.send(&es_client.get_index(&ElasticSearchIndex::ProviderServicabilityInterCity.to_string()), json_values).await?;
    }

    Ok(())
}


async fn get_geo_json_cache_data_from_db(
    pool: &PgPool,
    id_list: Vec<Uuid>,
) -> Result<Vec<ESGeoJsonServicabilityModel>, anyhow::Error> {
    let query = sqlx::query_as!(
        ESGeoJsonServicabilityModel,
        r#"
        SELECT 
            shc.id as id,
            shc.provider_location_cache_id as location_cache_id,
            shc.domain_code as "domain_code:CategoryDomain",
            shc.category_code as category_code,
            shc.coordinates,
            shc.created_on as created_on,
            pc.id AS provider_cache_id,
            pc.network_participant_cache_id as network_participant_cache_id
        FROM provider_servicability_geo_json_cache AS shc
        JOIN provider_location_cache AS plc
        ON shc.provider_location_cache_id = plc.id
        JOIN provider_cache AS pc
        ON plc.provider_cache_id = pc.id
        WHERE shc.id = ANY($1)
        "#,
        &id_list[..]
    );

    let data = query.fetch_all(pool).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e).context("A database failure occurred while fetching geo_json servicability data")
    })?;


    Ok(data)
}


async fn save_provider_geo_json_servicability_cache_to_elastic_search(pool: &PgPool, es_client: &ElasticSearchClient, id_list: Vec<Uuid>)-> Result<(), anyhow::Error>{
    if !id_list.is_empty() {
        let data = get_geo_json_cache_data_from_db(pool, id_list).await?;
        let json_values: Vec<JsonBody<_>> = data
        .into_iter()
        .flat_map(|record| {
            let id = record.id.to_string(); 
            vec![
                json!({"index": { "_index": es_client.get_index(&ElasticSearchIndex::ProviderServicabilityGeoJson.to_string()), "_id": id }}).into(),
                json!(record).into(),
            ]
        })
        .collect();
        es_client.send(&es_client.get_index(&ElasticSearchIndex::ProviderServicabilityGeoJson.to_string()), json_values).await?;
    }

    Ok(())
}


async fn save_provider_servicability_to_elastic_search(pool: &PgPool, es_client: &ElasticSearchClient, data: ServicabilityIds)-> Result<(), anyhow::Error>{
    save_provider_hyperlocal_servicability_cache_to_elastic_search(pool, es_client, data.hyperlocal).await?;
    save_provider_country_servicability_cache_to_elastic_search(pool, es_client, data.country).await?;
    save_provider_intercity_servicability_cache_to_elastic_search(pool, es_client, data.inter_city).await?;
    save_provider_geo_json_servicability_cache_to_elastic_search(pool, es_client, data.geo_json).await?;
    Ok(())
}


async fn get_network_participant_cache_data_from_db(
    pool: &PgPool,
    id_list: Vec<Uuid>,
) -> Result<Vec<ESNetworkParticipantModel>, anyhow::Error> {
    let query = sqlx::query_as!(
        ESNetworkParticipantModel,
        r#"
        SELECT id, subscriber_id, name, short_desc, long_desc, images, created_on FROM network_participant_cache
        WHERE id = ANY($1)
        "#,
        &id_list[..]
    );

    let data = query.fetch_all(pool).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e).context("A database failure occurred while fetching network participant data")
    })?;


    Ok(data)
}


async fn save_network_participant_to_elastic_search(pool: &PgPool, es_client: &ElasticSearchClient, id_list: Vec<Uuid>)-> Result<(), anyhow::Error>{
    if !id_list.is_empty() {
        let data = get_network_participant_cache_data_from_db(pool, id_list).await?;
        let json_values: Vec<JsonBody<_>> = data
        .into_iter()
        .flat_map(|record| {
            let id = record.id.to_string(); 
            vec![
                json!({"index": { "_index": es_client.get_index(&ElasticSearchIndex::NetworkParticipant.to_string()), "_id": id }}).into(),
                json!(record).into(),
            ]
        })
        .collect();
        es_client.send(&es_client.get_index(&ElasticSearchIndex::NetworkParticipant.to_string()), json_values).await?;
    }
    Ok(())
}


async fn get_provider_location_cache_data_from_db(
    pool: &PgPool,
    id_list: Vec<Uuid>,
) -> Result<Vec<ESProviderLocationModel>, anyhow::Error> {
    let query = sqlx::query_as!(
        ESProviderLocationModel,
        r#"
        SELECT 
            id,
            provider_cache_id,
            location_id,
            latitude,
            longitude,
            address,
            city_code,
            city_name,
            state_code,
            state_name,
            country_code as "country_code:CountryCode",
            country_name,
            area_code,
            created_on,
            updated_on
        FROM provider_location_cache
        WHERE id = ANY($1)
        "#,
        &id_list[..]
    );

    let data = query.fetch_all(pool).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e).context("A database failure occurred while fetching provider location data")
    })?;


    Ok(data)
}



async fn save_location_to_elastic_search(pool: &PgPool, es_client: &ElasticSearchClient, id_list: Vec<Uuid>)-> Result<(), anyhow::Error>{
    if !id_list.is_empty() {
        let data = get_provider_location_cache_data_from_db(pool, id_list).await?;
        let json_values: Vec<JsonBody<_>> = data
        .into_iter()
        .flat_map(|record| {
            let id = record.id.to_string(); 
            vec![
                json!({"index": { "_index": es_client.get_index(&ElasticSearchIndex::ProviderLocation.to_string()), "_id": id }}).into(),
                json!(record).into(),
            ]
        })
        .collect();
        es_client.send(&es_client.get_index(&ElasticSearchIndex::ProviderLocation.to_string()), json_values).await?;
    }
    Ok(())
}



async fn get_provider_cache_data_from_db(
    pool: &PgPool,
    id_list: Vec<Uuid>,
) -> Result<Vec<ESProviderModel>, anyhow::Error> {
    let query = sqlx::query_as!(
        ESProviderModel,
        r#"
        SELECT 
            id, provider_id, network_participant_cache_id, name, code, 
            short_desc, long_desc, images, rating, ttl, 
            credentials, contact, terms, identifications, 
            created_on, updated_on
        FROM provider_cache
        WHERE id = ANY($1)
        "#,
        &id_list[..]
    );

    let data = query.fetch_all(pool).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e).context("A database failure occurred while fetching provider location data")
    })?;


    Ok(data)
}


async fn save_provider_to_elastic_search(pool: &PgPool, es_client: &ElasticSearchClient, id_list: Vec<Uuid>)-> Result<(), anyhow::Error>{
    if !id_list.is_empty() {
        let data = get_provider_cache_data_from_db(pool, id_list).await?;
        let json_values: Vec<JsonBody<_>> = data
        .into_iter()
        .flat_map(|record| {
            let id = record.id.to_string(); 
            vec![
                json!({"index": { "_index": es_client.get_index(&ElasticSearchIndex::Provider.to_string()), "_id": id }}).into(),
                json!(record).into(),
            ]
        })
        .collect();
        es_client.send(&es_client.get_index(&ElasticSearchIndex::Provider.to_string()), json_values).await?;
    }
    Ok(())
}

async fn get_provider_item_variant_cache_data_from_db(
    pool: &PgPool,
    id_list: Vec<Uuid>,
) -> Result<Vec<ESProviderItemVariantModel>, anyhow::Error> {
    let query = sqlx::query_as!(
        ESProviderItemVariantModel,
        r#"
        SELECT 
        id,
        provider_cache_id, variant_id,
        variant_name, attributes, 
        created_on, updated_on
        FROM item_variant_cache
        WHERE id = ANY($1)
        "#,
        &id_list[..]
    );

    let data = query.fetch_all(pool).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e).context("A database failure occurred while fetching provider item variant location data")
    })?;


    Ok(data)
}




async fn save_provider_item_variant_to_elastic_search(pool: &PgPool, es_client: &ElasticSearchClient, id_list: Vec<Uuid>)-> Result<(), anyhow::Error>{
    if !id_list.is_empty() {
        let data = get_provider_item_variant_cache_data_from_db(pool, id_list).await?;
        let json_values: Vec<JsonBody<_>> = data
        .into_iter()
        .flat_map(|record| {
            let id = record.id.to_string(); 
            vec![
                json!({"index": { "_index": es_client.get_index(&ElasticSearchIndex::ProviderItemVariant.to_string()), "_id": id }}).into(),
                json!(record).into(),
            ]
        })
        .collect();
        es_client.send(&es_client.get_index(&ElasticSearchIndex::ProviderItemVariant.to_string()), json_values).await?;
    }
    Ok(())
}







// async fn get_provider_item_cache_data_from_db(
//     transaction: &mut Transaction<'_, Postgres>,
//     id_list: Vec<Uuid>,
// ) -> Result<Vec<ESProviderItemModel>, anyhow::Error> {
//     let item_data = get_provider_item_cache_data_from_db(transaction, id_list).await?;
//     let location_item_mapping = get_provider_item_location_mapping_cache_data_from_db(transaction, id_list).await?;
//     Ok(data)
// }



async fn get_provider_item_cache_data_from_db(
    pool: &PgPool,
    id_list: Vec<Uuid>,
) -> Result<Vec<ESProviderItemModel>, anyhow::Error> {
    let query = sqlx::query_as!(
        ESProviderItemModel,
        r#"
        SELECT 
            ic.provider_cache_id as provider_cache_id,
            ic.id as id,
            ic.country_code  as "country_code:CountryCode",
            ic.domain_code as "domain_code:CategoryDomain",
            ic.item_id,
            ic.item_code,
            ic.long_desc,
            ic.short_desc,
            ic.item_name,
            ic.currency  as "currency:CurrencyType",
            ic.price_with_tax,
            ic.price_without_tax,
            ic.offered_price,
            ic.maximum_price,
            ic.tax_rate,
            ic.variant_cache_id,
            ic.recommended,
            ic.matched,
            ic.attributes,
            ic.images,
            ic.videos,
            ic.price_slabs,
            ic.fulfillment_options,
            ic.payment_options,
            ic.categories,
            ic.qty,
            ic.creator,
            ic.time_to_ship,
            ic.country_of_origin,
            ic.validity,
            ic.replacement_terms,
            ic.return_terms,
            ic.cancellation_terms,
            ic.created_on,
            COALESCE(array_agg(DISTINCT ilcr.location_cache_id) FILTER (WHERE ilcr.location_cache_id IS NOT NULL), '{}') 
            AS location_ids
        FROM item_cache ic
        LEFT JOIN item_location_cache_relationship ilcr 
            ON ic.id = ilcr.item_cache_id
        WHERE ic.id = ANY($1)
        GROUP BY ic.id
        "#,
        &id_list
    );

    let data = query.fetch_all(pool).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e).context("A database failure occurred while fetching provider item data")
    })?;

    Ok(data) 
}




async fn save_provider_item_to_elastic_search(pool: &PgPool, es_client: &ElasticSearchClient, id_list: Vec<Uuid>)-> Result<(), anyhow::Error>{
    if !id_list.is_empty() {
        let data = get_provider_item_cache_data_from_db(pool, id_list).await?;
        let json_values: Vec<JsonBody<_>> = data
        .into_iter()
        .flat_map(|record| {
            let id = record.id.to_string(); 
            vec![
                json!({"index": { "_index": es_client.get_index(&ElasticSearchIndex::ProviderItem.to_string()), "_id": id }}).into(),
                json!(record).into(),
            ]
        })
        .collect();
        es_client.send(&es_client.get_index(&ElasticSearchIndex::ProviderItem.to_string()), json_values).await?;
    }
    Ok(())
}



 pub async fn save_cache_to_elastic_search(pool: &PgPool, es_client: &ElasticSearchClient, data: DBItemCacheData)-> Result<(), anyhow::Error>{
    try_join!(
        save_network_participant_to_elastic_search(pool, es_client, data.network_participant_ids),
        save_provider_to_elastic_search(pool, es_client, data.provider_ids),
        save_location_to_elastic_search(pool, es_client, data.location_ids),
        save_provider_servicability_to_elastic_search(pool, es_client, data.servicability_ids),
        save_provider_item_to_elastic_search(pool, es_client, data.item_ids),
        save_provider_item_variant_to_elastic_search(pool, es_client, data.variant_ids),
    )?;

    Ok(())
}