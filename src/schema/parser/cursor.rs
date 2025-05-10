use std::{cell::Cell, sync::Arc};

use crate::schema::token::{Token, kind::TokenKind};

pub struct TokenCursor<'a> {
    tokens: &'a [Token],
    position: Cell<usize>,
}

impl<'a> TokenCursor<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens,
            position: Cell::new(0),
        }
    }

    pub fn peek(&self) -> Option<&'a Token> {
        self.tokens.get(self.position.get())
    }

    pub fn peek_ahead(&self, n: usize) -> Option<&'a Token> {
        self.tokens.get(self.position.get() + n)
    }

    pub fn advance(&self) -> Option<&'a Token> {
        let token = self.peek();
        self.position.set(self.position.get() + 1);
        token
    }

    pub fn check(&self, kind: &TokenKind) -> bool {
        if let Some(token) = self.peek() {
            std::mem::discriminant(&token.kind) == std::mem::discriminant(kind)
        } else {
            false
        }
    }

    pub fn check_identifier(&self, name: &str) -> bool {
        if let Some(token) = self.peek() {
            if token.kind.eq(&TokenKind::Identifier) {
                token.value.as_ref().eq(name) // Dereferencing Arc<str> to string reference
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn get_identifier_text(&self) -> Option<Arc<str>> {
        if let Some(token) = self.peek() {
            if token.kind.eq(&TokenKind::Identifier) {
                Some(token.value.clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn is_at_end(&self) -> bool {
        self.peek().is_none() || self.check(&TokenKind::Eof)
    }

    pub fn get_string_literal_text(&self) -> Option<Arc<str>> {
        if let Some(token) = self.peek() {
            if token.kind.eq(&TokenKind::StringLiteral) {
                Some(token.value.clone())
            } else {
                None
            }
        } else {
            None
        }
    }
}
