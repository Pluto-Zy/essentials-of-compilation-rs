use std::{
    collections::HashMap,
    io,
    num::{ParseIntError, TryFromIntError},
};

use crate::{BinaryOpKind, Expr, UnaryOpKind};

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum OverflowKind {
    NegOverflow(i64),
    AddOverflow(i64, i64),
    SubOverflow(i64, i64),
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum InterpreterError {
    IntegerConversionError(TryFromIntError),
    ParseIntegerError(ParseIntError),
    ArithmeticOverflow(OverflowKind),
    UnknownIdentifier(String),
}

impl From<TryFromIntError> for InterpreterError {
    fn from(value: TryFromIntError) -> Self {
        InterpreterError::IntegerConversionError(value)
    }
}

impl From<ParseIntError> for InterpreterError {
    fn from(value: ParseIntError) -> Self {
        InterpreterError::ParseIntegerError(value)
    }
}

struct Interpreter {
    symbol_table: Vec<HashMap<String, i64>>,
}

impl Interpreter {
    fn new() -> Self {
        Self {
            symbol_table: Vec::new(),
        }
    }

    fn enter_scope(&mut self) {
        self.symbol_table.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        self.symbol_table.pop();
    }

    fn declare_name(&mut self, name: &str, value: i64) -> bool {
        self.symbol_table
            .last_mut()
            .unwrap()
            .insert(name.to_string(), value)
            .is_none()
    }

    fn lookup(&self, name: &str) -> Option<i64> {
        self.symbol_table
            .iter()
            .rev()
            .find_map(|table| table.get(name))
            .and_then(|&value| Some(value))
    }

    fn evaluate_expr(&mut self, expr: &Expr) -> Result<i64, InterpreterError> {
        use Expr::*;

        match *expr {
            Integer(val) => Ok(val.try_into()?),

            Read => {
                let mut input = String::new();
                io::stdin()
                    .read_line(&mut input)
                    .expect("Expected to read an integer.");
                Ok(input.trim().parse()?)
            }

            Identifier(ref name) => match self.lookup(name) {
                Some(value) => Ok(value),
                None => Err(InterpreterError::UnknownIdentifier(name.clone())),
            },

            UnaryOperation {
                kind: UnaryOpKind::Minus,
                ref operand,
            } => {
                let operand = self.evaluate_expr(operand)?;
                let (result, overflow) = operand.overflowing_neg();
                if overflow {
                    Err(InterpreterError::ArithmeticOverflow(
                        OverflowKind::NegOverflow(operand),
                    ))
                } else {
                    Ok(result)
                }
            }

            BinaryOperation {
                kind: BinaryOpKind::Add,
                ref left_operand,
                ref right_operand,
            } => {
                let lhs = self.evaluate_expr(left_operand)?;
                let rhs = self.evaluate_expr(right_operand)?;
                let (result, overflow) = lhs.overflowing_add(rhs);
                if overflow {
                    Err(InterpreterError::ArithmeticOverflow(
                        OverflowKind::AddOverflow(lhs, rhs),
                    ))
                } else {
                    Ok(result)
                }
            }

            BinaryOperation {
                kind: BinaryOpKind::Sub,
                ref left_operand,
                ref right_operand,
            } => {
                let lhs = self.evaluate_expr(left_operand)?;
                let rhs = self.evaluate_expr(right_operand)?;
                let (result, overflow) = lhs.overflowing_sub(rhs);
                if overflow {
                    Err(InterpreterError::ArithmeticOverflow(
                        OverflowKind::SubOverflow(lhs, rhs),
                    ))
                } else {
                    Ok(result)
                }
            }

            Let {
                ref variable_name,
                ref init_expr,
                ref body,
            } => {
                // We evaluate the initializer before entering the scope of the let expression, so
                // that the initializer can use the variable in the parent scope.
                let init = self.evaluate_expr(&init_expr)?;
                self.enter_scope();
                // We don't handle the result of `declare_name`, since in the current language, we
                // cannot define variables with the same name in the same scope.
                self.declare_name(&variable_name, init);
                let result = self.evaluate_expr(&body)?;
                self.exit_scope();
                Ok(result)
            }
        }
    }
}

pub fn interp_expr(expr: &Expr) -> Result<i64, InterpreterError> {
    Interpreter::new().evaluate_expr(expr)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn interp_test() {
        assert_eq!(interp_expr(&Expr::Integer(255)), Ok(255));

        assert_eq!(
            interp_expr(&Expr::UnaryOperation {
                kind: UnaryOpKind::Minus,
                operand: Box::new(Expr::Integer(3))
            }),
            Ok(-3)
        );

        assert_eq!(
            interp_expr(&Expr::UnaryOperation {
                kind: UnaryOpKind::Minus,
                operand: Box::new(Expr::UnaryOperation {
                    kind: UnaryOpKind::Minus,
                    operand: Box::new(Expr::Integer(5))
                })
            }),
            Ok(5)
        );

        assert_eq!(
            interp_expr(&Expr::BinaryOperation {
                kind: BinaryOpKind::Add,
                left_operand: Box::new(Expr::Integer(1)),
                right_operand: Box::new(Expr::Integer(2))
            }),
            Ok(3)
        );

        assert_eq!(
            interp_expr(&Expr::BinaryOperation {
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
            }),
            Ok(2)
        );

        assert_eq!(
            interp_expr(&Expr::BinaryOperation {
                kind: BinaryOpKind::Sub,
                left_operand: Box::new(Expr::UnaryOperation {
                    kind: UnaryOpKind::Minus,
                    operand: Box::new(Expr::Integer(1))
                }),
                right_operand: Box::new(Expr::BinaryOperation {
                    kind: BinaryOpKind::Add,
                    left_operand: Box::new(Expr::Integer(3)),
                    right_operand: Box::new(Expr::UnaryOperation {
                        kind: UnaryOpKind::Minus,
                        operand: Box::new(Expr::Integer(5))
                    })
                })
            }),
            Ok(1)
        );
    }

    #[test]
    fn interp_variable() {
        assert_eq!(
            interp_expr(&Expr::Let {
                variable_name: "x".to_string(),
                init_expr: Box::new(Expr::Integer(1)),
                body: Box::new(Expr::Identifier("x".to_string()))
            }),
            Ok(1)
        );

        assert_eq!(
            interp_expr(&Expr::Let {
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
            }),
            Ok(42)
        );

        assert_eq!(
            interp_expr(&Expr::Let {
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
            }),
            Ok(42)
        );
    }

    #[test]
    fn interp_error() {
        assert!(matches!(
            interp_expr(&Expr::Integer(u64::MAX)),
            Err(InterpreterError::IntegerConversionError(_))
        ));

        assert_eq!(
            interp_expr(&Expr::UnaryOperation {
                kind: UnaryOpKind::Minus,
                operand: Box::new(Expr::BinaryOperation {
                    kind: BinaryOpKind::Sub,
                    left_operand: Box::new(Expr::UnaryOperation {
                        kind: UnaryOpKind::Minus,
                        operand: Box::new(Expr::Integer(i64::MAX as u64))
                    }),
                    right_operand: Box::new(Expr::Integer(1))
                })
            }),
            Err(InterpreterError::ArithmeticOverflow(
                OverflowKind::NegOverflow(i64::MIN)
            ))
        );

        assert_eq!(
            interp_expr(&Expr::BinaryOperation {
                kind: BinaryOpKind::Sub,
                left_operand: Box::new(Expr::BinaryOperation {
                    kind: BinaryOpKind::Sub,
                    left_operand: Box::new(Expr::UnaryOperation {
                        kind: UnaryOpKind::Minus,
                        operand: Box::new(Expr::Integer(i64::MAX as u64))
                    }),
                    right_operand: Box::new(Expr::Integer(1))
                }),
                right_operand: Box::new(Expr::Integer(1))
            }),
            Err(InterpreterError::ArithmeticOverflow(
                OverflowKind::SubOverflow(i64::MIN, 1)
            ))
        );

        assert_eq!(
            interp_expr(&Expr::BinaryOperation {
                kind: BinaryOpKind::Add,
                left_operand: Box::new(Expr::Integer(i64::MAX as u64)),
                right_operand: Box::new(Expr::Integer(1))
            }),
            Err(InterpreterError::ArithmeticOverflow(
                OverflowKind::AddOverflow(i64::MAX, 1)
            ))
        );

        assert_eq!(
            interp_expr(&Expr::Let {
                variable_name: "x1".to_string(),
                init_expr: Box::new(Expr::Integer(1)),
                body: Box::new(Expr::Identifier("x".to_string()))
            }),
            Err(InterpreterError::UnknownIdentifier("x".to_string()))
        );
    }
}
