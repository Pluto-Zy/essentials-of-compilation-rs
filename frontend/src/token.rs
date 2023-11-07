#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum TokenKind {
    EOF,
    Unknown,
    Integer,
    Identifier,

    LParen,  // (
    RParen,  // )
    Plus,    // +
    Minus,   // -
    LSquare, // [
    RSquare, // ]

    Program, // keyword `program`
    Read,    // keyword `read`
    Let,     // keyword `let`
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub(crate) struct Token<'a> {
    pub(crate) kind: TokenKind,
    pub(crate) spelling: &'a str,
    pub(crate) location: usize,
}

impl<'a> Token<'a> {
    pub(crate) fn token_kind(&self) -> TokenKind {
        self.kind
    }

    pub(crate) fn spelling(&self) -> &'a str {
        self.spelling
    }

    pub(crate) fn start_location(&self) -> usize {
        self.location
    }

    pub(crate) fn end_location(&self) -> usize {
        self.start_location() + self.len()
    }

    pub(crate) fn len(&self) -> usize {
        self.spelling.len()
    }
}
