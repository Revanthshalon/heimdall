#![allow(unused)]

use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::errors::{HeimdallError, HeimdallResult};

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
    async fn map_string_to_uuids(&self, strings: &[String]) -> HeimdallResult<Vec<Uuid>>;
    async fn map_string_to_uuids_readonly(&self, strings: &[String]) -> HeimdallResult<Vec<Uuid>>;
    async fn map_uuid_to_strings(&self, uuids: &[Uuid]) -> HeimdallResult<Vec<String>>;
}

#[async_trait::async_trait]
impl UuidMappingRepositoryTrait for UuidMappingRepository {
    // This function only looks for existing mappings of strings to uuids and creates uuids for
    // mappings that does not exist
    async fn map_string_to_uuids(&self, strings: &[String]) -> HeimdallResult<Vec<Uuid>> {
        if strings.is_empty() {
            return Ok(Vec::new());
        }

        let mut results = Vec::with_capacity(strings.len());

        for string in strings {
            let query_result_option: Option<Uuid> = sqlx::query_scalar(
                "SELECT id FROM heimdall_uuid_mappings WHERE string_representation = $1",
            )
            .bind(string)
            .fetch_optional(&self.pool)
            .await?;

            let id = if let Some(id) = query_result_option {
                id
            } else {
                let new_id = Uuid::new_v4();
                sqlx::query(
                    "INSERT INTO heimdall_uuid_mappings (id, string_representation)
                     VALUES ($1, $2)",
                )
                .bind(new_id)
                .bind(string)
                .execute(&self.pool)
                .await?;
                new_id
            };

            results.push(id);
        }
        Ok(results)
    }

    // This function only looks for existing mappings of strings to uuids and returns an error of
    // not present
    async fn map_string_to_uuids_readonly(&self, strings: &[String]) -> HeimdallResult<Vec<Uuid>> {
        if strings.is_empty() {
            return Ok(Vec::new());
        }

        let mut results = Vec::with_capacity(strings.len());

        for string in strings {
            let query_result_option: Option<Uuid> = sqlx::query_scalar(
                "SELECT id FROM heimdall_uuid_mappings WHERE string_representation = $1",
            )
            .bind(string)
            .fetch_optional(&self.pool)
            .await?;
            match query_result_option {
                Some(id) => results.push(id),
                None => return Err(HeimdallError::NoUuidForString(string.to_string())),
            }
        }
        Ok(results)
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
