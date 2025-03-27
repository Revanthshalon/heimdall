use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a permission relationship between a subject and an object.
///
/// A RelationTuple is the core data structure that defines access control
/// relationships within a permission system, typically following the format:
/// "namespace:object#relation@subject"
#[derive(Debug, Serialize, Deserialize)]
pub struct RelationTuple {
    /// Unique identifier for the shard this relationship belongs to
    pub shard_id: Uuid,

    /// Network ID for multi-tenancy - identifies the tenant
    pub nid: Uuid,

    /// The namespace (category/domain) of the object
    pub namespace: String,

    /// The specific object identifier
    pub object: Uuid,

    /// The type of relationship (e.g., "owner", "viewer", "editor")
    pub relation: String,

    /// The subject (user or group) that has the relation to the object
    pub subject: Subject,

    /// Timestamp when this relation was committed/created
    pub commit_time: DateTime<Utc>,
}

impl std::fmt::Display for RelationTuple {
    /// Formats the RelationTuple as a string in the canonical format:
    /// "namespace:object#relation@subject"
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}#{}@{}",
            self.namespace, self.object, self.relation, self.subject
        )
    }
}

/// Represents either a direct user id or reference to a group.
///
/// A Subject can be either:
/// - A direct user identifier (UUID)
/// - A reference to a group/set of users defined by another relation
#[derive(Debug, Serialize, Deserialize)]
pub enum Subject {
    /// Direct Subject representing a specific user by their UUID
    Id(Uuid),

    /// Subject set representing group membership
    /// References another relation that defines a collection of subjects
    Set {
        /// The namespace of the group
        namespace: String,

        /// The object identifier of the group
        object: Uuid,

        /// The relation type within the group (e.g., "member")
        relation: String,
    },
}

impl std::fmt::Display for Subject {
    /// Formats the Subject as a string:
    /// - For direct IDs: just the UUID
    /// - For sets: "namespace#object.relation"
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Subject::Id(id) => write!(f, "{}", id),
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
            (Subject::Id(id1), Subject::Id(id2)) => id1 == id2,
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
            Subject::Id(id) => *id,
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
