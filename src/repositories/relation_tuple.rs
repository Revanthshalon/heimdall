#![allow(unused)]

// WARN: None of the repository methods are implemented yet
use sqlx::{Pool, Postgres};

use crate::{
    dtos::relation_tuple::{RelationTuple, RelationTupleQuery},
    errors::HeimdallResult,
};

pub struct RelationTupleRepository {
    pool: Pool<Postgres>,
}

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

impl RelationTupleRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl RelationTupleRepositoryTrait for RelationTupleRepository {
    async fn get_relation_tuples(
        &self,
        _query: &Option<RelationTupleQuery>,
    ) -> HeimdallResult<Vec<RelationTuple>> {
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
