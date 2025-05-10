#![allow(unused)]

use cursor::TokenCursor;

use super::token::Token;

mod cursor;

pub struct SchemaParser<'a> {
    cursor: TokenCursor<'a>,
}

impl<'a> SchemaParser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            cursor: TokenCursor::new(tokens),
        }
    }
}
