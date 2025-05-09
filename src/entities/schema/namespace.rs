use std::sync::Arc;

use super::relation::Relation;

#[derive(Debug, Clone)]
pub struct Namespace {
    pub name: Arc<str>,
    pub relation: Arc<Vec<Relation>>,
}
