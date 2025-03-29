use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a permission relationship between a subject and an object.
///
/// A RelationTuple is the core data structure that defines access control
/// relationships within a permission system, typically following the format:
/// "namespace:object#relation@subject"
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct RelationTuples {
    pub shard_id: Uuid,
    pub nid: Uuid,
    pub namespace: String,
    pub object: Uuid,
    pub relation: String,
    pub subject: Subject,
    pub commit_time: DateTime<Utc>,
}

/// Represents either a direct user id or reference to a group.
///
/// A Subject can be either:
/// - A direct user identifier (UUID)
/// - A reference to a group/set of users defined by another relation
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Subject {
    Direct {
        id: Uuid,
    },
    Set {
        namespace: String,
        object: Uuid,
        relation: String,
    },
}

impl std::fmt::Display for Subject {
    /// Formats the Subject as a string:
    /// - For direct IDs: just the UUID
    /// - For sets: "namespace#object.relation"
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Subject::Direct { id } => {
                write!(f, "id: {}", id)
            }
            Subject::Set {
                namespace,
                object,
                relation,
            } => {
                write!(f, "{}#{}.{}", namespace, object, relation)
            }
        }
    }
}

#[allow(unused)]
impl Subject {
    /// Compares two Subjects for equality
    ///
    /// Two subjects are equal if:
    /// - They are both direct IDs with the same UUID, or
    /// - They are both sets with the same namespace, object, and relation
    ///
    /// # Arguments
    ///
    /// * `other` - The Subject to compare with
    ///
    /// # Returns
    ///
    /// `true` if the subjects are equal, `false` otherwise
    pub fn equals(&self, other: &Subject) -> bool {
        match (self, other) {
            (Subject::Direct { id: id1 }, Subject::Direct { id: id2 }) => id1 == id2,
            (
                Subject::Set {
                    namespace: ns1,
                    object: obj1,
                    relation: rel1,
                },
                Subject::Set {
                    namespace: ns2,
                    object: obj2,
                    relation: rel2,
                },
            ) => ns1 == ns2 && obj1 == obj2 && rel1 == rel2,
            _ => false,
        }
    }

    /// Generates a unique identifier for this Subject
    ///
    /// # Returns
    ///
    /// A UUID that uniquely identifies this subject:
    /// - For direct IDs: returns the UUID directly
    /// - For sets: generates a v5 UUID based on namespace and relation
    pub fn unique_id(&self) -> Uuid {
        match self {
            Subject::Direct { id } => *id,
            Subject::Set {
                namespace,
                object: _,
                relation,
            } => {
                let namespace_string = format!("{}-{}", namespace, relation);
                Uuid::new_v5(&Uuid::NAMESPACE_OID, namespace_string.as_bytes())
            }
        }
    }
}
