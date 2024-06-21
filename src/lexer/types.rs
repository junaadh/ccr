use super::CToken;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    start: usize,
    end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    // to make clippy happy
    pub fn is_empty(&self) -> bool {
        self.start + self.end == 0
    }

    pub fn len(&self) -> usize {
        self.start + self.end
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiteralKind {
    Int { base: Base },
    Float { base: Base },
    Char,
    Str,
    Byte,
    Literal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Base {
    Binary = 2,
    Octal = 8,
    Decimal = 10,
    Hexadecimal = 16,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Symbol<'a> {
    Int { sym: usize },
    Float { sym: f64 },
    Char { sym: char },
    String { sym: &'a str },
    Byte { sym: u8 },
    Literal { sym: &'a str },
    None,
}

impl<'a> From<&'a str> for Symbol<'a> {
    fn from(value: &'a str) -> Self {
        Self::Literal { sym: value }
    }
}

impl<'a> From<u8> for Symbol<'a> {
    fn from(value: u8) -> Self {
        Self::Byte { sym: value }
    }
}

impl<'a> From<usize> for Symbol<'a> {
    fn from(value: usize) -> Self {
        Self::Int { sym: value }
    }
}

impl<'a> From<f64> for Symbol<'a> {
    fn from(value: f64) -> Self {
        Self::Float { sym: value }
    }
}

impl<'a> From<char> for Symbol<'a> {
    fn from(value: char) -> Self {
        Self::Char { sym: value }
    }
}

pub trait ScannerTypes {
    fn is_identifier(&self) -> bool;
    fn is_cdigit(&self, base: Base) -> bool;
    // fn is_whitespace(&self, line: &mut usize) -> bool;
}

impl ScannerTypes for char {
    fn is_identifier(&self) -> bool {
        self.is_ascii_alphabetic() || *self == '_'
    }

    fn is_cdigit(&self, base: Base) -> bool {
        self.is_digit(base as u32)
    }
}

#[derive(Debug, Default)]
pub struct TokenStream<'a>(Vec<CToken<'a>>);

impl<'a> TokenStream<'a> {
    pub fn push(&mut self, token: CToken<'a>) {
        self.0.push(token);
    }

    pub fn pop(&mut self) -> Option<CToken<'a>> {
        self.0.pop()
    }

    pub fn iter(&self) -> impl Iterator<Item = &CToken<'a>> {
        self.0.iter()
    }
}
