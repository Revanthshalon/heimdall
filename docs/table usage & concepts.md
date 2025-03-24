**Key Concepts and Table Usage**

-   **`namespaces`:**  Defines the object types (e.g.,  `user`,  `document`,  `group`).
-   **`relations`:**  Defines the  _names_  of relations within a namespace (e.g.,  `owner`,  `reader`,  `member`). It doesn't store the  _rules_  for how relations are defined.
-   **`relation_rules`:**  This is the  _core_  table for defining the  _logic_  of relations, using your  `rule_type`  enum. This is where we capture the "direct," "union," "tuple-to-userset," etc., nature of a relation.
-   **`relationship_tuples`:**  Stores the  _direct_  relationships (the facts). It also includes fields (`userset_namespace`,  `userset_relation`) to handle the  `tuple_to_userset`  cases, referencing  _another_  object's relation.
-   `zookie_token`: Used for ensuring consistency.
-   `ttu_object_namespace`,  `ttu_relation`: For  `tuple_to_userset`, this specifies the related object and relation.
-   `child_relations`: For  `union`,  `intersection`,  `exclusion`, this  `JSONB`  field stores an array of relation names that are combined.
-   `expression`: This stores the string representation of the expression.

**Scenario 1: Direct Relationship**

-   **Zanzibar Scenario:**  A user is directly assigned as the owner of a document.
    
-   **Zanzibar Example:**  `(document:doc1, owner, user:alice)`
    
-   **SQL Inserts:**
    
    SQL
    
    ```
    -- 1. Add namespaces (if they don't exist)
    INSERT INTO namespaces (id, name) VALUES ('user_ns', 'user') ON CONFLICT DO NOTHING;
    INSERT INTO namespaces (id, name) VALUES ('document_ns', 'document') ON CONFLICT DO NOTHING;
    
    -- 2. Add the 'owner' relation to the 'document' namespace
    INSERT INTO relations (namespace_id, name) VALUES ('document_ns', 'owner') ON CONFLICT DO NOTHING;
    
    -- 3. Define the 'owner' relation as 'direct'
    INSERT INTO relation_rules (namespace_id, relation_name, rule_type, expression)
    VALUES ('document_ns', 'owner', 'direct', 'owner');
    
    -- 4. Create the relationship tuple
    INSERT INTO relationship_tuples (namespace_id, object_id, relation, subject_type, subject_id, zookie_token)
    VALUES ('document_ns', 'doc1', 'owner', 'user', 'alice', 'some_token');
    
    ```
    
-   **Request Example:**  "Is user:alice the owner of document:doc1?"
    
-   **Evaluation:**
    
    1.  Look up the  `relation_rules`  for  `namespace_id = 'document_ns'`  and  `relation_name = 'owner'`. Find  `rule_type = 'direct'`.
    2.  Query  `relationship_tuples`  for a matching entry:  `namespace_id = 'document_ns'`,  `object_id = 'doc1'`,  `relation = 'owner'`,  `subject_type = 'user'`,  `subject_id = 'alice'`.
    3.  If a matching tuple is found, the relationship holds (return  `true`).

**Scenario 2: Tuple-to-Userset (Group Membership)**

-   **Zanzibar Scenario:**  A user is a reader of a document because they are a member of a group that is a reader.
    
-   **Zanzibar Example:**
    
    -   Definition:
        
        ```
        definition user {}
        definition group {
          relation member: user
        }
        definition document {
          relation reader: user | group#member
        }
        
        ```
        
    -   Tuples:  `(group:eng, member, user:bob)`,  `(document:doc1, reader, group:eng)`
-   **SQL Inserts:**
    
    SQL
    
    ```
    -- Add namespaces (if needed)
    INSERT INTO namespaces (id, name) VALUES ('group_ns', 'group') ON CONFLICT DO NOTHING;
    
    -- Add relations (if needed)
    INSERT INTO relations (namespace_id, name) VALUES ('group_ns', 'member') ON CONFLICT DO NOTHING;
    INSERT INTO relations (namespace_id, name) VALUES ('document_ns', 'reader') ON CONFLICT DO NOTHING;
    
    -- Define 'member' relation (direct)
    INSERT INTO relation_rules (namespace_id, relation_name, rule_type, expression)
    VALUES ('group_ns', 'member', 'direct', 'member');
    
    -- Define 'reader' relation (tuple-to-userset)
    INSERT INTO relation_rules (namespace_id, relation_name, rule_type, ttu_object_namespace, ttu_relation, expression)
    VALUES ('document_ns', 'reader', 'tuple-to-userset', 'group_ns', 'member', 'reader: user | group#member');
    
    -- Create relationship tuples
    INSERT INTO relationship_tuples (namespace_id, object_id, relation, subject_type, subject_id, zookie_token)
    VALUES ('group_ns', 'eng', 'member', 'user', 'bob', 'token1'); -- Bob is a member of 'eng'
    
    INSERT INTO relationship_tuples (namespace_id, object_id, relation, subject_type, subject_id, userset_namespace, userset_relation, zookie_token)
    VALUES ('document_ns', 'doc1', 'reader', 'group', 'eng', 'group_ns', 'member','token2'); -- 'eng' group is a reader of 'doc1'
    
    ```
    
