use std::{fmt::Debug, str::Chars};

use super::{Base, CToken, CTokenKind, LiteralKind, ScannerTypes, Span, Symbol, TokenStream};

pub const EOF: char = '\0';

#[derive(Debug)]
pub struct Scanner<'a> {
    pub source: &'a str,
    remaining_len: usize,
    chars: Chars<'a>,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        let chars = source.chars();
        Self {
            source,
            remaining_len: source.len(),
            chars,
            line: 1,
        }
    }

    pub fn iterate(mut self) -> impl Iterator<Item = CToken<'a>> + 'a {
        std::iter::from_fn(move || {
            let tt = self.next_token();
            if tt.kind != CTokenKind::Eof {
                Some(tt)
            } else {
                None
            }
        })
    }

    pub fn test(self) -> TokenStream<'a> {
        let mut stream = TokenStream::default();
        let iter = self.iterate();

        for c in iter {
            stream.push(c);
        }

        stream
    }

    fn first(&self) -> char {
        self.chars.clone().next().unwrap_or(EOF)
    }

    fn as_str(&self) -> &'a str {
        self.chars.as_str()
    }

    fn pos(&self) -> usize {
        self.normalize(self.chars.as_str().len())
    }

    fn reset_pos(&mut self) {
        self.remaining_len = self.chars.as_str().len();
    }

    fn is_eof(&self) -> bool {
        self.as_str().is_empty()
    }

    fn advance(&mut self) -> Option<char> {
        self.chars.next()
    }

    fn advance_while(&mut self, mut predicate: impl FnMut(char) -> bool) {
        while predicate(self.first()) && !self.is_eof() {
            self.advance();
        }
    }

    fn normalize(&self, pos: usize) -> usize {
        self.source.len() - pos
    }

    fn symbolize(&self, tt_ty: CTokenKind, start: usize, end: usize) -> Symbol<'a> {
        let lexeme = &self.source[start..end];

        match tt_ty {
            CTokenKind::Literal { kind, suffix_start } => match kind {
                LiteralKind::Int { base } => {
                    usize::from_str_radix(&lexeme[suffix_start..], base as u32)
                        .unwrap()
                        .into()
                }

                LiteralKind::Float { .. } => lexeme.parse::<f64>().unwrap().into(),

                LiteralKind::Char => todo!(),

                LiteralKind::Str => Symbol::String {
                    sym: &lexeme[suffix_start..lexeme.len() - 1],
                },

                LiteralKind::Byte => todo!(),

                LiteralKind::Literal => lexeme.into(),
            },

            _ => lexeme.into(),
        }
    }

    fn tokenize(&self, kind: CTokenKind) -> CToken<'a> {
        let start = self.normalize(self.remaining_len);
        let end = self.pos();

        let symbol = self.symbolize(kind, start, end);

        CToken::new(kind, symbol, Span::new(start, end))
    }

    fn next_token(&mut self) -> CToken<'a> {
        use CTokenKind::*;
        let first_char = match self.advance() {
            Some(c) => match self.is_whitespace_line(c) {
                true => {
                    self.skip_whitespace();
                    match self.advance() {
                        Some(c) => c,
                        None => return self.tokenize(Eof),
                    }
                }
                false => c,
            },
            None => return CToken::new(Eof, Symbol::None, Span::new(0, 0)),
        };

        let token_kind = match first_char {
            '/' => match self.first() {
                '/' => self.line_comment(),
                '*' => self.block_comment(),
                _ => Slash,
            },

            a if a.is_identifier() => self.identifier(),

            '0' => match self.first() {
                'b' => {
                    self.advance();
                    self.numbers(Base::Binary)
                }
                'x' => {
                    self.advance();
                    self.numbers(Base::Hexadecimal)
                }
                _o @ '0'..='7' => self.numbers(Base::Octal),
                // float??
                // '.' => self.numbers(Base::Decimal),
                _ => self.numbers(Base::Decimal),
            },
            _d @ '0'..='9' => self.numbers(Base::Decimal),

            ';' => SemiColon,
            ',' => Comma,
            '.' => Dot,
            '(' => LeftParen,
            ')' => RightParen,
            '{' => LeftBrace,
            '}' => RightBrace,
            '[' => LeftBraket,
            ']' => RightBracket,
            '#' => Pound,
            '*' => Star,
            '&' => And,
            '|' => Or,
            '^' => Caret,
            '~' => Tilde,

            '"' => self.string(),
            '-' => match self.first() {
                '-' => MinusMinus,
                '>' => Arrow,
                _ => Minus,
            },
            '+' => match self.first() {
                '+' => PlusPlus,
                _ => Plus,
            },
            '<' => match self.first() {
                a if a.is_ascii_alphabetic() => self.header(),
                '=' => LtEqual,
                _ => Lt,
            },
            '>' => match self.first() {
                '=' => GtEqual,
                _ => Gt,
            },
            '=' => match self.first() {
                '=' => EqualEqual,
                _ => Equal,
            },
            '!' => match self.first() {
                '=' => BangEqual,
                _ => Bang,
            },
            _ => CTokenKind::Unknown,
        };

        let res = self.tokenize(token_kind);
        self.reset_pos();
        res
    }

    fn header(&mut self) -> CTokenKind {
        let is_header = |c: char| -> bool { c != '>' };
        self.advance_while(is_header);
        CTokenKind::Header
    }

    fn identifier(&mut self) -> CTokenKind {
        let identifier = |c: char| -> bool { c.is_identifier() };
        self.advance_while(identifier);

        CTokenKind::Ident
    }

    fn numbers(&mut self, base: Base) -> CTokenKind {
        let bin = |c: char| -> bool { matches!(c, '0'..='1') };
        let oct = |c: char| -> bool { matches!(c, '0'..='7') };
        let dec = |c: char| -> bool { c.is_ascii_digit() };
        let hex = |c: char| -> bool { c.is_ascii_hexdigit() };

        let suf = match base {
            Base::Binary => {
                self.advance_while(bin);
                2
            }
            Base::Octal => {
                self.advance_while(oct);
                0
            }
            Base::Decimal => {
                self.advance_while(dec);
                if matches!(self.first(), '.') {
                    self.advance();
                    self.advance_while(dec);
                    return CTokenKind::Literal {
                        kind: LiteralKind::Float { base },
                        suffix_start: 0,
                    };
                }
                0
            }
            Base::Hexadecimal => {
                self.advance_while(hex);
                2
            }
        };

        CTokenKind::Literal {
            kind: LiteralKind::Int { base },
            suffix_start: suf,
        }
    }

    fn line_comment(&mut self) -> CTokenKind {
        let is_line = |c: char| -> bool { c != '\n' };
        self.advance_while(is_line);
        self.line += 1;
        CTokenKind::LineComment
    }

    fn block_comment(&mut self) -> CTokenKind {
        while let Some(c) = self.advance() {
            match c {
                '\n' => self.line += 1,
                '*' if self.first() == '/' => {
                    self.advance();
                    break;
                }
                _ => (),
            }
        }

        CTokenKind::BlockComment
    }

    fn is_whitespace_line(&mut self, c: char) -> bool {
        match c {
            '\n' => {
                self.line += 1;
                true
            }
            '\r' | ' ' | '\t' => true,
            _ => false,
        }
    }

    fn is_whitespace(c: char) -> bool {
        matches!(c, '\n' | '\r' | ' ' | '\t')
    }

    fn skip_whitespace(&mut self) -> CTokenKind {
        self.advance_while(Self::is_whitespace);
        self.reset_pos();
        CTokenKind::Whitespace
    }

    fn string(&mut self) -> CTokenKind {
        let is_string = |c: char| -> bool { c != '"' };
        self.advance_while(is_string);
        self.advance();
        CTokenKind::Literal {
            kind: LiteralKind::Str,
            suffix_start: 1,
        }
    }

    /*
    fn string(&mut self) -> CToken<'a> {

        while self.peek() != Some(&'"') {
            self.advance();
        }
        self.advance();

        self.make_token(CTokenKind::Literal {
            kind: LiteralKind::Str,
            suffix_start: 1,
        })
    }

    } */
}
