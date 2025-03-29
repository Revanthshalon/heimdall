use crate::{
    dtos::relation_tuple::{RelationTuple, RelationTupleQuery},
    errors::HeimdallResult,
};

use super::traits::RelationTupleRepositoryTrait;

use sqlx::{Pool, Postgres};
use tracing::Level;

pub struct RelationTupleRepository {
    pool: Pool<Postgres>,
}

impl RelationTupleRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub fn table_name(&self) -> String {
        String::from("heimdall_relationtuples")
    }
}

#[async_trait::async_trait]
impl RelationTupleRepositoryTrait for RelationTupleRepository {
    async fn get_relation_tuples(
        &self,
        _query: &Option<RelationTupleQuery>,
    ) -> HeimdallResult<Vec<RelationTuple>> {
        let span = tracing::span!(Level::INFO, "get_relation_tuples");
        let _enter = span.enter();
        todo!()
    }

    async fn relation_tuples_exists(
        &self,
        _query: &Option<RelationTupleQuery>,
    ) -> HeimdallResult<bool> {
        todo!()
    }

    async fn create_relation_tuples(
        &self,
        _relation_tuples: &[RelationTuple],
    ) -> HeimdallResult<()> {
        todo!()
    }

    async fn delete_relation_tuples(
        &self,
        _relation_tuples: &[RelationTuple],
    ) -> HeimdallResult<()> {
        todo!()
    }

    async fn delete_all_relation_tuples(
        &self,
        _query: &Option<RelationTupleQuery>,
    ) -> HeimdallResult<()> {
        todo!()
    }

    async fn transact_relation_tuples(
        &self,
        _insert: &[RelationTuple],
        _delete: &[RelationTuple],
    ) -> HeimdallResult<()> {
        todo!()
    }
}
