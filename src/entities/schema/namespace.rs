use std::sync::Arc;

use super::relation::Relation;

#[derive(Debug, Clone)]
pub struct Namespace {
    pub name: Arc<str>,
    pub relations: Arc<Vec<Relation>>,
}

impl Namespace {
    pub fn new(name: Arc<str>, relations: Vec<Relation>) -> Self {
        Self {
            name,
            relations: Arc::new(relations),
        }
    }
}
