#![allow(unused)]

use std::sync::Arc;

use cursor::TokenCursor;

use crate::{
    entities::schema::{Schema, namespace::Namespace, relation::Relation},
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

    fn parse_permits(&mut self) -> Result<Vec<Relation>, ParserError> {
        todo!()
    }
}
