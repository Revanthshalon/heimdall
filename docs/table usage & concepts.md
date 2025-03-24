# Google Zanzibar and Its Implementation in Heimdall Schema

## What is Google Zanzibar?

Google Zanzibar is a globally distributed authorization system that powers permissions for Google services like Google Drive, YouTube, and Google Photos. It provides consistent, low-latency permission checking while handling billions of access control list entries across many services.

## Key Concepts of Zanzibar

### 1. Relation Tuples
At its core, Zanzibar stores relationships as tuples in the form: `object#relation@subject`. This means "subject has relation to object".

In the schema, this is implemented in the `relationship_tuples` table, storing:
- Which object (`namespace_id` + `object_id`)
- What permission (`relation`)
- For which subject (`subject_type` + `subject_id`)

### 2. Namespaces
Objects are organized into namespaces, which define object types (like document, folder, project).

### 3. Relation Configurations
Each namespace has specific relations (permissions) that can be assigned, like "viewer", "editor", "owner".

### 4. Userset Rewriting
The most powerful aspect of Zanzibar is how it computes complex permissions through rules. The schema supports five rule types in the `relation_rules` table:
- `direct`: Simple assignment (user X has permission Y)
- `union`: OR logic (combines multiple permission sets)
- `intersection`: AND logic (requires all permissions)
- `exclusion`: Subtraction (everyone except blocked users)
- `tuple-to-userset`: References other objects' relations (like group membership)

### 5. Consistency
Zanzibar uses "zookies" (consistency tokens) to ensure that permission checks are consistent even in a distributed system. The schema implements this in the `zookies` table.

## Practical Examples

### Example 1: Document Sharing (Direct Permission)

Let's say Alice wants to share a document with Bob as a viewer:

1. First, the system would have these namespace and relation definitions:
```
-- In namespaces table
id: "document", name: "Documents"

-- In relations table
namespace_id: "document", name: "viewer"
```

2. The permission would be stored as:
```
-- In relationship_tuples table
namespace_id: "document"
object_id: "doc123"
relation: "viewer"
subject_type: "user"
subject_id: "bob@example.com"
zookie_token: "zk_12345"  -- consistency token
```

3. When Bob tries to access the document, the system executes:
```sql
SELECT COUNT(*) > 0 FROM relationship_tuples
WHERE namespace_id = 'document'
AND object_id = 'doc123'
AND relation = 'viewer'
AND subject_type = 'user'
AND subject_id = 'bob@example.com';
```

### Example 2: Group-based Permissions (Tuple-to-Userset)

Now let's say we want anyone in the "Engineering" group to have editor access to a project:

1. Define namespaces and relations:
```
-- In namespaces table
id: "group", name: "Groups"
id: "project", name: "Projects"

-- In relations table
namespace_id: "group", name: "member"
namespace_id: "project", name: "editor"
```

2. Define the rule that project editors include group members:
```
-- In relation_rules table
namespace_id: "project"
relation_name: "editor"
rule_type: "tuple-to-userset"
ttu_object_namespace: "group"
ttu_relation: "member"
```

3. Set up the relationship between the project and the group:
```
-- In relationship_tuples table
namespace_id: "project"
object_id: "proj456"
relation: "editor"
subject_type: "group"
subject_id: "engineering"
```

4. Set up group membership:
```
-- In relationship_tuples table
namespace_id: "group"
object_id: "engineering"
relation: "member"
subject_type: "user"
subject_id: "charlie@example.com"
```

5. When Charlie tries to edit the project, the system checks:
   - Is Charlie an editor of proj456? No direct entry.
   - The rule says: check if Charlie is a member of any group that is an editor of proj456
   - The system finds engineering group is an editor of proj456
   - Then checks if Charlie is a member of engineering: Yes
   - Permission granted

### Example 3: Complex Permission (Union and Exclusion)

Imagine we want to define a "collaborator" relation that includes both viewers and editors, but excludes blocked users:

```
-- In relation_rules table
namespace_id: "document"
relation_name: "collaborator"
rule_type: "union"
child_relations: ["viewer", "editor"]
expression: "viewer + editor"
```

And another rule:
```
-- In relation_rules table
namespace_id: "document"
relation_name: "active_collaborator"
rule_type: "exclusion"
child_relations: ["collaborator", "blocked"]
expression: "collaborator - blocked"
```

When checking if a user is an active_collaborator, the system:
1. First checks if they're in the collaborator set (either a viewer OR editor)
2. Then verifies they're not in the blocked set

## Advanced Features in the Schema

The schema also implements:

1. **Auditing**: All changes are tracked in `audit_log` and `transaction_log` tables
2. **Performance Optimization**: Pre-computed results stored in `permissions_cache`
3. **Monitoring**: Permission checks recorded in `auth_decisions` for analysis
4. **Distributed Consistency**: Managed via `zookies` and `replication_status` tables

This implementation provides a robust, scalable foundation for building complex authorization systems similar to what powers Google's products, with strong consistency guarantees even in distributed environments.
