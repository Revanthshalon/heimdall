#[cfg(test)]
use super::*;

#[test]
fn test_whitespace() {
    let lex = Lexer::new("");
    let input = Span::new("   Hello World");
    let (rest, _) = lex.skip_whitespace(input).unwrap();

    assert_eq!("Hello World", *rest.fragment())
}
