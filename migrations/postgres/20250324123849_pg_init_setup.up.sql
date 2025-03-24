CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ## Zanzibar-Style Relationship and Permission Modeling
--
-- This SQL schema implements a simplified version of Google Zanzibar's authorization model.
-- The following table summarizes the core rule types and how they are evaluated.
--
-- | Rule Type         | Zanzibar Example                                                     | Request Example                                  | Evaluation Steps (using this SQL schema)                                                                                                                                                                                                                                                                                                 |
-- | ----------------- | -------------------------------------------------------------------- | ------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
-- | `direct`          | `(document:doc1, owner, user:alice)`                                | "Is user:alice the owner of document:doc1?"        | 1. Look up `relation_rules`: `namespace_id = 'document_ns'`, `relation_name = 'owner'`, expect `rule_type = 'direct'`.<br>2. Query `relationship_tuples`: Match `namespace_id`, `object_id`, `relation`, `subject_type`, `subject_id`.<br>3. Return `true` if tuple found, `false` otherwise.                         |
-- | `tuple_to_userset` | `document:doc1, reader, group:eng#member`                         | "Is user:bob a reader of document:doc1?"          | 1. Look up `relation_rules`: `namespace_id = 'document_ns'`, `relation_name = 'reader'`, expect `rule_type = 'tuple_to_userset'`. Get `ttu_object_namespace` and `ttu_relation`.<br>2. Check direct `reader` tuples for `user:bob`.<br>3. If not found, find `reader` tuples with `subject_type = 'group'`.<br>4. For each such group (e.g., 'eng'), check `relationship_tuples` for membership: `namespace_id = ttu_object_namespace`,  `object_id = group_id`, `relation = ttu_relation`, `subject_id = 'bob'`.<br>5. Return `true` if *any* group membership check succeeds, `false` otherwise. |
-- | `union`           | `permission view = reader + owner`                                   | "Can user:carol view document:doc1?"             | 1. Look up `relation_rules`: `namespace_id = 'document_ns'`, `relation_name = 'view'`, expect `rule_type = 'union'`. Get `child_relations` (e.g., `["reader", "owner"]`).<br>2. For *each* relation in `child_relations`, check if the user has that relation to the object (as if checking a `direct` relation).<br>3. Return `true` if *any* of the child relation checks succeed, `false` otherwise.                                  |
-- | `intersection`    | `permission edit = editor & owner`                                  | "Can user:eve edit document:doc1?"                | 1. Look up `relation_rules`: `namespace_id = 'document_ns'`, `relation_name = 'edit'`, expect `rule_type = 'intersection'`. Get `child_relations` (e.g., `["editor", "owner"]`).<br>2. For *each* relation in `child_relations`, check if the user has that relation to the object.<br>3. Return `true` only if *all* of the child relation checks succeed, `false` otherwise.                          |
-- | `exclusion`       | `permission comment = viewer - editor`                               | "Can user:frank comment on document:doc1?"        | 1. Look up `relation_rules`: `namespace_id = 'document_ns'`, `relation_name = 'comment'`, expect `rule_type = 'exclusion'`. Get `child_relations` (e.g., `["viewer", "editor"]`).<br>2. Check if the user has the *first* relation (`viewer`).<br>3. Check if the user has the *second* relation (`editor`).<br>4. Return `true` only if the first check succeeds *and* the second check fails, `false` otherwise.                      |
--
-- **Key Tables:**
--
-- *   `namespaces`:  Defines object types (e.g., `user`, `document`, `group`).
-- *   `relations`:  Defines relation names within a namespace (e.g., `owner`, `reader`, `member`).
-- *   `relation_rules`: Defines the *logic* of relations (direct, tuple-to-userset, union, intersection, exclusion).
-- *   `relationship_tuples`: Stores the direct relationship facts.

CREATE TYPE rule_type AS ENUM (
  'direct',
  'union',
  'intersection',
  'exclusion',
  'tuple_to_userset'
);

CREATE TABLE IF NOT EXISTS namespaces (
  id VARCHAR(64) PRIMARY KEY,
  name VARCHAR(255) NOT NULL UNIQUE,
  description TEXT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  deleted_at TIMESTAMPTZ NULL
);

CREATE TABLE IF NOT EXISTS relations (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  namespace_id VARCHAR(64) NOT NULL REFERENCES namespaces(id) ON DELETE RESTRICT,
  name VARCHAR(64) NOT NULL,
  description TEXT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  deleted_at TIMESTAMPTZ NULL,
  UNIQUE (namespace_id, name)
);
