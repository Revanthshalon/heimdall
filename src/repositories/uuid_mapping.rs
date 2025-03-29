#![allow(unused)]

use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::{
    entities::uuid_mapping::UuidMappings,
    errors::{HeimdallError, HeimdallResult},
};

pub struct UuidMappingRepository {
    pool: Pool<Postgres>,
}

impl UuidMappingRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
pub trait UuidMappingRepositoryTrait: Send + Sync {
    async fn map_string_to_uuids(
        &self,
        nid: &Uuid,
        strings: &[String],
    ) -> HeimdallResult<Vec<Uuid>>;
    async fn map_string_to_uuids_readonly(
        &self,
        nid: &Uuid,
        strings: &[String],
    ) -> HeimdallResult<Vec<Uuid>>;
    async fn map_uuid_to_strings(&self, uuids: &[Uuid]) -> HeimdallResult<Vec<String>>;
}

impl UuidMappingRepository {
    pub fn table_name(&self) -> String {
        String::from("heimdall_uuid_mappings")
    }
}

impl UuidMappingRepository {
    pub async fn batch_from_uuids(&self, uuids: &[Uuid]) -> HeimdallResult<Vec<String>> {
        if uuids.is_empty() {
            return Ok(Vec::new());
        }
        todo!()
    }
}

#[async_trait::async_trait]
impl UuidMappingRepositoryTrait for UuidMappingRepository {
    // This function only looks for existing mappings of strings to uuids and creates uuids for
    // mappings that does not exist
    async fn map_string_to_uuids(
        &self,
        nid: &Uuid,
        strings: &[String],
    ) -> HeimdallResult<Vec<Uuid>> {
        if strings.is_empty() {
            return Ok(Vec::new());
        }

        let uuids = self.map_string_to_uuids_readonly(nid, strings).await?;

        let mut mappings = Vec::with_capacity(strings.len());

        for (string_representation, id) in strings.iter().zip(uuids.iter()) {
            mappings.push(UuidMappings {
                id: *id,
                string_representation: string_representation.clone(),
            });
        }

        mappings.sort_by(|a, b| a.id.cmp(&b.id));
        mappings.dedup_by(|a, b| a.id == b.id);

        let chunk_size = 1000;
        for chunk in mappings.chunks(chunk_size) {
            let (query_str, args) = build_insert_uuids(chunk);
            let mut query = sqlx::query(&query_str);

            for arg in args {
                query = query.bind(arg);
            }

            query.execute(&self.pool).await?;
        }

        Ok(uuids)
    }

    // NOTE: This function does not make calls to database
    async fn map_string_to_uuids_readonly(
        &self,
        nid: &Uuid,
        strings: &[String],
    ) -> HeimdallResult<Vec<Uuid>> {
        let result = strings
            .iter()
            .map(|s| Uuid::new_v5(nid, s.as_bytes()))
            .collect();
        Ok(result)
    }

    // This function looks for existing mappings of uuids to string representations and returns an
    // error of not present
    async fn map_uuid_to_strings(&self, uuids: &[Uuid]) -> HeimdallResult<Vec<String>> {
        if uuids.is_empty() {
            return Ok(Vec::new());
        }

        let mut results = Vec::with_capacity(uuids.len());

        for id in uuids {
            let query_result_option: Option<String> = sqlx::query_scalar(
                "SELECT string_representation FROM heimdall_uuid_mappings WHERE id = $1",
            )
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

            match query_result_option {
                Some(string_representation) => results.push(string_representation),
                None => return Err(HeimdallError::NoStringForUuid(*id)),
            }
        }
        Ok(results)
    }
}

fn build_insert_uuids(uuid_mappings: &[UuidMappings]) -> (String, Vec<String>) {
    if uuid_mappings.is_empty() {
        return (String::new(), Vec::new());
    }

    let mut query_builder = String::new();
    let mut args: Vec<String> = Vec::with_capacity(uuid_mappings.len() * 2);

    query_builder
        .push_str("INSERT INTO heimdall_uuid_mappings (id, string_representation) VALUES ");

    for (i, uuid_mapping) in uuid_mappings.iter().enumerate() {
        if i > 0 {
            query_builder.push_str(", ");
        }
        let param_placeholder = format!("(${}, ${})", (i * 2) + 1, (i * 2) + 2);
        query_builder.push_str(&param_placeholder);
        args.push(uuid_mapping.id.to_string());
        args.push(uuid_mapping.string_representation.clone())
    }

    query_builder.push_str("ON CONFLICT (id) DO NOTHING");

    (query_builder, args)
}
