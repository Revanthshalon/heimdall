use std::sync::Arc;

use super::subject::SubjectSetRewrite;

#[derive(Debug, Clone)]
pub struct Relation {
    pub name: Arc<str>,
    pub relation_type: Arc<Vec<RelationType>>,
    pub subject_set_rewrite: Option<Arc<SubjectSetRewrite>>,
}

#[derive(Debug, Clone)]
pub enum RelationType {
    Reference {
        namespace: Arc<str>,
        relation: Option<Arc<str>>,
    },
    Attribute(AttributeType),
}

#[derive(Debug, Clone, Copy)]
pub enum AttributeType {
    Boolean,
    String,
}
