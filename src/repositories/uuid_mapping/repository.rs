use std::collections::HashMap;

use sqlx::{Pool, Postgres};
use tracing::Level;
use uuid::Uuid;

use super::helpers::build_insert_uuids;
use super::traits::UuidMappingRepositoryTrait;
use crate::dtos::uuid_mapping::UuidMappings;
use crate::errors::HeimdallResult;

pub struct UuidMappingRepository {
    pool: Pool<Postgres>,
}

impl UuidMappingRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub async fn batch_from_uuids(&self, uuids: &[Uuid]) -> HeimdallResult<Vec<String>> {
        if uuids.is_empty() {
            return Ok(Vec::new());
        }

        tracing::trace!("looking up UUIDS");

        let mut id_idx: HashMap<Uuid, Vec<usize>> = HashMap::with_capacity(uuids.len());

        for (i, id) in uuids.iter().enumerate() {
            id_idx.entry(*id).or_default().push(i);
        }

        let mut result = Vec::with_capacity(uuids.len());
        result.resize_with(uuids.len(), String::new);

        // TODO: Implement dynamic pagination once we start getting pagination related details.
        let chunk_size = 100;

        for uuid_chunk in id_idx.keys().collect::<Vec<&Uuid>>().chunks(chunk_size) {
            let uuid_params = uuid_chunk.iter().map(|&id| *id).collect::<Vec<Uuid>>();

            let mappings_list: Vec<UuidMappings> = sqlx::query_as(
                "SELECT id, string_representation FROM heimdall_uuid_mappings WHERE id = ANY($1)",
            )
            .bind(uuid_params)
            .fetch_all(&self.pool)
            .await?;

            for mapping in mappings_list {
                if let Some(indices) = id_idx.get(&mapping.id) {
                    for &idx in indices {
                        result[idx] = mapping.string_representation.clone();
                    }
                }
            }
        }
        Ok(result)
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

        let span = tracing::span!(
            Level::INFO,
            "map_string_to_uuids",
            string_count = strings.len()
        );
        let _enter = span.enter();

        let uuids = self.map_string_to_uuids_readonly(nid, strings).await?;

        tracing::debug!(strings= ?strings, uuids= ?uuids, "adding UUID mappings");

        let mut mappings = Vec::with_capacity(strings.len());

        for (string_representation, id) in strings.iter().zip(uuids.iter()) {
            mappings.push(UuidMappings {
                id: *id,
                string_representation: string_representation.clone(),
            });
        }

        mappings.sort_by(|a, b| a.id.cmp(&b.id));
        mappings.dedup_by(|a, b| a.id == b.id);

        span.record("mappings_length", mappings.len());

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

    // note: this function does not make calls to database
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

    // this function looks for existing mappings of uuids to string representations and returns an
    // error of not present
    async fn map_uuid_to_strings(&self, uuids: &[Uuid]) -> HeimdallResult<Vec<String>> {
        let span = tracing::span!(Level::INFO, "map_uuid_to_strings");
        let _enter = span.enter();
        self.batch_from_uuids(uuids).await
    }
}
