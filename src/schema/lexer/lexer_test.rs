#[cfg(test)]
use super::*;

#[test]
fn test_whitespace() {
    let lex = Lexer::new("");
    let input = Span::new("   Hello World");
    let (rest, _) = lex.skip_whitespace(input).unwrap();

    assert_eq!("Hello World", *rest.fragment())
}

#[test]
fn test_keyword() {
    let lex = Lexer::new("");
    let input = Span::new("class");
    let (_, token) = lex.lex_keyword(input).unwrap();

    assert!(token.kind.eq(&TokenKind::KeywordClass));

    let input = Span::new("implements");
    let (_, token) = lex.lex_keyword(input).unwrap();

    assert!(token.kind.eq(&TokenKind::KeywordImplements));

    let input = Span::new("related");
    let (_, token) = lex.lex_keyword(input).unwrap();

    assert!(token.kind.eq(&TokenKind::KeywordRelated));

    let input = Span::new("permits");
    let (_, token) = lex.lex_keyword(input).unwrap();

    assert!(token.kind.eq(&TokenKind::KeywordPermits));

    let input = Span::new("this");
    let (_, token) = lex.lex_keyword(input).unwrap();

    assert!(token.kind.eq(&TokenKind::KeywordThis));

    let input = Span::new("ctx");
    let (_, token) = lex.lex_keyword(input).unwrap();

    assert!(token.kind.eq(&TokenKind::KeywordCtx));

    let input = Span::new("Namespace");
    let (_, token) = lex.lex_keyword(input).unwrap();

    assert!(token.kind.eq(&TokenKind::KeywordNamespace));
}
