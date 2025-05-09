mod lexer_test;

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_until, take_while, take_while1},
    character::complete::{line_ending, multispace0, not_line_ending},
    combinator::{eof, recognize, value},
    error::context,
    sequence::{delimited, pair},
};

use crate::error::ParserError;

use super::token::{Span, Token, kind::TokenKind};

pub type LexResult<'a, T> = IResult<Span<'a>, T, ParserError>;

pub struct Lexer<'a> {
    pub source: &'a str,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source }
    }

    pub fn tokenize(&self) -> Vec<Token> {
        let mut tokens = Vec::new();

        let mut rest = Span::new(self.source);

        while !(*rest.fragment()).is_empty() {
            rest = match self.skip_whitespace(rest) {
                Ok((new_rest, _)) => new_rest,
                Err(_) => rest,
            };

            if (*rest.fragment()).is_empty() {
                break;
            }

            match self.lex_token(rest) {
                Ok((new_rest, token)) => {
                    if !token.kind.eq(&TokenKind::Comment) {
                        tokens.push(token);
                    }
                    rest = new_rest;
                }
                Err(e) => match e {
                    nom::Err::Incomplete(_) => {
                        let token = Token::incomplete("Incomplete error");
                        tokens.push(token);
                    }
                    nom::Err::Failure(e) | nom::Err::Error(e) => {
                        let token = e.into();
                        tokens.push(token);
                    }
                },
            }
        }

        tokens
    }

    fn lex_token(&self, input: Span<'a>) -> LexResult<Token> {
        context(
            "lexing token",
            alt((
                |i| self.lex_comment(i),
                |i| self.lex_keyword(i),
                |i| self.lex_identifier(i),
                |i| self.lex_string_literal(i),
                |i| self.lex_operator(i),
                |i| self.lex_bracket(i),
                |i| self.lex_misc(i),
            )),
        )
        .parse(input)
    }

    fn skip_whitespace(&self, input: Span<'a>) -> LexResult<()> {
        context("skipping whitespaces", value((), multispace0)).parse(input)
    }

    fn lex_keyword(&self, input: Span<'a>) -> LexResult<Token> {
        let (rest, span) = context(
            "lexing keyword",
            alt((
                tag("class"),
                tag("implements"),
                tag("related"),
                tag("permits"),
                tag("this"),
                tag("ctx"),
            )),
        )
        .parse(input)?;
        match *span.fragment() {
            "class" => Ok((rest, Token::new(TokenKind::KeywordClass, span))),
            "implements" => Ok((rest, Token::new(TokenKind::KeywordImplements, span))),
            "related" => Ok((rest, Token::new(TokenKind::KeywordRelated, span))),
            "permits" => Ok((rest, Token::new(TokenKind::KeywordPermits, span))),
            "this" => Ok((rest, Token::new(TokenKind::KeywordThis, span))),
            "ctx" => Ok((rest, Token::new(TokenKind::KeywordCtx, span))),
            _ => unreachable!(),
        }
    }

    fn lex_operator(&self, input: Span<'a>) -> LexResult<Token> {
        let (rest, span) = context(
            "lexing operator",
            alt((
                tag("&&"),
                tag("||"),
                tag("!"),
                tag("="),
                tag("=>"),
                tag("."),
                tag(":"),
                tag(","),
            )),
        )
        .parse(input)?;
        match *span.fragment() {
            "&&" => Ok((rest, Token::new(TokenKind::OperatorAnd, span))),
            "||" => Ok((rest, Token::new(TokenKind::OperatorOr, span))),
            "!" => Ok((rest, Token::new(TokenKind::OperatorNot, span))),
            "=" => Ok((rest, Token::new(TokenKind::OperatorAssign, span))),
            "=>" => Ok((rest, Token::new(TokenKind::OperatorAssign, span))),
            "." => Ok((rest, Token::new(TokenKind::OperatorDot, span))),
            ":" => Ok((rest, Token::new(TokenKind::OperatorColon, span))),
            "," => Ok((rest, Token::new(TokenKind::OperatorComma, span))),
            _ => unreachable!(),
        }
    }

    fn lex_misc(&self, input: Span<'a>) -> LexResult<Token> {
        let (rest, span) = context("lexing misc", alt((tag(";"), tag("|")))).parse(input)?;
        match *span.fragment() {
            ";" => Ok((rest, Token::new(TokenKind::SemiColon, span))),
            "|" => Ok((rest, Token::new(TokenKind::TypeUnion, span))),
            _ => unreachable!(),
        }
    }

    fn lex_bracket(&self, input: Span<'a>) -> LexResult<Token> {
        let (rest, span) = context(
            "lexing bracket",
            alt((
                tag("("),
                tag(")"),
                tag("{"),
                tag("}"),
                tag("["),
                tag("]"),
                tag("<"),
                tag(">"),
            )),
        )
        .parse(input)?;
        match *span.fragment() {
            "(" => Ok((rest, Token::new(TokenKind::ParenLeft, span))),
            ")" => Ok((rest, Token::new(TokenKind::ParenRight, span))),
            "{" => Ok((rest, Token::new(TokenKind::BraceLeft, span))),
            "}" => Ok((rest, Token::new(TokenKind::BraceRight, span))),
            "[" => Ok((rest, Token::new(TokenKind::BracketLeft, span))),
            "]" => Ok((rest, Token::new(TokenKind::BracketRight, span))),
            "<" => Ok((rest, Token::new(TokenKind::AngledLeft, span))),
            ">" => Ok((rest, Token::new(TokenKind::AngledRight, span))),
            _ => unreachable!(),
        }
    }

    fn lex_identifier(&self, input: Span<'a>) -> LexResult<Token> {
        let (rest, span) = context(
            "lex identifier",
            recognize(pair(
                take_while1(|c: char| c.is_alphanumeric() || c == '_'),
                take_while(|c: char| c.is_alphanumeric() || c == '_'),
            )),
        )
        .parse(input)?;
        Ok((rest, Token::new(TokenKind::Identifier, span)))
    }

    fn lex_string_literal(&self, input: Span<'a>) -> LexResult<Token> {
        let (rest, span) = context(
            "lex string literal",
            alt((
                delimited(tag("'"), take_until("'"), tag("'")),
                delimited(tag("\""), take_until("\""), tag("\"")),
            )),
        )
        .parse(input)?;
        Ok((rest, Token::new(TokenKind::StringLiteral, span)))
    }

    fn lex_comment(&self, input: Span<'a>) -> LexResult<Token> {
        let (rest, span) = context(
            "lexing comment",
            alt((
                delimited(tag("//"), not_line_ending, alt((eof, line_ending))),
                delimited(tag("/*"), take_until("*/"), tag("*/")),
            )),
        )
        .parse(input)?;
        Ok((rest, Token::new(TokenKind::Comment, span)))
    }
}
