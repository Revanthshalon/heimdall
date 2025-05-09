use chrono::{DateTime, Utc};
use sqlx::{FromRow, Row, postgres::PgRow};
use uuid::Uuid;

#[derive(Debug)]
pub struct RelationTuple {
    pub shard_id: Uuid,
    pub nid: Uuid,
    pub namespace: Namespace,
    pub relation: String,
    pub subject: Subject,
    pub commit_time: DateTime<Utc>,
}

impl std::fmt::Display for RelationTuple {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}#{}@{}",
            self.namespace.name, self.namespace.object, self.relation, self.subject
        )
    }
}

//TODO: Implement Into<Proto> for Relation Tuple

#[derive(Debug)]
pub struct Namespace {
    pub name: String,
    pub object: Uuid,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Subject {
    Direct {
        id: Uuid,
    },
    SubjectSet {
        namespace: String,
        object: Uuid,
        relation: String,
    },
}

impl Subject {
    pub fn unique_id(&self) -> Uuid {
        match self {
            Self::Direct { id } => *id,
            Self::SubjectSet {
                namespace,
                object,
                relation,
            } => Uuid::new_v5(object, format!("{}-{}", namespace, relation).as_bytes()),
        }
    }
}

impl std::fmt::Display for Subject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Direct { id } => id.fmt(f),
            Self::SubjectSet {
                namespace,
                object,
                relation,
            } => write!(f, "{}:{}#{}", namespace, object, relation),
        }
    }
}

impl<'r> FromRow<'r, PgRow> for RelationTuple {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        // Getting the values for the model
        let shard_id = row.try_get("shard_id")?;
        let nid = row.try_get("nid")?;
        let namespace_name = row.try_get("namespace")?;
        let namespace_object = row.try_get("object")?;
        let relation = row.try_get("relation")?;
        let subject = match row.try_get("subject_id")? {
            Some(id) => Subject::Direct { id },
            None => {
                let subject_set_namespace = row.try_get("subject_set_namespace")?;
                let subject_set_object = row.try_get("subject_set_object")?;
                let subject_set_relation = row.try_get("subject_set_relation")?;
                Subject::SubjectSet {
                    namespace: subject_set_namespace,
                    object: subject_set_object,
                    relation: subject_set_relation,
                }
            }
        };
        let commit_time = row.try_get("commit_time")?;

        Ok(RelationTuple {
            shard_id,
            nid,
            namespace: Namespace {
                name: namespace_name,
                object: namespace_object,
            },
            relation,
            subject,
            commit_time,
        })
    }
}
