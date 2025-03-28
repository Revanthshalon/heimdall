#![allow(unused)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::relation_tuple::Subject;

#[derive(Debug, Deserialize)]
pub struct RelationTupleQuery {
    pub namespace: Option<String>,
    pub object: Option<Uuid>,
    pub relation: Option<String>,
    #[serde(rename = "suject_id")]
    pub subject: Option<Subject>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RelationTuple {
    pub namespace: String,
    pub object: Uuid,
    pub relation: String,
    pub subject: Subject,
}