-   **Request Example:**  "Is user:bob a reader of document:doc1?"
    
-   **Evaluation:**
    
    1.  Look up  `relation_rules`  for  `namespace_id = 'document_ns'`  and  `relation_name = 'reader'`. Find  `rule_type = 'tuple-to-userset'`,  `ttu_object_namespace = 'group_ns'`,  `ttu_relation = 'member'`.
    2.  Check for direct  `reader`  tuples for  `user:bob`  and  `document:doc1`. None found.
    3.  Because it's a  `tuple-to-userset`, look for tuples where:
        -   `relationship_tuples.namespace_id = 'document_ns'`,  `relationship_tuples.object_id = 'doc1'`,  `relationship_tuples.relation = 'reader'`,  `relationship_tuples.subject_type = 'group'`.
        -   This finds the tuple where  `subject_id = 'eng'`.
        -   Now, check if  `user:bob`  is a  `member`  of  `group:eng`: Look for tuples in  `relationship_tuples`  where  `namespace_id = 'group_ns'`,  `object_id = 'eng'`,  `relation = 'member'`,  `subject_type = 'user'`,  `subject_id = 'bob'`.
        -   This tuple exists, so  `user:bob`  is a reader of  `document:doc1`.

**Scenario 3: Union (Permission combining readers and owners)**

-   **Zanzibar Scenario:**  A user can view a document if they are  _either_  a reader  _or_  an owner.
    
-   **Zanzibar Example:**
    
    ```
    definition document {
        relation reader: user;
        relation owner: user;
        permission view = reader + owner;
    }
    
    ```
    
-   **SQL Inserts:**
    
    SQL
    
    ```
     -- Add relations (if needed)
    INSERT INTO relations (namespace_id, name) VALUES ('document_ns', 'view') ON CONFLICT DO NOTHING;
    
    -- Define 'reader' relation (direct)
    INSERT INTO relation_rules (namespace_id, relation_name, rule_type, expression)
    VALUES ('document_ns', 'reader', 'direct', 'reader');
    
    -- Define 'owner' relation (direct)
    INSERT INTO relation_rules (namespace_id, relation_name, rule_type, expression)
    VALUES ('document_ns', 'owner', 'direct', 'owner');
    
    -- Define 'view' permission (union)
    INSERT INTO relation_rules (namespace_id, relation_name, rule_type, child_relations, expression)
    VALUES ('document_ns', 'view', 'union', '["reader", "owner"]', 'view = reader + owner');
    
    -- Create relationship tuples (Carol is a reader, David is an owner)
    INSERT INTO relationship_tuples (namespace_id, object_id, relation, subject_type, subject_id, zookie_token)
    VALUES ('document_ns', 'doc1', 'reader', 'user', 'carol', 'token3');
    INSERT INTO relationship_tuples (namespace_id, object_id, relation, subject_type, subject_id, zookie_token)
    VALUES ('document_ns', 'doc1', 'owner', 'user', 'david', 'token4');
    
    
    ```
    
-   **Request Example:**  "Can user:carol view document:doc1?"
    
-   **Evaluation:**
    
    1.  Look up  `relation_rules`  for  `namespace_id = 'document_ns'`  and  `relation_name = 'view'`. Find  `rule_type = 'union'`,  `child_relations = '["reader", "owner"]'`.
    2.  Check if  `user:carol`  is a  `reader`  of  `document:doc1`  (using the direct relationship check). This finds a matching tuple.
    3.  Since it's a union, and one of the child relations is satisfied, return  `true`.

**Scenario 4: Intersection (Permission requiring both editor and owner)**

-   **Zanzibar Scenario:**  A user can edit a document only if they are  _both_  an editor  _and_  an owner.
    
    ```
    definition document {
        relation editor: user;
        relation owner: user;
        permission edit = editor & owner;
    }
    
    ```
    
