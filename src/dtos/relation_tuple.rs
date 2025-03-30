#![allow(unused)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Row, postgres::PgRow};
use uuid::Uuid;

use crate::repositories::relation_tuple::traits::RelationTupleRepositoryTrait;

#[derive(Debug, Serialize, Deserialize)]
pub struct RelationTuples {
    pub shard_id: Uuid,
    pub nid: Uuid,
    pub namespace: String,
    pub object: Uuid,
    pub relation: String,
    pub subject: Subject,
    pub commit_time: DateTime<Utc>,
}

impl<'r> sqlx::FromRow<'r, PgRow> for RelationTuples {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        let shard_id: Uuid = row.try_get("shard_id")?;
        let nid: Uuid = row.try_get("nid")?;
        let namespace: String = row.try_get("namespace")?;
        let object: Uuid = row.try_get("object")?;
        let relation: String = row.try_get("relation")?;
        let commit_time: DateTime<Utc> = row.try_get("commit_time")?;
        let subject_id: Option<Uuid> = row.try_get("subject_id")?;

        let subject: Subject = match subject_id {
            Some(id) => Subject::Direct(SubjectId::new(id)),
            None => {
                let namespace: Option<String> = row.try_get("subject_set_namepace")?;
                let object: Option<Uuid> = row.try_get("subject_set_object")?;
                let relation: Option<String> = row.try_get("subject_set_relation")?;
                Subject::Set(SubjectSet::new(namespace, object, relation))
            }
        };

        Ok(RelationTuples {
            shard_id,
            nid,
            namespace,
            object,
            relation,
            subject,
            commit_time,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Subject {
    Direct(SubjectId),
    Set(SubjectSet),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SubjectId {
    pub id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SubjectSet {
    pub namespace: String,
    pub object: Uuid,
    pub relation: String,
}

impl SubjectId {
    pub fn new(id: Uuid) -> Self {
        Self { id }
    }

    pub fn equals(&self, other: Subject) -> bool {
        match other {
            Subject::Set(_) => false,
            Subject::Direct(other) => self.id.eq(&other.id),
        }
    }

    pub fn unique_id(&self) -> Uuid {
        self.id
    }
}

impl std::fmt::Display for SubjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.id.fmt(f)
    }
}

impl SubjectSet {
    pub fn new(
        namespace_opt: Option<String>,
        object_opt: Option<Uuid>,
        relation_opt: Option<String>,
    ) -> Self {
        let namespace = namespace_opt.unwrap();
        let object = object_opt.unwrap();
        let relation = relation_opt.unwrap();
        Self {
            namespace,
            object,
            relation,
        }
    }

    pub fn equals(&self, other: Subject) -> bool {
        match other {
            Subject::Direct(_) => false,
            Subject::Set(other) => {
                self.relation.eq(&other.relation)
                    && self.namespace.eq(&other.namespace)
                    && self.object.eq(&other.object)
            }
        }
    }

    pub fn unique_id(&self) -> Uuid {
        Uuid::new_v5(
            &self.object,
            format!("{}-{}", self.namespace, self.relation).as_bytes(),
        )
    }
}

impl std::fmt::Display for SubjectSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}#{}", self.namespace, self.object, self.relation)
    }
}

#[derive(Debug, Deserialize)]
pub struct RelationTupleQuery {
    pub namespace: Option<String>,
    pub object: Option<Uuid>,
    pub relation: Option<String>,
    pub subject: Option<Subject>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RelationTuple {
    pub namespace: String,
    pub object: Uuid,
    pub relation: String,
    pub subject: Subject,
}

impl From<RelationTuples> for RelationTuple {
    fn from(value: RelationTuples) -> Self {
        RelationTuple {
            namespace: value.namespace,
            object: value.object,
            relation: value.relation,
            subject: value.subject,
        }
    }
}
