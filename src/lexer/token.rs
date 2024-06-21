use super::{LiteralKind, Span, Symbol};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CToken<'a> {
    pub kind: CTokenKind,
    pub symbol: Symbol<'a>,
    pub span: Span,
}

impl<'a> CToken<'a> {
    pub fn new(kind: CTokenKind, symbol: Symbol<'a>, span: Span) -> Self {
        Self { kind, symbol, span }
    }

    pub fn eof(span: Span) -> Self {
        Self::new(CTokenKind::Eof, Symbol::None, span)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CTokenKind {
    LineComment,
    BlockComment,
    Whitespace,
    Ident,
    Literal {
        kind: LiteralKind,
        suffix_start: usize,
    },
    Header,
    SemiColon,
    Comma,
    Dot,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBraket,
    RightBracket,
    Pound,
    Colon,
    Equal,
    EqualEqual,
    Bang,
    BangEqual,
    Lt,
    LtEqual,
    GtEqual,
    Gt,
    Minus,
    MinusMinus,
    Plus,
    PlusPlus,
    And,
    Or,
    Star,
    Slash,
    Caret,
    Percent,
    Arrow,
    Tilde,
    Unknown,
    Eof,
}
