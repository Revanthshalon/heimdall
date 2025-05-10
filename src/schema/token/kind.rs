#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Eof,
    Error,

    Identifier,
    Comment,
    StringLiteral,

    KeywordClass,
    KeywordImplements,
    KeywordRelated,
    KeywordPermits,
    KeywordThis,
    KeywordCtx,
    KeywordNamespace,

    OperatorAnd,
    OperatorOr,
    OperatorNot,
    OperatorAssign,
    OperatorArrow,
    OperatorDot,
    OperatorColon,
    OperatorComma,

    SemiColon,
    TypeUnion,

    ParenLeft,
    ParenRight,
    BraceLeft,
    BraceRight,
    BracketLeft,
    BracketRight,
    AngledLeft,
    AngledRight,
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Eof => write!(f, "End of file"),
            Self::Error => write!(f, "Error"),
            Self::Identifier => write!(f, "Identifier"),
            Self::Comment => write!(f, "Comment"),
            Self::StringLiteral => write!(f, "StringLiteral"),
            Self::KeywordClass => write!(f, "class"),
            Self::KeywordImplements => write!(f, "implements"),
            Self::KeywordNamespace => write!(f, "Namespace"),
            Self::KeywordRelated => write!(f, "related"),
            Self::KeywordPermits => write!(f, "permits"),
            Self::KeywordThis => write!(f, "this"),
            Self::KeywordCtx => write!(f, "ctx"),
            Self::OperatorAnd => write!(f, "&&"),
            Self::OperatorOr => write!(f, "||"),
            Self::OperatorNot => write!(f, "!"),
            Self::OperatorAssign => write!(f, "="),
            Self::OperatorArrow => write!(f, "=>"),
            Self::OperatorDot => write!(f, "."),
            Self::OperatorColon => write!(f, ":"),
            Self::OperatorComma => write!(f, ","),
            Self::SemiColon => write!(f, ";"),
            Self::TypeUnion => write!(f, "|"),
            Self::ParenLeft => write!(f, "("),
            Self::ParenRight => write!(f, ")"),
            Self::BraceLeft => write!(f, "{{"),
            Self::BraceRight => write!(f, "}}"),
            Self::BracketLeft => write!(f, "["),
            Self::BracketRight => write!(f, "]"),
            Self::AngledLeft => write!(f, "<"),
            Self::AngledRight => write!(f, ">"),
        }
    }
}
