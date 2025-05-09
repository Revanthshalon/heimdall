use std::sync::Arc;

use super::subject::SubjectSetRewrite;

#[derive(Debug, Clone)]
pub struct Relation {
    pub name: Arc<str>,
    pub relation_type: Vec<RelationType>,
    pub subject_set_rewrite: Option<Arc<SubjectSetRewrite>>,
}

#[derive(Debug, Clone)]
pub struct RelationType {
    pub namespace: Arc<str>,
    pub relation: Option<Arc<str>>,
}
