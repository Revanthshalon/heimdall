use crate::{
    dtos::relation_tuple::{RelationTuple, RelationTupleQuery},
    errors::HeimdallResult,
};

#[async_trait::async_trait]
pub trait RelationTupleRepositoryTrait: Send + Sync {
    async fn get_relation_tuples(
        &self,
        query: &Option<RelationTupleQuery>,
    ) -> HeimdallResult<Vec<RelationTuple>>;
    async fn relation_tuples_exists(
        &self,
        query: &Option<RelationTupleQuery>,
    ) -> HeimdallResult<bool>;
    async fn create_relation_tuples(&self, relation_tuples: &[RelationTuple])
    -> HeimdallResult<()>;
    async fn delete_relation_tuples(&self, relation_tuples: &[RelationTuple])
    -> HeimdallResult<()>;
    async fn delete_all_relation_tuples(
        &self,
        query: &Option<RelationTupleQuery>,
    ) -> HeimdallResult<()>;
    async fn transact_relation_tuples(
        &self,
        insert: &[RelationTuple],
        delete: &[RelationTuple],
    ) -> HeimdallResult<()>;
}
