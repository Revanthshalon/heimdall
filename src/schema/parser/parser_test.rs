#[cfg(test)]
use super::*;

use crate::schema::{
    lexer::Lexer,
    parser::{ParserError, Schema, SchemaParser},
    token::Token,
};

fn tokenize(input: &str) -> Vec<Token> {
    let lexer = Lexer::new(input);
    lexer.tokenize()
}

fn parse_tokens(tokens: &[Token]) -> Result<Schema, Vec<ParserError>> {
    let mut parser = SchemaParser::new(tokens);
    parser.parse()
}

fn parse(input: &str) -> Result<Schema, Vec<ParserError>> {
    let tokens = tokenize(input);
    parse_tokens(&tokens)
}

#[test]
fn test_simple_namespace() {
    let input = r#"
            class User implements Namespace {
                related: {
                    manager: User[];
                }
            }
        "#;

    let result = parse(input).expect("Failed to parse simple namespace");
    assert_eq!(result.namespaces.len(), 1);
    assert_eq!(result.namespaces[0].name.as_ref(), "User");
    assert_eq!(result.namespaces[0].relations.len(), 1);
    assert_eq!(result.namespaces[0].relations[0].name.as_ref(), "manager");
}

#[test]
fn test_multiple_namespaces() {
    let input = r#"
            class User implements Namespace { related: {} }
            class Document implements Namespace { related: {} }
        "#;

    let result = parse(input).expect("Failed to parse multiple namespaces");
    assert_eq!(result.namespaces.len(), 2);
    assert_eq!(result.namespaces[0].name.as_ref(), "User");
    assert_eq!(result.namespaces[1].name.as_ref(), "Document");
}

#[test]
fn test_namespace_error_recovery() {
    let input = r#"
            class User implements Namespace { related: {} }
            class BadClass { /* missing implements */ }
            class Document implements Namespace { related: {} }
        "#;

    let result = parse(input);
    assert!(result.is_err(), "Should have detected an error");
    let schema = result.unwrap_err();
    // Verify the appropriate error was detected
    assert_eq!(schema.len(), 1);
    assert!(schema[0].message.contains("expected"));
}

#[test]
fn test_relation_types() {
    let input = r#"
            class Document implements Namespace {
                related: {
                    // Simple reference
                    owner: User[];
                    // Type union
                    viewers: (User | Team)[];
                    // Subject set
                    members: SubjectSet<Team, "members">[];
                    // Primitive types
                    is_public: boolean;
                    title: string;
                }
            }
        "#;

    let result = parse(input).expect("Failed to parse relation types");
    let relations = &result.namespaces[0].relations;

    // Check the owner relation
    let owner_rel = relations
        .iter()
        .find(|r| r.name.as_ref() == "owner")
        .expect("Owner relation not found");
    assert_eq!(owner_rel.relation_type.len(), 1);
    match &owner_rel.relation_type[0] {
        RelationType::Reference {
            namespace,
            relation,
        } => {
            assert_eq!(namespace.as_ref(), "User");
            assert!(relation.is_none());
        }
        _ => panic!("Expected reference type for owner"),
    }

    // Check the viewers relation (type union)
    let viewers_rel = relations
        .iter()
        .find(|r| r.name.as_ref() == "viewers")
        .expect("Viewers relation not found");
    assert_eq!(
        viewers_rel.relation_type.len(),
        2,
        "Type union should have two types"
    );

    // Check the boolean attribute
    let is_public_rel = relations
        .iter()
        .find(|r| r.name.as_ref() == "is_public")
        .expect("is_public relation not found");
    match &is_public_rel.relation_type[0] {
        RelationType::Attribute(AttributeType::Boolean) => {}
        _ => panic!("Expected boolean attribute type for is_public"),
    }
}

#[test]
fn test_subject_set_relation() {
    let input = r#"
            class Document implements Namespace {
                related: {
                    viewers: SubjectSet<Team, "members">[];
                }
            }
        "#;

    let result = parse(input).expect("Failed to parse subject set relation");
    let relation = &result.namespaces[0].relations[0];

    assert_eq!(relation.name.as_ref(), "viewers");
    match &relation.relation_type[0] {
        RelationType::Reference {
            namespace,
            relation,
        } => {
            assert_eq!(namespace.as_ref(), "Team");
            assert_eq!(relation.as_ref().unwrap().as_ref(), "members");
        }
        _ => panic!("Expected reference with relation for subject set"),
    }
}

#[test]
fn test_simple_permission() {
    let input = r#"
            class Document implements Namespace {
                related: {
                    owner: User[];
                }
                permits: {
                    view: (ctx) => this.related.owner.includes(ctx.subject);
                }
            }
        "#;

    let result = parse(input).expect("Failed to parse simple permission");
    let relation = &result.namespaces[0].relations[1];

    assert_eq!(relation.name.as_ref(), "view");
    assert!(relation.subject_set_rewrite.is_some());

    let rewrite = relation.subject_set_rewrite.as_ref().unwrap();
    assert_eq!(rewrite.operation, Operator::And);
    assert_eq!(rewrite.children.len(), 1);

    match &rewrite.children[0] {
        Child::ComputerSubjectSet { relation } => {
            assert_eq!(relation.as_ref(), "owner");
        }
        _ => panic!("Expected ComputerSubjectSet child for view permission"),
    }
}

