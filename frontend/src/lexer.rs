use std::{iter::Enumerate, str::Bytes};

use crate::token::{Token, TokenKind};

pub(crate) struct Lexer<'a> {
    cur: Enumerate<Bytes<'a>>,
    code: &'a str,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(code: &'a str) -> Self {
        Self {
            cur: code.bytes().enumerate(),
            code,
        }
    }

    fn cur_value(&self) -> Option<(usize, u8)> {
        self.cur.clone().next()
    }

    fn cur_value_and_consume(&mut self) -> Option<(usize, u8)> {
        self.cur.next()
    }

    fn consume(&mut self) {
        self.cur.next();
    }

    fn consume_while(&mut self, mut pred: impl FnMut(u8) -> bool) {
        while let Some((_, ch)) = self.cur_value() {
            if !pred(ch) {
                break;
            }
            self.consume();
        }
    }

    fn new_token(&self, kind: TokenKind, start_index: usize, len: usize) -> Token<'a> {
        Token {
            kind,
            spelling: &self.code[start_index..(start_index + len)],
            location: start_index,
        }
    }

    fn handle_integer_literal(&mut self, start_index: usize) -> (TokenKind, usize) {
        self.consume_while(|ch| ch.is_ascii_digit());
        (
            TokenKind::Integer,
            self.cur_value().unwrap_or((self.code.len(), 0)).0 - start_index,
        )
    }

    fn handle_identifier(&mut self, start_index: usize) -> (TokenKind, usize) {
        self.consume_while(|ch| ch.is_ascii_alphanumeric());

        let end_index = self.cur_value().unwrap_or((self.code.len(), 0)).0;
        (
            match &self.code[start_index..end_index] {
                "program" => TokenKind::Program,
                "read" => TokenKind::Read,
                _ => TokenKind::Unknown,
            },
            end_index - start_index,
        )
    }

    pub(crate) fn next_token(&mut self) -> Token<'a> {
        // Consume the whitespaces.
        self.consume_while(|ch| ch.is_ascii_whitespace());

        match self.cur_value_and_consume() {
            None => self.new_token(TokenKind::EOF, self.code.len(), 0),
            Some((index, ch)) => {
                let (kind, len) = match ch {
                    b'(' => (TokenKind::LParen, 1),
                    b')' => (TokenKind::RParen, 1),
                    b'+' => (TokenKind::Plus, 1),
                    b'-' => (TokenKind::Minus, 1),
                    ch if ch.is_ascii_digit() => self.handle_integer_literal(index),
                    ch if ch.is_ascii_alphabetic() => self.handle_identifier(index),
                    _ => (TokenKind::Unknown, 1),
                };
                self.new_token(kind, index, len)
            }
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token() {
            Token {
                kind: TokenKind::EOF,
                ..
            } => None,
            other => Some(other),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn lex() {
        let code = "1 +23program-7";
        let mut lexer = Lexer::new(code);

        assert_eq!(
            lexer.next_token(),
            Token {
                kind: TokenKind::Integer,
                spelling: "1",
                location: 0,
            }
        );

        assert_eq!(
            lexer.next_token(),
            Token {
                kind: TokenKind::Plus,
                spelling: "+",
                location: 2,
            }
        );

        assert_eq!(
            lexer.next_token(),
            Token {
                kind: TokenKind::Integer,
                spelling: "23",
                location: 3,
            }
        );

        assert_eq!(
            lexer.next_token(),
            Token {
                kind: TokenKind::Program,
                spelling: "program",
                location: 5,
            }
        );

        assert_eq!(
            lexer.next_token(),
            Token {
                kind: TokenKind::Minus,
                spelling: "-",
                location: 12,
            }
        );

        assert_eq!(
            lexer.next_token(),
            Token {
                kind: TokenKind::Integer,
                spelling: "7",
                location: 13,
            }
        );

        assert_eq!(
            lexer.next_token(),
            Token {
                kind: TokenKind::EOF,
                spelling: "",
                location: 14,
            }
        );
    }

    #[test]
    fn integers() {
        let code = " 12 3 -3 256";
        let lexer = Lexer::new(code);

        let result_tokens: Vec<_> = lexer.into_iter().collect();

        assert_eq!(
            result_tokens
                .iter()
                .map(|token| token.token_kind())
                .collect::<Vec<_>>(),
            vec![
                TokenKind::Integer,
                TokenKind::Integer,
                TokenKind::Minus,
                TokenKind::Integer,
                TokenKind::Integer,
            ]
        );

        let spellings = vec!["12", "3", "-", "3", "256"];
        assert_eq!(
            result_tokens
                .iter()
                .map(|token| token.spelling())
                .collect::<Vec<_>>(),
            spellings
        );

        let start_locations = vec![1, 4, 6, 7, 9];
        assert_eq!(
            result_tokens
                .iter()
                .map(|token| token.start_location())
                .collect::<Vec<_>>(),
            start_locations
        );

        let lens: Vec<_> = spellings.iter().map(|spelling| spelling.len()).collect();
        assert_eq!(
            result_tokens
                .iter()
                .map(|token| token.len())
                .collect::<Vec<_>>(),
            lens
        );

        let end_locations: Vec<_> = start_locations
            .iter()
            .zip(lens.iter())
            .map(|(loc, len)| loc + len)
            .collect();
        assert_eq!(
            result_tokens
                .iter()
                .map(|token| token.end_location())
                .collect::<Vec<_>>(),
            end_locations
        );
    }

    #[test]
    fn operators() {
        let code = ")(+- ) -*";
        let lexer = Lexer::new(code);

        let result_tokens: Vec<_> = lexer.into_iter().collect();

        assert_eq!(
            result_tokens
                .iter()
                .map(|token| token.token_kind())
                .collect::<Vec<_>>(),
            vec![
                TokenKind::RParen,
                TokenKind::LParen,
                TokenKind::Plus,
                TokenKind::Minus,
                TokenKind::RParen,
                TokenKind::Minus,
                TokenKind::Unknown,
            ]
        );

        let spellings = vec![")", "(", "+", "-", ")", "-", "*"];
        assert_eq!(
            result_tokens
                .iter()
                .map(|token| token.spelling())
                .collect::<Vec<_>>(),
            spellings
        );

        let lens = vec![1; 7];
        assert_eq!(
            result_tokens
                .iter()
                .map(|token| token.len())
                .collect::<Vec<_>>(),
            lens
        );

        let start_locations = vec![0, 1, 2, 3, 5, 7, 8];
        assert_eq!(
            result_tokens
                .iter()
                .map(|token| token.start_location())
                .collect::<Vec<_>>(),
            start_locations
        );

        let end_locations: Vec<_> = start_locations.iter().map(|loc| loc + 1).collect();
        assert_eq!(
            result_tokens
                .iter()
                .map(|token| token.end_location())
                .collect::<Vec<_>>(),
            end_locations
        );
    }

    #[test]
    fn identifers() {
        let code = "program read reAD  Program pRogram xxx";
        let lexer = Lexer::new(code);

        let result_tokens: Vec<_> = lexer.into_iter().collect();

        assert_eq!(
            result_tokens
                .iter()
                .map(|token| token.token_kind())
                .collect::<Vec<_>>(),
            vec![
                TokenKind::Program,
                TokenKind::Read,
                TokenKind::Unknown,
                TokenKind::Unknown,
                TokenKind::Unknown,
                TokenKind::Unknown,
            ]
        );

        let spellings = vec!["program", "read", "reAD", "Program", "pRogram", "xxx"];
        assert_eq!(
            result_tokens
                .iter()
                .map(|token| token.spelling())
                .collect::<Vec<_>>(),
            spellings
        );

        let lens: Vec<_> = spellings.iter().map(|spelling| spelling.len()).collect();
        assert_eq!(
            result_tokens
                .iter()
                .map(|token| token.len())
                .collect::<Vec<_>>(),
            lens
        );

        let start_locations = vec![0, 8, 13, 19, 27, 35];
        assert_eq!(
            result_tokens
                .iter()
                .map(|token| token.start_location())
                .collect::<Vec<_>>(),
            start_locations
        );

        let end_locations: Vec<_> = start_locations
            .iter()
            .zip(lens.iter())
            .map(|(loc, len)| loc + len)
            .collect();
        assert_eq!(
            result_tokens
                .iter()
                .map(|token| token.end_location())
                .collect::<Vec<_>>(),
            end_locations
        );
    }
}
