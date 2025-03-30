#![allow(unused)]

use crate::{
    dtos::relation_tuple::{self, RelationTuple, RelationTupleQuery},
    errors::HeimdallResult,
};

use super::{helpers::build_insert, traits::RelationTupleRepositoryTrait};

use chrono::Utc;
use sqlx::{Pool, Postgres};
use tracing::Level;
use uuid::Uuid;

const CHUNK_SIZE_INSERT_TUPLE: usize = 3000;
const CHUNK_SIZE_DELETE_TUPLE: usize = 100;

pub struct RelationTupleRepository {
    pool: Pool<Postgres>,
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
        _nid: Uuid, //NOTE: Additional context maybe needed here.
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

    async fn write_relation_tuples(
        &self,
        nid: Uuid,
        relation_tuples: &[RelationTuple],
    ) -> HeimdallResult<()> {
        if relation_tuples.is_empty() {
            return Ok(());
        }
        let span = tracing::span!(
            Level::INFO,
            "write_relation_tuples",
            relation_tuples_count = relation_tuples.len()
        );
        let _enter = span.enter();

        let commit_time = Utc::now();

        let mut tx = self.pool.begin().await?;

        for relation_tuple_chunk in relation_tuples.chunks(CHUNK_SIZE_INSERT_TUPLE) {
            let (query_str, args) = build_insert(commit_time, nid, relation_tuples)?;
            let mut query = sqlx::query(&query_str);

            for arg in args {
                query = query.bind(arg);
            }

            query.execute(&mut *tx).await?;
        }

        tx.commit().await?;
        Ok(())
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
