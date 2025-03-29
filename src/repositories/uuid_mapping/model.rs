#![allow(unused)]

use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct UuidMappings {
    pub id: Uuid,
    pub string_representation: String,
}
