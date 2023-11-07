use std::num::ParseIntError;

use crate::{
    ast::{BinaryOpKind, Expr, UnaryOpKind},
    lexer::Lexer,
    token::{Token, TokenKind},
};

#[derive(Debug, Eq, PartialEq)]
pub enum ParseErrorKind {
    ParseIntegerError(ParseIntError),
    InvalidOperandCount(usize),
    MismatchedOpenParen,
    UnexpectedToken(String),
}

#[derive(Debug, Eq, PartialEq)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub location: usize,
}

struct Parser<'a> {
    lexer: Lexer<'a>,
    cur_token: Token<'a>,
}

impl<'a> Parser<'a> {
    fn new(code: &'a str) -> Self {
        let mut lexer = Lexer::new(code);
        let cur_token = lexer.next_token();
        Parser { lexer, cur_token }
    }

    fn consume_token(&mut self) {
        self.cur_token = self.lexer.next_token();
    }

    fn current_token_and_consume(&mut self) -> Token<'a> {
        let result = self.cur_token.clone();
        self.consume_token();
        result
    }

    fn expect_and_consume(&mut self, kind: TokenKind) -> Result<Token<'a>, ParseError> {
        if self.cur_token.token_kind() == kind {
            Ok(self.current_token_and_consume())
        } else {
            Err(ParseError {
                kind: ParseErrorKind::UnexpectedToken(self.cur_token.spelling().to_string()),
                location: self.cur_token.start_location(),
            })
        }
    }

    fn expect_closing_paren_and_consume(
        &mut self,
        kind: TokenKind,
        open_token: &Token<'a>,
    ) -> Result<Token<'a>, ParseError> {
        if self.cur_token.token_kind() == kind {
            Ok(self.current_token_and_consume())
        } else {
            Err(ParseError {
                kind: ParseErrorKind::MismatchedOpenParen,
                location: open_token.start_location(),
            })
        }
    }

    fn parse_integer(&mut self) -> Result<u64, ParseError> {
        // eat the integer token
        let token = self.current_token_and_consume();

        let spelling = token.spelling();
        match spelling.parse() {
            Ok(result) => Ok(result),
            Err(e) => Err(ParseError {
                kind: ParseErrorKind::ParseIntegerError(e),
                location: token.start_location(),
            }),
        }
    }

    fn parse_multi_operands_expr(&mut self) -> Result<Expr, ParseError> {
        // eat the operator
        let operator_token = self.current_token_and_consume();

        let mut operands = Vec::new();
        while let Ok(expr) = self.parse_expr() {
            operands.push(expr);
        }

        match operator_token.token_kind() {
            TokenKind::Plus if operands.len() == 2 => unsafe {
                let right_operand = operands.pop().unwrap_unchecked();
                let left_operand = operands.pop().unwrap_unchecked();

                Ok(Expr::BinaryOperation {
                    kind: BinaryOpKind::Add,
                    left_operand: Box::new(left_operand),
                    right_operand: Box::new(right_operand),
                })
            },

            TokenKind::Minus if operands.len() == 2 => unsafe {
                let right_operand = operands.pop().unwrap_unchecked();
                let left_operand = operands.pop().unwrap_unchecked();
                Ok(Expr::BinaryOperation {
                    kind: BinaryOpKind::Sub,
                    left_operand: Box::new(left_operand),
                    right_operand: Box::new(right_operand),
                })
            },

            TokenKind::Minus if operands.len() == 1 => unsafe {
                Ok(Expr::UnaryOperation {
                    kind: UnaryOpKind::Minus,
                    operand: Box::new(operands.pop().unwrap_unchecked()),
                })
            },

            _ => Err(ParseError {
                kind: ParseErrorKind::InvalidOperandCount(operands.len()),
                location: operator_token.start_location(),
            }),
        }
    }

    fn parse_paren_expr(&mut self) -> Result<Expr, ParseError> {
        // eat the '('
        let lparen_token = self.current_token_and_consume();
        // Parse the body.
        let body = self.parse_expr();
        // eat the ')'
        self.expect_closing_paren_and_consume(TokenKind::RParen, &lparen_token)?;
        body
    }

    fn parse_variable_declaration(&mut self) -> Result<(&'a str, Expr), ParseError> {
        // Parse the `([var exp])` structure.

        // eat the '('
        let lparen_token = self.expect_and_consume(TokenKind::LParen)?;
        // eat the '['
        let lsquare_token = self.expect_and_consume(TokenKind::LSquare)?;

        // parse the variable name
        let variable_token = self.expect_and_consume(TokenKind::Identifier)?;
        // parse the initialiazer
        let initializer_expr = self.parse_expr()?;

        // eat the ']'
        let _ = self.expect_closing_paren_and_consume(TokenKind::RSquare, &lsquare_token)?;
        // eat the ')'
        let _ = self.expect_closing_paren_and_consume(TokenKind::RParen, &lparen_token)?;

        Ok((variable_token.spelling(), initializer_expr))
    }

    fn parse_let_expr(&mut self) -> Result<Expr, ParseError> {
        // eat the 'let' keyword
        let _ = self.current_token_and_consume();
        // Parse the variable declaration of the expression.
        let (variable_name, init_expr) = self.parse_variable_declaration()?;
        // Parse the body of the let expression.
        let body = self.parse_expr()?;

        Ok(Expr::Let {
            variable_name: variable_name.to_string(),
            init_expr: Box::new(init_expr),
            body: Box::new(body),
        })
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        let token = self.cur_token.clone();

        match token.token_kind() {
            TokenKind::Integer => Ok(Expr::Integer(self.parse_integer()?)),
            TokenKind::Read => {
                self.consume_token();
                Ok(Expr::Read)
            }
            TokenKind::Identifier => Ok(Expr::Identifier(
                self.current_token_and_consume().spelling().to_string(),
            )),
            TokenKind::Plus | TokenKind::Minus => self.parse_multi_operands_expr(),
            TokenKind::LParen => self.parse_paren_expr(),
            TokenKind::Let => self.parse_let_expr(),
            _ => Err(ParseError {
                kind: ParseErrorKind::UnexpectedToken(String::from(token.spelling())),
                location: token.start_location(),
            }),
        }
    }

    fn parse_finished(&self) -> bool {
        self.cur_token.token_kind() == TokenKind::EOF
    }
}