-   **SQL Inserts:**
    
    SQL
    
    ```
    -- Add relations (if needed)
    INSERT INTO relations (namespace_id, name) VALUES ('document_ns', 'edit') ON CONFLICT DO NOTHING;
    
    -- Define 'editor' relation (direct)
    INSERT INTO relation_rules (namespace_id, relation_name, rule_type, expression)
    VALUES ('document_ns', 'editor', 'direct', 'editor');
    
    -- Define 'edit' permission (intersection)
    INSERT INTO relation_rules (namespace_id, relation_name, rule_type, child_relations, expression)
    VALUES ('document_ns', 'edit', 'intersection', '["editor", "owner"]', 'edit = editor & owner');
    
    -- Create relationship tuples (Eve is both editor and owner)
    INSERT INTO relationship_tuples (namespace_id, object_id, relation, subject_type, subject_id, zookie_token)
    VALUES ('document_ns', 'doc1', 'editor', 'user', 'eve', 'token5');
    INSERT INTO relationship_tuples (namespace_id, object_id, relation, subject_type, subject_id, zookie_token)
    VALUES ('document_ns', 'doc1', 'owner', 'user', 'eve', 'token6');
    
    ```
    
-   **Request Example:**  "Can user:eve edit document:doc1?"
    
-   **Evaluation:**
    
    1.  Look up  `relation_rules`  for  `namespace_id = 'document_ns'`  and  `relation_name = 'edit'`. Find  `rule_type = 'intersection'`,  `child_relations = '["editor", "owner"]'`.
    2.  Check if  `user:eve`  is an  `editor`  of  `document:doc1`. This finds a matching tuple.
    3.  Check if  `user:eve`  is an  `owner`  of  `document:doc1`. This finds a matching tuple.
    4.  Since it's an intersection, and  _both_  child relations are satisfied, return  `true`.

**Scenario 5: Exclusion (Permission to comment if viewer but not editor)**

-   **Zanzibar Scenario:**  Users can comment if they are viewers but  _not_  editors.
    
    ```
    definition document {
        relation viewer: user;
        relation editor: user;
        permission comment = viewer - editor;
    }
    
    ```
    
-   **SQL Inserts:**
    
    SQL
    
    ```
    -- Add relations (if needed)
    INSERT INTO relations (namespace_id, name) VALUES ('document_ns', 'comment') ON CONFLICT DO NOTHING;
    
    -- Define 'viewer' relation (direct)
    INSERT INTO relation_rules (namespace_id, relation_name, rule_type, expression)
    VALUES ('document_ns', 'viewer', 'direct', 'viewer');
    
     -- Define 'comment' permission (exclusion)
    INSERT INTO relation_rules (namespace_id, relation_name, rule_type, child_relations, expression)
    VALUES ('document_ns', 'comment', 'exclusion', '["viewer", "editor"]', 'comment = viewer - editor');
    
    -- Create relationship tuples (Frank is a viewer, Grace is an editor)
    INSERT INTO relationship_tuples (namespace_id, object_id, relation, subject_type, subject_id, zookie_token)
    VALUES ('document_ns', 'doc1', 'viewer', 'user', 'frank', 'token7');
    INSERT INTO relationship_tuples (namespace_id, object_id, relation, subject_type, subject_id, zookie_token)
    VALUES ('document_ns', 'doc1', 'editor', 'user', 'grace', 'token8');
    
    
    ```
    
-   **Request Example:**  "Can user:frank comment on document:doc1?"
    
-   **Evaluation:**
    
    1.  Look up  `relation_rules`  for  `namespace_id = 'document_ns'`  and  `relation_name = 'comment'`. Find  `rule_type = 'exclusion'`,  `child_relations = '["viewer", "editor"]'`.
    2.  Check if  `user:frank`  is a  `viewer`  of  `document:doc1`. This finds a matching tuple.
    3.  Check if  `user:frank`  is an  `editor`  of  `document:doc1`. No matching tuple is found.
    4.  Since it's an exclusion, and the first relation is satisfied and the second is  _not_, return  `true`.

**Summary and Key Considerations**

This comprehensive set of examples demonstrates how your SQL schema can represent all the core relationship types in Zanzibar:

-   **Direct Relationships:**  Simple entries in  `relationship_tuples`.
-   **Tuple-to-Userset:**  Uses  `relationship_tuples`  in conjunction with  `ttu_object_namespace`  and  `ttu_relation`  in  `relation_rules`  to link to another object's relations.
-   **Union, Intersection, Exclusion:**  These are handled within the  `relation_rules`  table, using the  `child_relations`  field (as a JSONB array of relation names) to define how relations are combined  _within a permission_.

This schema allows you to model complex relationships and permission logic, mirroring the capabilities of Zanzibar. Remember that a real-world implementation would need careful indexing and optimization for performance, especially for the  `tuple_to_userset`  lookups. The  `zookie_token`  is essential for consistency in a distributed system like Zanzibar.
