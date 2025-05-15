#[cfg(test)]
use super::*;

#[test]
fn test_operator() {
    let lex = Lexer::new("");
    
    // Test AND operator
    let input = Span::new("&&");
    let (_, token) = lex.lex_operator(input).unwrap();
    assert!(token.kind.eq(&TokenKind::OperatorAnd));
    
    // Test OR operator
    let input = Span::new("||");
    let (_, token) = lex.lex_operator(input).unwrap();
    assert!(token.kind.eq(&TokenKind::OperatorOr));
    
    // Test NOT operator
    let input = Span::new("!");
    let (_, token) = lex.lex_operator(input).unwrap();
    assert!(token.kind.eq(&TokenKind::OperatorNot));
    
    // Test ASSIGN operator
    let input = Span::new("=");
    let (_, token) = lex.lex_operator(input).unwrap();
    assert!(token.kind.eq(&TokenKind::OperatorAssign));
    
    // Test ARROW operator
    let input = Span::new("=>");
    let (_, token) = lex.lex_operator(input).unwrap();
    assert!(token.kind.eq(&TokenKind::OperatorAssign));
    
    // Test DOT operator
    let input = Span::new(".");
    let (_, token) = lex.lex_operator(input).unwrap();
    assert!(token.kind.eq(&TokenKind::OperatorDot));
    
    // Test COLON operator
    let input = Span::new(":");
    let (_, token) = lex.lex_operator(input).unwrap();
    assert!(token.kind.eq(&TokenKind::OperatorColon));
    
    // Test COMMA operator
    let input = Span::new(",");
    let (_, token) = lex.lex_operator(input).unwrap();
    assert!(token.kind.eq(&TokenKind::OperatorComma));
}

#[test]
fn test_misc() {
    let lex = Lexer::new("");
    
    // Test semicolon
    let input = Span::new(";");
    let (_, token) = lex.lex_misc(input).unwrap();
    assert!(token.kind.eq(&TokenKind::SemiColon));
    
    // Test pipe (type union)
    let input = Span::new("|");
    let (_, token) = lex.lex_misc(input).unwrap();
    assert!(token.kind.eq(&TokenKind::TypeUnion));
}

#[test]
fn test_bracket() {
    let lex = Lexer::new("");
    
    // Test all bracket types
    let brackets = [
        ("(", TokenKind::ParenLeft),
        (")", TokenKind::ParenRight),
        ("{", TokenKind::BraceLeft),
        ("}", TokenKind::BraceRight),
        ("[", TokenKind::BracketLeft),
        ("]", TokenKind::BracketRight),
        ("<", TokenKind::AngledLeft),
        (">", TokenKind::AngledRight),
    ];
    
    for (bracket, expected_kind) in brackets {
        let input = Span::new(bracket);
        let (_, token) = lex.lex_bracket(input).unwrap();
        assert!(token.kind.eq(&expected_kind));
    }
}

#[test]
fn test_identifier() {
    let lex = Lexer::new("");
    
    // Test basic identifier
    let input = Span::new("foo");
    let (_, token) = lex.lex_identifier(input).unwrap();
    assert!(token.kind.eq(&TokenKind::Identifier));
    
    // Test identifier with underscore
    let input = Span::new("foo_bar");
    let (_, token) = lex.lex_identifier(input).unwrap();
    assert!(token.kind.eq(&TokenKind::Identifier));
    
    // Test identifier with numbers
    let input = Span::new("foo123");
    let (_, token) = lex.lex_identifier(input).unwrap();
    assert!(token.kind.eq(&TokenKind::Identifier));
}

#[test]
fn test_string_literal() {
    let lex = Lexer::new("");
    
    // Test single quoted string
    let input = Span::new("'hello'");
    let (_, token) = lex.lex_string_literal(input).unwrap();
    assert!(token.kind.eq(&TokenKind::StringLiteral));
    
    // Test double quoted string
    let input = Span::new("\"world\"");
    let (_, token) = lex.lex_string_literal(input).unwrap();
    assert!(token.kind.eq(&TokenKind::StringLiteral));
}