pub fn parse_expr(code: &str) -> Result<Expr, ParseError> {
    let mut parser = Parser::new(code);
    let result = parser.parse_expr()?;

    if parser.parse_finished() {
        Ok(result)
    } else {
        let cur_token = &parser.cur_token;
        Err(ParseError {
            kind: ParseErrorKind::UnexpectedToken(cur_token.spelling().to_string()),
            location: cur_token.start_location(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_simple_expr() {
        assert_eq!(
            parse_expr("(+ 1 2)"),
            Ok(Expr::BinaryOperation {
                kind: BinaryOpKind::Add,
                left_operand: Box::new(Expr::Integer(1)),
                right_operand: Box::new(Expr::Integer(2))
            })
        );

        assert_eq!(parse_expr("1"), Ok(Expr::Integer(1)));

        assert_eq!(
            parse_expr("-3"),
            Ok(Expr::UnaryOperation {
                kind: UnaryOpKind::Minus,
                operand: Box::new(Expr::Integer(3))
            })
        );

        assert_eq!(
            parse_expr("(( (+ 1 (3) )))"),
            Ok(Expr::BinaryOperation {
                kind: BinaryOpKind::Add,
                left_operand: Box::new(Expr::Integer(1)),
                right_operand: Box::new(Expr::Integer(3))
            })
        );

        assert_eq!(
            parse_expr("(+ 10 (- (+ 5 3)))"),
            Ok(Expr::BinaryOperation {
                kind: BinaryOpKind::Add,
                left_operand: Box::new(Expr::Integer(10)),
                right_operand: Box::new(Expr::UnaryOperation {
                    kind: UnaryOpKind::Minus,
                    operand: Box::new(Expr::BinaryOperation {
                        kind: BinaryOpKind::Add,
                        left_operand: Box::new(Expr::Integer(5)),
                        right_operand: Box::new(Expr::Integer(3))
                    })
                })
            })
        );

        assert_eq!(
            parse_expr("- (- read) (+ 3 (- 5))"),
            Ok(Expr::BinaryOperation {
                kind: BinaryOpKind::Sub,
                left_operand: Box::new(Expr::UnaryOperation {
                    kind: UnaryOpKind::Minus,
                    operand: Box::new(Expr::Read)
                }),
                right_operand: Box::new(Expr::BinaryOperation {
                    kind: BinaryOpKind::Add,
                    left_operand: Box::new(Expr::Integer(3)),
                    right_operand: Box::new(Expr::UnaryOperation {
                        kind: UnaryOpKind::Minus,
                        operand: Box::new(Expr::Integer(5))
                    })
                })
            })
        );
    }

    #[test]
    fn parse_variable() {
        assert_eq!(
            parse_expr("(let ([x 1]) x)"),
            Ok(Expr::Let {
                variable_name: "x".to_string(),
                init_expr: Box::new(Expr::Integer(1)),
                body: Box::new(Expr::Identifier("x".to_string()))
            })
        );

        assert_eq!(
            parse_expr("(let ([x1 1]) x)"),
            Ok(Expr::Let {
                variable_name: "x1".to_string(),
                init_expr: Box::new(Expr::Integer(1)),
                body: Box::new(Expr::Identifier("x".to_string()))
            })
        );

        assert_eq!(
            parse_expr("(((let ([a1b (+ 12 20)]) (+ 10 x))))"),
            Ok(Expr::Let {
                variable_name: "a1b".to_string(),
                init_expr: Box::new(Expr::BinaryOperation {
                    kind: BinaryOpKind::Add,
                    left_operand: Box::new(Expr::Integer(12)),
                    right_operand: Box::new(Expr::Integer(20))
                }),
                body: Box::new(Expr::BinaryOperation {
                    kind: BinaryOpKind::Add,
                    left_operand: Box::new(Expr::Integer(10)),
                    right_operand: Box::new(Expr::Identifier("x".to_string()))
                })
            })
        );

        assert_eq!(
            parse_expr("(let ([x (32)]) (+ (let ([x 10]) x) (x)))"),
            Ok(Expr::Let {
                variable_name: "x".to_string(),
                init_expr: Box::new(Expr::Integer(32)),
                body: Box::new(Expr::BinaryOperation {
                    kind: BinaryOpKind::Add,
                    left_operand: Box::new(Expr::Let {
                        variable_name: "x".to_string(),
                        init_expr: Box::new(Expr::Integer(10)),
                        body: Box::new(Expr::Identifier("x".to_string()))
                    }),
                    right_operand: Box::new(Expr::Identifier("x".to_string()))
                })
            })
        );

        assert_eq!(
            parse_expr("(let ([x (read)]) (let ([y (read)]) (+ x (- y))))"),
            Ok(Expr::Let {
                variable_name: "x".to_string(),
                init_expr: Box::new(Expr::Read),
                body: Box::new(Expr::Let {
                    variable_name: "y".to_string(),
                    init_expr: Box::new(Expr::Read),
                    body: Box::new(Expr::BinaryOperation {
                        kind: BinaryOpKind::Add,
                        left_operand: Box::new(Expr::Identifier("x".to_string())),
                        right_operand: Box::new(Expr::UnaryOperation {
                            kind: UnaryOpKind::Minus,
                            operand: Box::new(Expr::Identifier("y".to_string()))
                        })
                    })
                })
            })
        );
    }

    #[test]
    fn parse_error() {
        assert!(matches!(
            parse_expr("18446744073709551616"),
            Err(ParseError {
                kind: ParseErrorKind::ParseIntegerError(_),
                location: _
            })
        ));

        assert_eq!(
            parse_expr(" + 3"),
            Err(ParseError {
                kind: ParseErrorKind::InvalidOperandCount(1),
                location: 1
            })
        );

        assert_eq!(
            parse_expr(" + 3 3 1"),
            Err(ParseError {
                kind: ParseErrorKind::InvalidOperandCount(3),
                location: 1
            })
        );

        assert_eq!(
            parse_expr("- 3 3 1"),
            Err(ParseError {
                kind: ParseErrorKind::InvalidOperandCount(3),
                location: 0
            })
        );

        assert_eq!(
            parse_expr(" * 3 3 1"),
            Err(ParseError {
                kind: ParseErrorKind::UnexpectedToken("*".to_string()),
                location: 1
            })
        );

        assert_eq!(
            parse_expr(" (+ 2 3"),
            Err(ParseError {
                kind: ParseErrorKind::MismatchedOpenParen,
                location: 1
            })
        );

        assert_eq!(
            parse_expr("3 3"),
            Err(ParseError {
                kind: ParseErrorKind::UnexpectedToken("3".to_string()),
                location: 2
            })
        );

        assert_eq!(
            parse_expr("(3))"),
            Err(ParseError {
                kind: ParseErrorKind::UnexpectedToken(")".to_string()),
                location: 3
            })
        );

        assert_eq!(
            parse_expr("let [x 10] 10"),
            Err(ParseError {
                kind: ParseErrorKind::UnexpectedToken("[".to_string()),
                location: 4
            })
        );

        assert_eq!(
            parse_expr("let ([(x) 10]) 10"),
            Err(ParseError {
                kind: ParseErrorKind::UnexpectedToken("(".to_string()),
                location: 6
            })
        );

        assert_eq!(
            parse_expr("let ([x 1 2]) 10"),
            Err(ParseError {
                kind: ParseErrorKind::MismatchedOpenParen,
                location: 5
            })
        );
    }
}
