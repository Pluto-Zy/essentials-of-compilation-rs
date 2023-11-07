use std::{
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

pub fn interp_expr(expr: &Expr) -> Result<i64, InterpreterError> {
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

        UnaryOperation {
            kind: UnaryOpKind::Minus,
            ref operand,
        } => {
            let operand = interp_expr(operand)?;
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
            let lhs = interp_expr(left_operand)?;
            let rhs = interp_expr(right_operand)?;
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
            let lhs = interp_expr(left_operand)?;
            let rhs = interp_expr(right_operand)?;
            let (result, overflow) = lhs.overflowing_sub(rhs);
            if overflow {
                Err(InterpreterError::ArithmeticOverflow(
                    OverflowKind::SubOverflow(lhs, rhs),
                ))
            } else {
                Ok(result)
            }
        }

        _ => todo!(),
    }
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
    }
}