#[test]
fn test_comment() {
    let lex = Lexer::new("");
    
    // Test block comment
    let input = Span::new("/* this is a comment */");
    let (_, token) = lex.lex_comment(input).unwrap();
    assert!(token.kind.eq(&TokenKind::Comment));
}

#[test]
fn test_lex_token() {
    let lex = Lexer::new("");
    
    // Test comment detection
    let input = Span::new("/* comment */");
    let (_, token) = lex.lex_token(input).unwrap();
    assert!(token.kind.eq(&TokenKind::Comment));
    
    // Test keyword detection
    let input = Span::new("class");
    let (_, token) = lex.lex_token(input).unwrap();
    assert!(token.kind.eq(&TokenKind::KeywordClass));
    
    // Test identifier detection
    let input = Span::new("identifier123");
    let (_, token) = lex.lex_token(input).unwrap();
    assert!(token.kind.eq(&TokenKind::Identifier));
    
    // Test string literal detection
    let input = Span::new("\"string\"");
    let (_, token) = lex.lex_token(input).unwrap();
    assert!(token.kind.eq(&TokenKind::StringLiteral));
    
    // Test operator detection
    let input = Span::new("&&");
    let (_, token) = lex.lex_token(input).unwrap();
    assert!(token.kind.eq(&TokenKind::OperatorAnd));
    
    // Test bracket detection
    let input = Span::new("{");
    let (_, token) = lex.lex_token(input).unwrap();
    assert!(token.kind.eq(&TokenKind::BraceLeft));
    
    // Test misc token detection
    let input = Span::new(";");
    let (_, token) = lex.lex_token(input).unwrap();
    assert!(token.kind.eq(&TokenKind::SemiColon));
}

#[test]
fn test_tokenize() {
    // Test empty input
    let lex = Lexer::new("");
    let tokens = lex.tokenize();
    assert!(tokens.is_empty());
    
    // Test simple class declaration
    let source = "class Foo {}";
    let lex = Lexer::new(source);
    let tokens = lex.tokenize();
    
    assert_eq!(tokens.len(), 4);
    assert!(tokens[0].kind.eq(&TokenKind::KeywordClass));
    assert!(tokens[1].kind.eq(&TokenKind::Identifier));
    assert!(tokens[2].kind.eq(&TokenKind::BraceLeft));
    assert!(tokens[3].kind.eq(&TokenKind::BraceRight));
    
    // Test with comments (should be filtered out)
    let source = "class Foo { /* comment */ }";
    let lex = Lexer::new(source);
    let tokens = lex.tokenize();
    
    assert_eq!(tokens.len(), 4); // Comment should be filtered out
    assert!(tokens[0].kind.eq(&TokenKind::KeywordClass));
    assert!(tokens[1].kind.eq(&TokenKind::Identifier));
    assert!(tokens[2].kind.eq(&TokenKind::BraceLeft));
    assert!(tokens[3].kind.eq(&TokenKind::BraceRight));
    
    // Test more complex example
    let source = "class Foo implements Bar { this.x = 'hello'; }";
    let lex = Lexer::new(source);
    let tokens = lex.tokenize();
    
    assert_eq!(tokens.len(), 12);
    assert!(tokens[0].kind.eq(&TokenKind::KeywordClass));
    assert!(tokens[1].kind.eq(&TokenKind::Identifier)); // Foo
    assert!(tokens[2].kind.eq(&TokenKind::KeywordImplements));
    assert!(tokens[3].kind.eq(&TokenKind::Identifier)); // Bar
    assert!(tokens[4].kind.eq(&TokenKind::BraceLeft));
    assert!(tokens[5].kind.eq(&TokenKind::KeywordThis));
    assert!(tokens[6].kind.eq(&TokenKind::OperatorDot));
    assert!(tokens[7].kind.eq(&TokenKind::Identifier)); // x
    assert!(tokens[8].kind.eq(&TokenKind::OperatorAssign));
    assert!(tokens[9].kind.eq(&TokenKind::StringLiteral)); // 'hello'
    assert!(tokens[10].kind.eq(&TokenKind::SemiColon));
    assert!(tokens[11].kind.eq(&TokenKind::BraceRight));
}