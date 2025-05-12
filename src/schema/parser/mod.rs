#![allow(unused)]

use std::sync::Arc;

use cursor::TokenCursor;

use crate::{
    entities::schema::{
        Schema,
        namespace::Namespace,
        relation::{AttributeType, Relation, RelationType},
        subject::{Child, SubjectSetRewrite},
    },
    error::ParserError,
};

use super::token::{Token, kind::TokenKind};

mod cursor;

pub struct SchemaParser<'a> {
    cursor: TokenCursor<'a>,
    errors: Vec<ParserError>,
    fatal: bool,
}

impl<'a> SchemaParser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            cursor: TokenCursor::new(tokens),
            errors: Vec::new(),
            fatal: false,
        }
    }

    pub fn parse(&mut self) -> Result<Schema, Vec<ParserError>> {
        let mut namespaces = Vec::new();

        while !self.cursor.is_at_end() && !self.fatal {
            match self.parse_next() {
                Ok(Some(namespace)) => namespaces.push(namespace),
                Ok(None) => {}
                Err(err) => {
                    if err.fatal {
                        self.fatal = true;
                    }
                    self.errors.push(err);
                }
            }
        }

        if !self.errors.is_empty() {
            return Err(self.errors.clone());
        }

        Ok(Schema {
            namespaces: Arc::new(namespaces),
        })
    }

    fn parse_next(&mut self) -> Result<Option<Namespace>, ParserError> {
        let token = match self.cursor.peek() {
            Some(token) => token,
            None => return Ok(None),
        };

        match token.kind {
            TokenKind::KeywordClass => self.parse_namespace(),
            TokenKind::Error => Err(ParserError::fatal(
                format!("Syntax error {}", token.value),
                Some(token),
            )),
            _ => {
                self.cursor.advance();
                Ok(None)
            }
        }
    }

    fn add_error(&mut self, message: impl Into<String>, token: Option<&Token>, is_fatal: bool) {
        let error = ParserError::new(message, token, is_fatal);
        self.errors.push(error);
        if is_fatal {
            self.fatal = true
        }
    }

    fn expect(&mut self, kind: TokenKind) -> Result<&'a Token, ParserError> {
        if self.cursor.check(&kind) {
            let token = self.cursor.advance();
            match token {
                Some(token) => Ok(token),
                None => Err(ParserError::new("Unexpected EOF", token, true)),
            }
        } else {
            let token = self.cursor.peek();
            Err(ParserError::fatal(
                format!("expected {:?}, found {:?}", kind, token.map(|t| t.kind)),
                token,
            ))
        }
    }

    pub fn parse_namespace(&mut self) -> Result<Option<Namespace>, ParserError> {
        // Consume 'class'
        self.expect(TokenKind::KeywordClass)?;

        // Get namespace name
        let name_token = self.cursor.advance().ok_or(ParserError::fatal(
            "Expected identifier for namespace name",
            None,
        ))?;

        if !name_token.kind.eq(&TokenKind::Identifier) {
            return Err(ParserError::fatal(
                "Expected identifier for namespace name",
                Some(name_token),
            ));
        }

        let namespace_name = name_token.value.clone();

        // Consume 'implements Namespace {'
        self.expect(TokenKind::KeywordImplements)?;
        self.expect(TokenKind::KeywordNamespace)?;
        self.expect(TokenKind::BraceLeft)?;

        let mut relations = Vec::new();

        // Parse namespace body
        while !self.cursor.is_at_end() {
            let token = match self.cursor.peek() {
                Some(token) => token,
                None => break,
            };

            match token.kind {
                TokenKind::BraceRight => {
                    // Consume '}'
                    self.cursor.advance();
                    return Ok(Some(Namespace {
                        name: namespace_name,
                        relation: Arc::new(relations),
                    }));
                }
                TokenKind::KeywordRelated => {
                    let related_relations = self.parse_related()?;
                    relations.extend(related_relations);
                }
                TokenKind::KeywordPermits => {
                    let permit_relations = self.parse_permits()?;
                    relations.extend(permit_relations);
                }
                _ => {
                    return Err(ParserError::fatal(
                        format!(
                            "Expected 'related', 'permits' or '}}', found {:?}",
                            token.kind
                        ),
                        Some(token),
                    ));
                }
            }
        }
        Err(ParserError::fatal(
            "Unexpected end of file while parsing namespace",
            None,
        ))
    }

    /// Sample
    /// related : {
    ///     owner: User[];
    ///     editors: User[];
    ///     viewers: (User|SubjectSet<Team, "members">)[];
    ///     parent_folder: Folder[];
    ///     confidential: boolean;
    /// }
    fn parse_related(&mut self) -> Result<Vec<Relation>, ParserError> {
        // Consume 'related: {'
        self.expect(TokenKind::KeywordRelated)?;
        self.expect(TokenKind::OperatorColon)?;
        self.expect(TokenKind::BraceLeft)?;

        let mut relations = Vec::new();

        while !self.cursor.is_at_end() {
            let token = match self.cursor.peek() {
                Some(token) => token,
                None => break,
            };

            match token.kind {
                // Consume '}'
                TokenKind::BraceRight => {
                    self.cursor.advance();
                    return Ok(relations);
                }
                TokenKind::Identifier | TokenKind::StringLiteral => {
                    let relation_name = token.value.clone();
                    self.cursor.advance();
                    self.expect(TokenKind::OperatorColon)?;

                    let types = self.parse_relation_types()?;

                    relations.push(Relation {
                        name: relation_name,
                        relation_type: Arc::new(types),
                        subject_set_rewrite: None,
                    });
                }
                TokenKind::SemiColon => {
                    self.cursor.advance();
                }
                _ => {
                    return Err(ParserError::fatal(
                        format!("Expected relation name or '}}', found {:?}", token.kind),
                        Some(token),
                    ));
                }
            }
        }

        Ok(relations)
    }

    fn parse_relation_types(&mut self) -> Result<Vec<RelationType>, ParserError> {
        let token = self
            .cursor
            .peek()
            .ok_or(ParserError::fatal("Unexpected end of input", None))?;

        match token.kind {
            TokenKind::Identifier => match token.value.as_ref() {
                "boolean" => {
                    // Consume boolean
                    self.cursor.advance();

                    if self.cursor.check(&TokenKind::SemiColon) {
                        self.cursor.advance();
                    }
                    Ok(vec![RelationType::Attribute(AttributeType::Boolean)])
                }
                "string" => {
                    // Consume string
                    self.cursor.advance();

                    if self.cursor.check(&TokenKind::SemiColon) {
                        self.cursor.advance();
                    }

                    Ok(vec![RelationType::Attribute(AttributeType::String)])
                }
                "SubjectSet" => {
                    // Consume Subject Set
                    self.cursor.advance();
                    let subject_set = self.parse_subject_set()?;

                    self.expect(TokenKind::BracketLeft)?;
                    self.expect(TokenKind::BracketRight)?;

                    if self.cursor.check(&TokenKind::SemiColon) {
                        self.cursor.advance();
                    }

                    Ok(vec![subject_set])
                }
                simple_reference => {
                    let type_ref = RelationType::Reference {
                        namespace: Arc::from(simple_reference),
                        relation: None,
                    };

                    self.cursor.advance();

                    self.expect(TokenKind::BracketLeft)?;
                    self.expect(TokenKind::BracketRight)?;

                    if self.cursor.check(&TokenKind::SemiColon) {
                        self.cursor.advance();
                    }

                    Ok(vec![type_ref])
                }
            },
            // Handles Type Union
            TokenKind::ParenLeft => {
                self.cursor.advance();
                let types = self.parse_type_union()?;

                self.expect(TokenKind::BracketLeft)?;
                self.expect(TokenKind::BracketRight)?;

                if self.cursor.check(&TokenKind::SemiColon) {
                    self.cursor.advance();
                }

                Ok(types)
            }
            others => Err(ParserError::fatal(
                format!("Expected type declaration, found {:?}", others),
                Some(token),
            )),
        }
    }

    fn parse_subject_set(&mut self) -> Result<RelationType, ParserError> {
        self.expect(TokenKind::AngledLeft)?;

        let ns_token = self.cursor.advance().ok_or(ParserError::fatal(
            "expected identifier for subject set namespace",
            None,
        ))?;

        if ns_token.kind.ne(&TokenKind::Identifier) {
            return Err(ParserError::fatal(
                "expected identifer for subject set namespace",
                Some(ns_token),
            ));
        }

        self.expect(TokenKind::OperatorComma)?;

        let relation_token = self.cursor.advance().ok_or(ParserError::fatal(
            "expected identifer for subject set relation",
            None,
        ))?;

        if relation_token.kind.ne(&TokenKind::Identifier)
            && relation_token.kind.ne(&TokenKind::StringLiteral)
        {
            return Err(ParserError::fatal(
                "Expected identifier or string literal for subject set relation",
                Some(relation_token),
            ));
        }

        self.expect(TokenKind::AngledRight)?;

        Ok(RelationType::Reference {
            namespace: ns_token.value.clone(),
            relation: Some(relation_token.value.clone()),
        })
    }

    fn parse_type_union(&mut self) -> Result<Vec<RelationType>, ParserError> {
        let mut types = Vec::new();

        loop {
            let token = self
                .cursor
                .peek()
                .ok_or(ParserError::fatal("Unexpected end of input", None))?;

            match token.kind {
                TokenKind::Identifier => {
                    if token.value.as_ref().eq("SubjectSet") {
                        self.cursor.advance();
                        types.push(self.parse_subject_set()?);
                    } else {
                        let namespace_name = token.value.clone();
                        self.cursor.advance();
                        types.push(RelationType::Reference {
                            namespace: namespace_name,
                            relation: None,
                        });
                    }

                    let next = self
                        .cursor
                        .peek()
                        .ok_or(ParserError::fatal("Unexpected end of token", None))?;

                    match next.kind {
                        TokenKind::AngledRight => {
                            self.cursor.advance();
                            return Ok(types);
                        }
                        TokenKind::TypeUnion => {
                            self.cursor.advance();
                            continue;
                        }
                        _ => {
                            return Err(ParserError::fatal(
                                format!("Expected '|' or '>', found {:?}", next.kind),
                                Some(next),
                            ));
                        }
                    }
                }
                TokenKind::AngledRight => {
                    self.cursor.advance();
                    return Ok(types);
                }
                _ => {
                    return Err(ParserError::fatal(
                        format!("Expected identifer or '>', found {:?}", token.kind),
                        Some(token),
                    ));
                }
            }
        }

        Ok(types)
    }

    fn parse_permits(&mut self) -> Result<Vec<Relation>, ParserError> {
        // Consume 'permits'
        self.expect(TokenKind::KeywordPermits)?;
        self.expect(TokenKind::OperatorColon)?;
        self.expect(TokenKind::BraceLeft)?;

        let mut relations = Vec::new();

        while !self.cursor.is_at_end() {
            let token = match self.cursor.peek() {
                Some(token) => token,
                None => break,
            };

            match token.kind {
                TokenKind::BraceRight => {
                    self.cursor.advance();
                    return Ok(relations);
                }
                TokenKind::Identifier | TokenKind::StringLiteral => {
                    let permission_name = token.value.clone();
                    self.cursor.advance();

                    self.expect(TokenKind::OperatorColon)?;
                    self.expect(TokenKind::ParenLeft)?;
                    self.expect(TokenKind::KeywordCtx)?;

                    if self.cursor.check(&TokenKind::OperatorColon) {
                        self.cursor.advance();
                        self.expect(TokenKind::Identifier)?;
                    }

                    self.expect(TokenKind::ParenRight)?;
                    self.expect(TokenKind::OperatorArrow)?;

                    let rewrite = self.parse_permission_expression()?;

                    let relation = Relation {
                        name: permission_name,
                        relation_type: Arc::new(Vec::new()),
                        subject_set_rewrite: Some(Arc::new(rewrite)),
                    };
                    relations.push(relation);
                }
                TokenKind::SemiColon => {
                    self.cursor.advance();
                }
                _ => {
                    return Err(ParserError::fatal(
                        "expected permission name or '}'",
                        Some(token),
                    ));
                }
            }
        }

        Err(ParserError::fatal(
            "Unexpected end of input while parsing 'permits' block",
            None,
        ))
    }

    fn parse_permission_expression(&mut self) -> Result<SubjectSetRewrite, ParserError> {
        todo!()
    }

    fn parser_simple_permission_expression(&mut self) -> Result<Child, ParserError> {
        todo!()
    }

    fn parse_property_access(&mut self) -> Result<Child, ParserError> {
        todo!()
    }

    fn parse_tuple_to_subject_set(&mut self) -> Result<Child, ParserError> {
        todo!()
    }

    fn parse_computed_subject_set(&mut self) -> Result<Child, ParserError> {
        todo!()
    }
}
