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
mod parser_test;

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
        // ```ignore
        // class User implements Namespace {...}
        // class Document implements Namespace {...}
        // ```
        let mut namespaces = Vec::new();

        while !self.cursor.is_at_end() {
            if self.check(TokenKind::KeywordClass) {
                match self.parse_namespace() {
                    Ok(namespace) => namespaces.push(namespace),
                    Err(e) => self.errors.push(e),
                }
            } else {
                // Skip tokens that are not a part of namespace definition
                self.cursor.advance();
            }
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
                format!("expected {:?}, found {:?}", kind, self.cursor.peek()),
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
        // ```ignore
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
                    format!(
                        "expected 'related' or 'permits' or '}}', found {:?}",
                        self.cursor.peek()
                    ),
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
    /// ```ignore
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
                // Parse relation definition
                relations.push(self.parse_relation_definitions()?);
            } else if self.check(TokenKind::SemiColon) {
                // Skip semicolons
                self.cursor.advance();
                continue;
            } else {
                return Err(ParserError::new(
                    "expected relation name or '}'",
                    self.cursor.peek(),
                ));
            }
        }

        // Consume closing brace
        self.consume(TokenKind::BraceRight)?;

        Ok(relations)
    }

    fn parse_relation_definitions(&mut self) -> Result<Relation, ParserError> {
        // Parses a single relation or attribute definition
        // Example:
        // ```ignore
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
        // ```ignore
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
                // Simple Relation Type
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
            self.consume(TokenKind::ParenLeft)?;
            return self.parse_type_union();
        }

        Err(ParserError::new(
            format!("expected type definition, found {:?}", self.cursor.peek()),
            self.cursor.peek(),
        ))
    }

    fn parse_subject_set_type(&mut self) -> Result<Vec<RelationType>, ParserError> {
        // Parsers a subject set type reference
        // Example:
        // ```ignore
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
                format!("expected relation name, {:?}", self.cursor.peek()),
                self.cursor.peek(),
            ));
        };

        self.consume(TokenKind::AngledRight)?;

        self.consume(TokenKind::BracketLeft);
        self.consume(TokenKind::BracketRight);

        Ok(vec![RelationType::Reference {
            namespace,
            relation: Some(relation),
        }])
    }

    fn parse_type_union(&mut self) -> Result<Vec<RelationType>, ParserError> {
        // Parses a type union expression
        // Example:
        // ```ignore
        // (User | Team | SubjectSet<Group, "members">)
        // ```
        let mut types = Vec::new();

        if self.check(TokenKind::Identifier) {
            let token = self.consume(TokenKind::Identifier)?;

            match token.value.as_ref() {
                "SubjectSet" => {
                    types.extend(self.parse_subject_set_type()?);
                }
                _ => {
                    let namespace = token.value.clone();
                    types.push(RelationType::Reference {
                        namespace,
                        relation: None,
                    });
                }
            }
        } else {
            return Err(ParserError::new(
                format!("expected type in union, {:?}", self.cursor.peek()),
                self.cursor.peek(),
            ));
        }

        while self.check(TokenKind::TypeUnion) {
            self.cursor.advance();
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
                        let namespace = token.value.clone();
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
        // ```ignore
        // this.related.owner.includes(ctx.subject) || this.related.editors.includes(ctx.subject)
        // ```
        self.consume(TokenKind::OperatorColon)?;
        self.consume(TokenKind::BraceLeft)?;

        let mut relations = Vec::new();

        while !self.cursor.is_at_end() && !self.check(TokenKind::BraceRight) {
            if self.check(TokenKind::Identifier) || self.check(TokenKind::StringLiteral) {
                // Parse permission rule
                relations.push(self.parse_permission_rule()?);
            } else if self.check(TokenKind::SemiColon) {
                // Skip semicolons
                self.cursor.advance();
                continue;
            } else {
                return Err(ParserError::new(
                    format!("expected permission name or '}}', {:?}", self.cursor.peek()),
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
        // ```ignore
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

        // Check for optional semicolon
        if self.check(TokenKind::SemiColon) {
            self.cursor.advance();
        }

        Ok(Relation {
            name,
            relation_type: Arc::new(Vec::new()),
            subject_set_rewrite: Some(Arc::new(expression)),
        })
    }

    fn parse_context_parameter(&mut self) -> Result<(), ParserError> {
        // Parses the context parameter of a permission rule
        // Examples:
        // ```ignore
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
        // Entry point for parsing permission expression
        // Example:
        // ```ignore
        // this.related.owner.includes(ctx.subject) || this.related.editor.includes(ctx.subject)
        // ```
        self.parse_logical_or()
    }

    fn parse_logical_or(&mut self) -> Result<SubjectSetRewrite, ParserError> {
        // Parse OR expression with precedence over AND
        // Example:
        // ```ignore
        // A || B || C
        // ```
        let mut expr = self.parse_logical_and()?;

        while self.check(TokenKind::OperatorOr) {
            self.cursor.advance();
            let right = self.parse_logical_and()?;

            // Create a new OR expression with left and right as children
            let mut children = Vec::new();

            if expr.operation.eq(&Operator::Or) {
                for child in expr.children.iter() {
                    children.push(child.clone());
                }
            } else {
                children.push(Child::Rewrite {
                    rewrite: Arc::new(expr.clone()),
                });
            }

            if right.operation.eq(&Operator::Or) {
                for child in right.children.iter() {
                    children.push(child.clone());
                }
            } else {
                children.push(Child::Rewrite {
                    rewrite: Arc::new(right),
                });
            }

            expr = SubjectSetRewrite {
                operation: Operator::Or,
                children: Arc::new(children),
            };
        }

        Ok(expr)
    }

    fn parse_logical_and(&mut self) -> Result<SubjectSetRewrite, ParserError> {
        let mut expr = self.parse_unary()?;

        while self.check(TokenKind::OperatorAnd) {
            self.cursor.advance();
            let right = self.parse_unary()?;

            let mut children = Vec::new();

            if expr.operation.eq(&Operator::And) {
                for child in expr.children.iter() {
                    children.push(child.clone());
                }
            } else {
                children.push(Child::Rewrite {
                    rewrite: Arc::new(expr.clone()),
                });
            }

            if right.operation.eq(&Operator::And) {
                for child in right.children.iter() {
                    children.push(child.clone());
                }
            } else {
                children.push(Child::Rewrite {
                    rewrite: Arc::new(right.clone()),
                });
            }
            expr = SubjectSetRewrite {
                operation: Operator::And,
                children: Arc::new(children),
            }
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<SubjectSetRewrite, ParserError> {
        // Parses unary expression
        if self.check(TokenKind::OperatorNot) {
            self.cursor.advance();

            let expr = self.parse_primary()?;

            // Create negated expression
            let child_to_negate = Child::Rewrite {
                rewrite: Arc::new(expr.clone()),
            };

            let negated_child = Child::InvertResult {
                child: Arc::new(child_to_negate),
            };

            return Ok(SubjectSetRewrite {
                operation: Operator::And,
                children: Arc::new(vec![negated_child]),
            });
        }

        // Not a negation, parse a primary expression
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<SubjectSetRewrite, ParserError> {
        if self.check(TokenKind::ParenLeft) {
            self.cursor.advance();

            let expr = self.parse_expression()?;
            self.consume(TokenKind::ParenRight)?;
            return Ok(expr);
        }

        self.parse_simple_expression()
    }

    fn parse_simple_expression(&mut self) -> Result<SubjectSetRewrite, ParserError> {
        self.consume(TokenKind::KeywordThis)?;
        self.consume(TokenKind::OperatorDot)?;

        let verb = if self.check(TokenKind::KeywordRelated) {
            self.consume(TokenKind::KeywordRelated)?.value.clone()
        } else if self.check(TokenKind::KeywordPermits) {
            self.consume(TokenKind::KeywordPermits)?.value.clone()
        } else {
            self.consume_identifier()?
        };

        match verb.as_ref() {
            "related" => self.parse_related_expression(),
            "permits" => self.parse_permits_expression(),
            _ => Err(ParserError::new(
                format!("expected 'related' or 'permits', found {}", verb),
                self.cursor.peek(),
            )),
        }
    }

    fn parse_related_expression(&mut self) -> Result<SubjectSetRewrite, ParserError> {
        let relation_name = self.parse_property_name()?;

        if !self.check(TokenKind::OperatorDot) {
            let attribute = Child::AttributeReference {
                relation: relation_name,
            };
            return Ok(SubjectSetRewrite {
                operation: Operator::And,
                children: Arc::new(vec![attribute]),
            });
        }

        self.consume(TokenKind::OperatorDot)?;
        let method = self.consume_identifier()?;

        match method.as_ref() {
            "includes" => self.parse_includes_expression(relation_name),
            "traverse" => self.parse_traverse_expression(relation_name),
            _ => Err(ParserError::new(
                format!("expected 'includes'  or 'traverse' found, {}", method),
                self.cursor.peek(),
            )),
        }
    }

    fn parse_includes_expression(
        &mut self,
        relation: Arc<str>,
    ) -> Result<SubjectSetRewrite, ParserError> {
        self.consume(TokenKind::ParenLeft)?;
        self.consume(TokenKind::KeywordCtx)?;
        self.consume(TokenKind::OperatorDot)?;
        self.consume_identifier()?;
        self.consume(TokenKind::ParenRight)?;

        let computed_set = Child::ComputerSubjectSet { relation };

        Ok(SubjectSetRewrite {
            operation: Operator::And,
            children: Arc::new(vec![computed_set]),
        })
    }

    fn parse_traverse_expression(
        &mut self,
        relation: Arc<str>,
    ) -> Result<SubjectSetRewrite, ParserError> {
        self.consume(TokenKind::ParenLeft)?;

        // Handle optional extra parenthesis
        self.consume(TokenKind::ParenLeft);

        let param_name = self.consume_identifier()?;

        self.consume(TokenKind::ParenRight);

        self.consume(TokenKind::OperatorArrow)?;

        let param_ref = self.consume_identifier()?;
        if param_ref.as_ref().ne(param_name.as_ref()) {
            return Err(ParserError::new(
                format!("expected reference to parameter '{}'", param_name),
                self.cursor.peek(),
            ));
        }

        self.consume(TokenKind::OperatorDot)?;

        let verb = if self.check(TokenKind::KeywordPermits) {
            self.cursor
                .advance()
                .ok_or(ParserError::new("expected permits", self.cursor.peek()))?
                .value
                .clone()
        } else if self.check(TokenKind::KeywordRelated) {
            self.cursor
                .advance()
                .ok_or(ParserError::new("expected related", self.cursor.peek()))?
                .value
                .clone()
        } else if self.check(TokenKind::Identifier) {
            self.consume_identifier()?
        } else {
            return Err(ParserError::new(
                "Expected 'related' or 'permits'",
                self.cursor.peek(),
            ));
        };

        let target_relation = match verb.as_ref() {
            "related" => {
                let rel_name = self.parse_property_name()?;
                self.consume(TokenKind::OperatorDot)?;
                self.consume_identifier()?;

                self.consume(TokenKind::ParenLeft)?;
                self.consume(TokenKind::KeywordCtx)?;
                self.consume(TokenKind::OperatorDot)?;
                self.consume_identifier()?;
                self.consume(TokenKind::ParenRight)?;

                rel_name
            }
            "permits" => {
                let rel_name = self.parse_property_name()?;

                self.consume(TokenKind::ParenLeft)?;
                self.consume(TokenKind::KeywordCtx)?;
                self.consume(TokenKind::ParenRight)?;

                rel_name
            }
            _ => {
                return Err(ParserError::new(
                    format!("expected 'related' or 'permits', found {}", verb),
                    self.cursor.peek(),
                ));
            }
        };

        self.consume(TokenKind::ParenRight)?;

        let tuple_to_set = Child::TupleToSubjectSet {
            relation,
            computed_subject_set_relation: target_relation,
        };

        Ok(SubjectSetRewrite {
            operation: Operator::And,
            children: Arc::new(vec![tuple_to_set]),
        })
    }

    fn parse_permits_expression(&mut self) -> Result<SubjectSetRewrite, ParserError> {
        let relation_name = self.parse_property_name()?;

        self.consume(TokenKind::ParenLeft)?;
        self.consume(TokenKind::KeywordCtx)?;
        self.consume(TokenKind::ParenRight)?;

        let computed_set = Child::ComputerSubjectSet {
            relation: relation_name,
        };

        Ok(SubjectSetRewrite {
            operation: Operator::And,
            children: Arc::new(vec![computed_set]),
        })
    }

    fn parse_property_name(&mut self) -> Result<Arc<str>, ParserError> {
        if self.check(TokenKind::BracketLeft) {
            self.cursor.advance();

            let name_token = self
                .cursor
                .peek()
                .ok_or(ParserError::new("expected property name", None))?;

            if name_token.kind.ne(&TokenKind::Identifier)
                && name_token.kind.ne(&TokenKind::StringLiteral)
            {
                return Err(ParserError::new("expected property name", Some(name_token)));
            }

            let name = name_token.value.clone();
            self.cursor.advance();
            self.consume(TokenKind::BracketRight)?;
            Ok(name)
        } else {
            self.consume(TokenKind::OperatorDot)?;

            let name_token = self
                .cursor
                .peek()
                .ok_or(ParserError::new("expected property name", None))?;

            if name_token.kind.ne(&TokenKind::StringLiteral)
                && name_token.kind.ne(&TokenKind::Identifier)
            {
                return Err(ParserError::new("expected property name", Some(name_token)));
            }

            let name = name_token.value.clone();
            self.cursor.advance();
            Ok(name)
        }
    }
}
