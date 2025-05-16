use std::sync::Arc;

use namespace::Namespace;

pub mod namespace;
pub mod relation;
pub mod subject;

#[derive(Debug)]
pub struct Schema {
    pub namespaces: Arc<Vec<Namespace>>,
}
