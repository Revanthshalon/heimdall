#![allow(unused)]

use std::sync::Arc;

use relation_tuple::{RelationTupleRepository, RelationTupleRepositoryTrait};
use sqlx::{Pool, Postgres};
use uuid_mapping::{UuidMappingRepository, UuidMappingRepositoryTrait};

pub mod relation_tuple;
pub mod uuid_mapping;

pub struct RepositoryFactory {
    relation_tuple_repo: Arc<dyn RelationTupleRepositoryTrait>,
    uuid_mapping_repo: Arc<dyn UuidMappingRepositoryTrait>,
}

impl RepositoryFactory {
    // NOTE: Pass through the configuration that is necessary for the service to connect to the
    // specified database. For now, we are doing a tight coupling with the postgres database
    pub fn new(pool: Pool<Postgres>) -> Self {
        let relation_tuple_repo = RelationTupleRepository::new(pool.clone());
        let uuid_mapping_repo = UuidMappingRepository::new(pool.clone());
        Self {
            relation_tuple_repo: Arc::new(relation_tuple_repo),
            uuid_mapping_repo: Arc::new(uuid_mapping_repo),
        }
    }

    pub fn relation_tuple_repo(&self) -> Arc<dyn RelationTupleRepositoryTrait> {
        self.relation_tuple_repo.clone()
    }

    pub fn uuid_mapping_repo(&self) -> Arc<dyn UuidMappingRepositoryTrait> {
        self.uuid_mapping_repo.clone()
    }
}
