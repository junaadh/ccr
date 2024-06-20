use std::{iter::Peekable, str::CharIndices};

use super::{Base, CToken, CTokenKind, LiteralKind, ScannerTypes, Span, Symbol, TokenStream};

#[derive(Debug)]
pub struct Scanner<'a> {
    pub source: &'a str,
    cursor_pos: usize,
    chars: Peekable<CharIndices<'a>>,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut chars = source.char_indices().peekable();
        Self {
            source,
            cursor_pos: chars
                .peek()
                .map(|(index, _char)| *index)
                .unwrap_or_default(),
            chars,
            line: 1,
        }
    }

    pub fn tokenize(&mut self) -> TokenStream<'a> {
        let mut stream = TokenStream::default();

        loop {
            let tok = self.next_token();
            stream.push(tok);
            match tok.kind {
                CTokenKind::RightBrace => break,
                _ => continue,
            }
        }
        let tok = self.make_eof();

        stream.push(tok);

        stream
    }

    fn advance(&mut self) -> Option<char> {
        self.chars.next().map(|(_index, char)| char)
    }

    fn current_pos(&mut self) -> usize {
        self.chars
            .peek()
            .map(|(index, _char)| *index)
            .unwrap_or_else(|| self.source.len())
    }

    fn symbol(&mut self, end: usize, token_ty: CTokenKind) -> Symbol<'a> {
        let lex = &self.source[self.cursor_pos..end];
        match token_ty {
            // FIXME
            CTokenKind::Literal { kind, suffix_start } => match kind {
                LiteralKind::Int { base } => {
                    usize::from_str_radix(&lex[suffix_start..], base as u32)
                        .unwrap()
                        .into()
                }
                LiteralKind::Char => (lex.as_bytes()[0] as char).into(),
                LiteralKind::Str => lex.into(),
                _ => todo!(),
            },
            _kind => lex.into(),
        }
    }

    fn peek(&mut self) -> Option<&char> {
        self.chars.peek().map(|(_index, char)| char)
    }

    fn make_token(&mut self, kind: CTokenKind) -> CToken<'a> {
        let len = self.current_pos();
        let lexeme = self.symbol(len, kind);

        CToken::new(kind, lexeme, Span::new(self.cursor_pos, len))
    }

    fn make_eof(&mut self) -> CToken<'a> {
        let len = self.current_pos();
        CToken::eof(Span::new(len, len))
    }

    fn next_token(&mut self) -> CToken<'a> {
        self.whitespace();
        self.cursor_pos = self.current_pos();

        let c = self.advance();
        // println!("{c:?}");

        if c.is_identifier() {
            return self.identifiers();
        }

        match c {
            Some(char) => match char {
                '0' => match self.advance() {
                    Some('b') => self.number(Base::Binary),
                    Some('x') => self.number(Base::Hexadecimal),
                    x if x.is_digit(Base::Octal) => self.number(Base::Octal),
                    Some(_) => self.number(Base::Decimal),
                    None => self.make_eof(),
                },
                x if Some(x).is_digit(Base::Decimal) => self.number(Base::Decimal),
                '/' => match self.peek() {
                    Some('/') => self.line_comment(),
                    Some('*') => self.block_comment(),
                    _ => self.make_token(CTokenKind::Slash),
                },
                '{' => self.make_token(CTokenKind::LeftBrace),
                '}' => self.make_token(CTokenKind::RightBrace),
                '(' => self.make_token(CTokenKind::LeftParen),
                ')' => self.make_token(CTokenKind::RightParen),
                '=' => self.make_token(CTokenKind::Equal),
                ';' => self.make_token(CTokenKind::SemiColon),
                _ => todo!(),
            },
            None => self.make_eof(),
        }
    }

    fn number(&mut self, base: Base) -> CToken<'a> {
        while self.chars.peek().map(|(_index, char)| char).is_digit(base) {
            self.advance();
        }

        let suf = match base {
            Base::Binary | Base::Hexadecimal => 2,
            Base::Octal | Base::Decimal => 0,
        };

        self.make_token(CTokenKind::Literal {
            kind: LiteralKind::Int { base },
            // TODO: dynamic??!
            suffix_start: suf,
        })
    }

    fn identifiers(&mut self) -> CToken<'a> {
        while self.chars.peek().map(|(_index, char)| char).is_identifier() {
            self.advance();
        }

        self.make_token(CTokenKind::Ident)
    }

    fn line_comment(&mut self) -> CToken<'a> {
        loop {
            match self.advance() {
                Some('\n') => return self.make_token(CTokenKind::LineComment),
                Some(_a) => continue,
                None => return self.make_token(CTokenKind::Eof),
            }
        }
    }

    fn block_comment(&mut self) -> CToken<'a> {
        loop {
            match self.advance() {
                Some('\n') => {
                    self.line += 1;
                    continue;
                }
                Some('*') => {
                    if let Some(c) = self.advance() {
                        if c == '/' {
                            return self.make_token(CTokenKind::BlockComment);
                        }
                    } else {
                        continue;
                    }
                }
                Some(_) => continue,
                None => return self.make_token(CTokenKind::Eof),
            }
        }
    }

    fn whitespace(&mut self) -> CToken<'a> {
        loop {
            if let Some(c) = self.peek() {
                match c {
                    _whitespace @ (' ' | '\r' | '\t') => {
                        self.advance();
                    }
                    '\n' => {
                        self.line += 1;
                        self.advance();
                    }
                    _ => break,
                }
            }
        }
        self.make_token(CTokenKind::Whitespace)
    }
}
