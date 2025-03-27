use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a Network entity for multi-tenancy support.
///
/// Networks allow for logical separation of resources between different tenants
/// in a multi-tenant environment. Each network has its own unique identifier
/// and tracks when it was created and last updated.
#[derive(Debug, Serialize, Deserialize)]
pub struct Network {
    /// Unique identifier for the network
    pub id: Uuid,

    /// Timestamp when the network was created
    pub created_at: DateTime<Utc>,

    /// Timestamp when the network was last updated
    pub update_at: DateTime<Utc>,
}
