#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TokenKind {
    EOF,
    Unknown,
    Integer,

    LParen, // (
    RParen, // )
    Plus,   // +
    Minus,  // -

    Program, // keyword `program`
    Read,    // keyword `read`
}

#[derive(PartialEq, Eq, Debug)]
pub struct Token<'a> {
    pub(crate) kind: TokenKind,
    pub(crate) spelling: &'a str,
    pub(crate) location: usize,
}

impl<'a> Token<'a> {
    pub fn token_kind(&self) -> TokenKind {
        self.kind
    }

    pub fn spelling(&self) -> &str {
        self.spelling
    }

    pub fn start_location(&self) -> usize {
        self.location
    }

    pub fn end_location(&self) -> usize {
        self.start_location() + self.len()
    }

    pub fn len(&self) -> usize {
        self.spelling.len()
    }
}
