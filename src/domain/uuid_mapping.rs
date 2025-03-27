/// A struct that maps between a UUID and its string representation.
///
/// This struct is used for serialization and deserialization using the serde framework.
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a mapping between a UUID and its string representation.
///
/// # Fields
///
/// * `id` - The UUID value
/// * `string_representation` - String representation of the UUID
#[derive(Debug, Serialize, Deserialize)]
pub struct UuidMapping {
    /// The UUID identifier
    pub id: Uuid,
    /// String representation of the UUID
    pub string_representation: String,
}
