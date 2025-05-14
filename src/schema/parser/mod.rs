#![allow(unused)]

use std::sync::Arc;

use cursor::TokenCursor;

use crate::{
    entities::schema::{
        Schema,
        namespace::Namespace,
        relation::{AttributeType, Relation, RelationType},
        subject::{Child, Operator, SubjectSetRewrite},
    },
    error::ParserError,
};

use super::token::{Token, kind::TokenKind};

mod cursor;

pub const TUPLE_TO_SUBJECT_SET_TYPE_CHECK_MAX_DEPTH: usize = 10;
pub const EXPRESSION_NESTING_MAX_DEPTH: usize = 10;

pub struct SchemaParser<'a> {
    cursor: TokenCursor<'a>,
    errors: Vec<ParserError>,
}

impl<'a> SchemaParser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            cursor: TokenCursor::new(tokens),
            errors: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> Result<Schema, Vec<ParserError>> {
        // Entry point for parsing an entire schema file
        // Example:
        // ```
        // class User implements Namespace {...}
        // class Document implements Namespace {...}
        // ```
        let mut namespaces = Vec::new();

        while !self.cursor.is_at_end() {
            todo!()
        }

        if !self.errors.is_empty() {
            return Err(self.errors.clone());
        }

        Ok(Schema {
            namespaces: Arc::new(namespaces),
        })
    }

    //---------------------------
    // Helper Methods
    //---------------------------

    /// Method takes a tokenkind as an argument and checks if the token matches the argument
    fn check(&self, kind: TokenKind) -> bool {
        self.cursor.check(&kind)
    }

    /// Method takes a reference to tokenkind as argument and checks if the token matches. If it
    /// matches then it consumes the token else returns an error
    fn consume(&mut self, kind: TokenKind) -> Result<&Token, ParserError> {
        if self.check(kind) {
            Ok(self
                .cursor
                .advance()
                .ok_or(ParserError::new("unexpected end of file", None))?)
        } else {
            Err(ParserError::new(
                format!("expected {:?}", kind),
                self.cursor.peek(),
            ))
        }
    }

    fn consume_identifier(&mut self) -> Result<Arc<str>, ParserError> {
        let token = self
            .cursor
            .peek()
            .ok_or(ParserError::new("unexpected end of file", None))?;

        if token.kind.ne(&TokenKind::Identifier) {
            return Err(ParserError::new(
                format!("expected identifier, found {:?}", token.kind),
                Some(token),
            ));
        }

        let identifier = token.value.clone();
        self.cursor.advance();
        Ok(identifier)
    }

    //---------------------------
    // Namespace Parsing
    //---------------------------

    fn parse_namespace(&mut self) -> Result<Namespace, ParserError> {
        // Parses a single namespace definition
        // Example:
        // ```
        // class User implements Namespace {
        //   related: { ... }
        //   permits: { ... }
        // }
        // ```
        self.consume(TokenKind::KeywordClass)?;

        let name = self.consume_identifier()?;

        self.consume(TokenKind::KeywordImplements)?;
        self.consume(TokenKind::KeywordNamespace)?;
        self.consume(TokenKind::BraceLeft)?;

        let mut relations = Vec::new();

        while !self.cursor.is_at_end() && !self.check(TokenKind::BraceRight) {
            if self.check(TokenKind::KeywordRelated) {
                self.consume(TokenKind::KeywordRelated)?;
                let related_relations = self.parse_related_block()?;
                relations.extend(related_relations);
            } else if self.check(TokenKind::KeywordPermits) {
                self.consume(TokenKind::KeywordPermits)?;
                let permits_relations = self.parse_permits_block()?;
                relations.extend(permits_relations);
            } else {
                return Err(ParserError::new(
                    "expected 'related' or 'permits' or '}'",
                    self.cursor.peek(),
                ));
            }
        }

        self.consume(TokenKind::BraceRight)?;

        Ok(Namespace::new(name, relations))
    }

    //---------------------------
    // Related block parsing
    //---------------------------

    /// Parses a related block which defines relations and attributes
    /// ```
    /// related: {
    ///     owner: User[];
    ///     editors: (User | Team)[];
    ///     confidential: boolean;
    /// }
    /// ```
    fn parse_related_block(&mut self) -> Result<Vec<Relation>, ParserError> {
        self.consume(TokenKind::OperatorColon)?;
        self.consume(TokenKind::BraceLeft)?;

        let mut relations = Vec::new();

        while !self.cursor.is_at_end() && !self.check(TokenKind::BraceRight) {
            if self.check(TokenKind::Identifier) || self.check(TokenKind::StringLiteral) {
                relations.push(self.parse_relation_definitions()?);
            } else if self.check(TokenKind::SemiColon) {
                continue;
            } else {
                return Err(ParserError::new(
                    "expected relation name",
                    self.cursor.peek(),
                ));
            }
        }

        self.consume(TokenKind::BraceRight)?;

        Ok(relations)
    }

    fn parse_relation_definitions(&mut self) -> Result<Relation, ParserError> {
        // Parses a single relation or attribute definition
        // Example:
        // ```
        // owner: User[];
        // confidential: boolean;
        // ```
        let name = if self.check(TokenKind::StringLiteral) {
            let token = self
                .cursor
                .advance()
                .ok_or(ParserError::new("unexpected end of file", None))?;
            token.value.clone()
        } else {
            self.consume_identifier()?
        };

        self.consume(TokenKind::OperatorColon)?;

        let relation_types = self.parse_relation_type()?;

        self.consume(TokenKind::SemiColon);

        Ok(Relation {
            name,
            relation_type: Arc::new(relation_types),
            subject_set_rewrite: None,
        })
    }

    fn parse_relation_type(&mut self) -> Result<Vec<RelationType>, ParserError> {
        // Parses the type definition for a relation
        // Example:
        // ```
        // User[]                           -> Namespace Reference
        // boolean                          -> Primitive Attribute
        // (User | Team) []                 -> Type union
        // SubjectSet<Team, "members">[]    -> Subject set
        // ```
        if self.check(TokenKind::Identifier) {
            let token = self
                .cursor
                .peek()
                .ok_or(ParserError::new("unexpected end of file", None))?;

            match token.value.as_ref() {
                "boolean" => {
                    self.cursor.advance();
                    return Ok(vec![RelationType::Attribute(AttributeType::Boolean)]);
                }
                "string" => {
                    self.cursor.advance();
                    return Ok(vec![RelationType::Attribute(AttributeType::String)]);
                }
                "SubjectSet" => {
                    self.cursor.advance();
                    return self.parse_subject_set_type();
                }
                _ => {
                    // Regular namespace reference
                    let namespace = token.value.clone();
                    self.cursor.advance();

                    // Check for array notation
                    self.consume(TokenKind::BracketLeft)?;
                    self.consume(TokenKind::BracketRight)?;

                    return Ok(vec![RelationType::Reference {
                        namespace,
                        relation: None,
                    }]);
                }
            }
        } else if self.check(TokenKind::ParenLeft) {
            // Union type like (User | Team)[]
            return self.parse_type_union();
        }

        Err(ParserError::new(
            "expected type definition",
            self.cursor.peek(),
        ))
    }

    fn parse_subject_set_type(&mut self) -> Result<Vec<RelationType>, ParserError> {
        // Parsers a subject set type reference
        // Example:
        // ```
        // SubjectSet<Team, "members">[]
        // ```
        self.consume(TokenKind::AngledLeft)?;

        let namespace = self.consume_identifier()?;

        self.consume(TokenKind::OperatorComma)?;

        let relation = if self.check(TokenKind::Identifier) {
            self.consume_identifier()?
        } else if self.check(TokenKind::StringLiteral) {
            let token = self
                .cursor
                .advance()
                .ok_or(ParserError::new("unexpected end of file", None))?;
            token.value.clone()
        } else {
            return Err(ParserError::new(
                "expected relation name",
                self.cursor.peek(),
            ));
        };

        self.consume(TokenKind::AngledRight)?;

        self.consume(TokenKind::BracketLeft)?;
        self.consume(TokenKind::BracketRight)?;

        Ok(vec![RelationType::Reference {
            namespace,
            relation: Some(relation),
        }])
    }

    fn parse_type_union(&mut self) -> Result<Vec<RelationType>, ParserError> {
        // Parses a type union expression
        // Example:
        // ```
        // (User | Team | SubjectSet<Group, "members">)
        // ```
        let mut types = Vec::new();

        if self.check(TokenKind::Identifier) {
            let token = self
                .cursor
                .advance()
                .ok_or(ParserError::new("unexpected end of file", None))?;
            match token.value.as_ref() {
                "SubjectSet" => {
                    types.extend(self.parse_subject_set_type()?);
                }
                _ => {
                    let namespace = self.consume_identifier()?;
                    types.push(RelationType::Reference {
                        namespace,
                        relation: None,
                    });
                }
            }
        } else {
            return Err(ParserError::new(
                "expected type in union",
                self.cursor.peek(),
            ));
        }

        while self.check(TokenKind::TypeUnion) {
            if self.check(TokenKind::Identifier) {
                let token = self
                    .cursor
                    .advance()
                    .ok_or(ParserError::new("unexpected end of file", None))?;
                match token.value.as_ref() {
                    "SubjectSet" => {
                        types.extend(self.parse_subject_set_type()?);
                    }
                    _ => {
                        let namespace = self.consume_identifier()?;
                        types.push(RelationType::Reference {
                            namespace,
                            relation: None,
                        });
                    }
                }
            } else {
                return Err(ParserError::new(
                    "expected type in union",
                    self.cursor.peek(),
                ));
            }
        }

        self.consume(TokenKind::ParenRight)?;

        self.consume(TokenKind::BracketLeft)?;
        self.consume(TokenKind::BracketRight)?;

        Ok(types)
    }

    //---------------------------
    // Permits block parsing
    //---------------------------

    fn parse_permits_block(&mut self) -> Result<Vec<Relation>, ParserError> {
        // Entry point for parsing permission expressions
        // Example:
        // ```
        // this.related.owner.includes(ctx.subject) || this.related.editors.includes(ctx.subject)
        // ```
        self.consume(TokenKind::OperatorColon)?;
        self.consume(TokenKind::BraceLeft)?;

        let mut relations = Vec::new();

        while !self.cursor.is_at_end() || !self.check(TokenKind::BraceRight) {
            if self.check(TokenKind::Identifier) || self.check(TokenKind::StringLiteral) {
                // Parse permission rule
                relations.push(self.parse_permission_rule()?);
            } else if self.check(TokenKind::SemiColon) {
                // Skip semicolons
                continue;
            } else {
                return Err(ParserError::new(
                    "expected permission name or '}'",
                    self.cursor.peek(),
                ));
            }
        }

        // Consume closing brace
        self.consume(TokenKind::BraceRight)?;

        Ok(relations)
    }

    fn parse_permission_rule(&mut self) -> Result<Relation, ParserError> {
        // Parses a single permission rule
        // Example:
        // ```
        // edit: (ctx) => this.related.owner.includes(ctx.subject) || this.related.editors.includes(ctx.subject);
        // ```
        let name = if self.check(TokenKind::StringLiteral) {
            let token = self
                .cursor
                .advance()
                .ok_or(ParserError::new("unexpected end of file", None))?;
            token.value.clone()
        } else {
            self.consume_identifier()?
        };

        self.consume(TokenKind::OperatorColon)?;
        self.parse_context_parameter()?;
        self.consume(TokenKind::OperatorArrow)?;

        let expression = self.parse_expression()?;

        self.consume(TokenKind::SemiColon);

        Ok(Relation {
            name,
            relation_type: Arc::new(Vec::new()),
            subject_set_rewrite: Some(Arc::new(expression)),
        })
    }

    fn parse_context_parameter(&mut self) -> Result<(), ParserError> {
        // Parses the context parameter of a permission rule
        // Examples:
        // ```
        // (ctx)
        // (ctx: Context)
        // ```
        self.consume(TokenKind::ParenLeft)?;
        self.consume(TokenKind::KeywordCtx)?;

        // Optional type annotation
        if self.check(TokenKind::OperatorColon) {
            self.cursor.advance();
            self.consume_identifier()?; // Skip type name (Context)
        }

        self.consume(TokenKind::ParenRight)?;

        // Optional return type
        if self.check(TokenKind::OperatorColon) {
            self.cursor.advance();
            self.consume_identifier()?; // Skip return type
        }

        Ok(())
    }

    //---------------------------
    // Permission parsing
    //---------------------------

    fn parse_expression(&mut self) -> Result<SubjectSetRewrite, ParserError> {
        todo!()
    }

    fn parse_logical_or(&mut self) -> Result<SubjectSetRewrite, ParserError> {
        todo!()
    }

    fn parse_logical_and(&mut self) -> Result<SubjectSetRewrite, ParserError> {
        todo!()
    }

    fn parse_unary(&mut self) -> Result<SubjectSetRewrite, ParserError> {
        todo!()
    }

    fn parse_primary(&mut self) -> Result<SubjectSetRewrite, ParserError> {
        todo!()
    }

    fn parse_simple_expression(&mut self) -> Result<SubjectSetRewrite, ParserError> {
        todo!()
    }

    fn parse_related_expression(&mut self) -> Result<SubjectSetRewrite, ParserError> {
        todo!()
    }

    fn parse_includes_expression(&mut self) -> Result<SubjectSetRewrite, ParserError> {
        todo!()
    }

    fn parse_traverse_expression(&mut self) -> Result<SubjectSetRewrite, ParserError> {
        todo!()
    }

    fn parse_permits_expression(&mut self) -> Result<SubjectSetRewrite, ParserError> {
        todo!()
    }

    fn parse_property_name(&mut self) -> Result<SubjectSetRewrite, ParserError> {
        todo!()
    }
}
