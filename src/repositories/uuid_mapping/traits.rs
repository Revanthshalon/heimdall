use uuid::Uuid;

use crate::errors::HeimdallResult;

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
