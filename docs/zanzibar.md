# Heimdall: A PostgreSQL Implementation of Google Zanzibar Authorization

This migration establishes the foundational database schema for Heimdall, a PostgreSQL implementation of Google's Zanzibar authorization system. Zanzibar is a globally distributed, highly available authorization system that Google uses to manage permissions for billions of objects across diverse services.

## Core Concepts of Zanzibar

Google Zanzibar models permissions through "relation tuples" which define who has what permissions on which resources. The model follows the form:

```
{namespace}:{object}#{relation}@{subject}
```

For example: `document:doc123#viewer@user456` means "user456 has viewer permission on document doc123".

## Key Tables in the Schema

### 1. heimdall_relation_tuples

This is the heart of the Zanzibar implementation, storing all permission relationships. The table stores two types of relationships:

- **Direct relationships**: A specific user has a specific permission on a resource
- **Subject set relationships**: Members of a group have specific permissions on a resource

The table structure handles both cases through a constraint that enforces either:
- `subject_id` is populated (direct relationship)
- OR the `subject_set_*` fields are populated (group-based relationship)

The design supports the core Zanzibar concept of "subject sets," which allows for permission inheritance and group-based access control.

### 2. networks

This table enables multi-tenancy, allowing completely isolated authorization domains to coexist in the same database. Each network represents a separate tenant with its own permission model and relationships. The `nid` foreign key in the relation tuples enforces this isolation.

### 3. heimdall_uuid_mappings

To make the system more human-friendly, this table translates the technical UUIDs used throughout the system to readable identifiers, facilitating debugging and user interface presentation.

### 4. schema_migration

Standard migration tracking table for managing database schema versions over time.

## Index Strategy

The migration creates several carefully designed indexes to support efficient permission checking:

1. **Full tuple lookups**: The `heimdall_relation_tuples_full_idx` supports complete tuple matching.

2. **Direct permission checks**: The `heimdall_relation_tuples_subject_ids_idx` optimizes the common case of checking if a specific user has a specific permission on a resource.

3. **Group-based permission checks**: The `heimdall_relation_tuples_subject_sets_idx` accelerates checking if members of a group have particular permissions.

4. **Reverse lookups**: Two indexes (`heimdall_relation_tuples_reverse_subject_ids_idx` and `heimdall_relation_tuples_reverse_subject_sets_idx`) support efficiently answering "What can this user/group do?" queries.

Each index includes a WHERE clause to limit its scope to the relevant subset of rows, optimizing both index size and query performance.

## Sharding and Scaling Design

The schema supports horizontal scaling through sharding. The `shard_id` column in the relation tuples table allows partitioning permission data across multiple database instances, essential for high-volume authorization systems.

## Consistency and Integrity

The schema enforces referential integrity with foreign key constraints, ensuring that relation tuples can only reference valid networks, with cascading deletes ensuring clean removal of a network's permission data when the network itself is deleted.

## System Benefits

This implementation provides:

1. **Performance**: Highly optimized for fast permission lookups through targeted indexes
2. **Flexibility**: Support for complex permission models through direct and group-based relationships
3. **Scalability**: Horizontal scaling via sharding and efficient query patterns
4. **Multi-tenancy**: Complete isolation between different authorization domains
5. **Consistency**: Strong data integrity through constraints and foreign keys

By implementing Zanzibar in PostgreSQL, this schema delivers a robust authorization system capable of handling complex permission structures while maintaining high performance.