#[test]
fn test_logical_operators() {
    let input = r#"
            class Document implements Namespace {
                related: {
                    owner: User[];
                    editors: User[];
                    confidential: boolean;
                }
                permits: {
                    edit: (ctx) => this.related.owner.includes(ctx.subject) || this.related.editors.includes(ctx.subject);
                    share: (ctx) => this.related.owner.includes(ctx.subject) && !this.related.confidential;
                }
            }
        "#;

    let result = parse(input).expect("Failed to parse logical operators");
    let relations = &result.namespaces[0].relations;

    // Find permissions
    let edit = relations
        .iter()
        .find(|r| r.name.as_ref() == "edit")
        .expect("Edit permission not found");
    let share = relations
        .iter()
        .find(|r| r.name.as_ref() == "share")
        .expect("Share permission not found");

    // Check OR operator
    let edit_rewrite = edit.subject_set_rewrite.as_ref().unwrap();
    assert_eq!(edit_rewrite.operation, Operator::Or);
    assert_eq!(edit_rewrite.children.len(), 2);

    // Check AND operator with negation
    let share_rewrite = share.subject_set_rewrite.as_ref().unwrap();
    assert_eq!(share_rewrite.operation, Operator::And);
    assert_eq!(share_rewrite.children.len(), 2);

    // Check for negation
    match &share_rewrite.children[1] {
        Child::InvertResult { .. } => {}
        _ => panic!("Expected InvertResult for negation in share permission"),
    }
}

#[test]
fn test_traverse_expression() {
    let input = r#"
            class Document implements Namespace {
                related: {
                    parent_folder: Folder[];
                }
                permits: {
                    view: (ctx) => this.related.parent_folder.traverse(parent => parent.permits.view(ctx));
                }
            }
        "#;

    let result = parse(input).expect("Failed to parse traverse expression");
    let view = result.namespaces[0]
        .relations
        .iter()
        .find(|r| r.name.as_ref() == "view")
        .expect("View permission not found");

    let rewrite = view.subject_set_rewrite.as_ref().unwrap();
    assert_eq!(rewrite.operation, Operator::And);
    assert_eq!(rewrite.children.len(), 1);

    match &rewrite.children[0] {
        Child::TupleToSubjectSet {
            relation,
            computed_subject_set_relation,
        } => {
            assert_eq!(relation.as_ref(), "parent_folder");
            assert_eq!(computed_subject_set_relation.as_ref(), "view");
        }
        _ => panic!("Expected TupleToSubjectSet for traverse expression"),
    }
}
#[test]
fn test_mismatched_braces() {
    let input = r#"
            class Document implements Namespace {
                related: {
                    owner: User[];
                // Missing closing brace
            }
        "#;

    let result = parse(input);
    assert!(result.is_err());
}

#[test]
fn test_invalid_relation_type() {
    let input = r#"
            class Document implements Namespace {
                related: {
                    owner: InvalidType[];
                }
            }
        "#;

    // This should still parse, as we don't validate type names during parsing
    let result = parse(input).expect("Should parse with unknown type");
    assert_eq!(result.namespaces[0].relations[0].name.as_ref(), "owner");
}

#[test]
fn test_invalid_permission_expression() {
    let input = r#"
            class Document implements Namespace {
                related: { owner: User[]; }
                permits: {
                    view: (ctx) => invalid.syntax.here;
                }
            }
        "#;

    let result = parse(input);
    assert!(
        result.is_err(),
        "Should fail on invalid permission expression"
    );
}

#[test]
fn test_complete_document_namespace() {
    let input = r#"
        class Document implements Namespace {
          related: {
            owner: User[];
            editors: User[];
            viewers: (User | SubjectSet<Team, "members">)[];
            parent_folder: Folder[];
            confidential: boolean;
          }

          permits: {
            edit: (ctx: Context) => 
              this.related.owner.includes(ctx.subject) || 
              this.related.editors.includes(ctx.subject) ||
              this.related.parent_folder.traverse(parent => 
                parent.permits.edit(ctx)
              );

            view: (ctx) => 
              this.permits.edit(ctx) || 
              this.related.viewers.includes(ctx.subject) ||
              this.related.parent_folder.traverse((parent) => 
                parent.permits.view(ctx)
              );

            share: (ctx) => 
              this.related.owner.includes(ctx.subject) && 
              !this.related.confidential;
          }
        }
        "#;

    let result = parse(input).expect("Failed to parse complex Document namespace");
    assert_eq!(result.namespaces.len(), 1);
    assert_eq!(result.namespaces[0].name.as_ref(), "Document");

    // Check relation count (5 related + 3 permits = 8)
    assert_eq!(result.namespaces[0].relations.len(), 8);

    // Verify viewers relation with the union type including SubjectSet
    let viewers = result.namespaces[0]
        .relations
        .iter()
        .find(|r| r.name.as_ref() == "viewers")
        .expect("Viewers relation not found");
    assert_eq!(viewers.relation_type.len(), 2);

    // Verify the share permission with AND and NOT operators
    let share = result.namespaces[0]
        .relations
        .iter()
        .find(|r| r.name.as_ref() == "share")
        .expect("Share permission not found");
    let share_rewrite = share.subject_set_rewrite.as_ref().unwrap();
    assert_eq!(share_rewrite.operation, Operator::And);
}
