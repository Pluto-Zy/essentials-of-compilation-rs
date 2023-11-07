use core::fmt;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum UnaryOpKind {
    Minus, // -
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum BinaryOpKind {
    Add, // +
    Sub, // -
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Expr {
    Integer(u64),
    Read,
    // Note that we cannot use &str here, because the uniquify pass will modify the name of the
    // variable.
    Identifier(String),
    UnaryOperation {
        kind: UnaryOpKind,
        operand: Box<Expr>,
    },
    BinaryOperation {
        kind: BinaryOpKind,
        left_operand: Box<Expr>,
        right_operand: Box<Expr>,
    },
    Let {
        variable_name: String,
        init_expr: Box<Expr>,
        body: Box<Expr>,
    },
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Expr::*;

        match self {
            Integer(val) => write!(f, "{}", val),

            Read => write!(f, "read"),

            Identifier(name) => write!(f, "{}", name),

            UnaryOperation { kind, operand } => write!(
                f,
                "({} {})",
                match *kind {
                    UnaryOpKind::Minus => '-',
                },
                &operand
            ),

            BinaryOperation {
                kind,
                left_operand,
                right_operand,
            } => write!(
                f,
                "({} {} {})",
                match *kind {
                    BinaryOpKind::Add => '+',
                    BinaryOpKind::Sub => '-',
                },
                &left_operand,
                &right_operand
            ),

            Let {
                variable_name,
                init_expr,
                body,
            } => write!(f, "(let ([{} {}]) {})", variable_name, &init_expr, &body),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Program {
    pub body: Expr,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn display_expr() {
        assert_eq!(Expr::Integer(100).to_string(), "100".to_string());

        assert_eq!(
            Expr::UnaryOperation {
                kind: UnaryOpKind::Minus,
                operand: Box::new(Expr::Read)
            }
            .to_string(),
            "(- read)".to_string()
        );

        assert_eq!(
            Expr::BinaryOperation {
                kind: BinaryOpKind::Add,
                left_operand: Box::new(Expr::BinaryOperation {
                    kind: BinaryOpKind::Sub,
                    left_operand: Box::new(Expr::Identifier("abc".to_string())),
                    right_operand: Box::new(Expr::Read)
                }),
                right_operand: Box::new(Expr::UnaryOperation {
                    kind: UnaryOpKind::Minus,
                    operand: Box::new(Expr::Integer(42))
                })
            }
            .to_string(),
            "(+ (- abc read) (- 42))".to_string()
        );

        assert_eq!(
            Expr::Let {
                variable_name: "x1".to_string(),
                init_expr: Box::new(Expr::BinaryOperation {
                    kind: BinaryOpKind::Add,
                    left_operand: Box::new(Expr::Identifier("x1".to_string())),
                    right_operand: Box::new(Expr::Integer(5))
                }),
                body: Box::new(Expr::Let {
                    variable_name: "x2".to_string(),
                    init_expr: Box::new(Expr::UnaryOperation {
                        kind: UnaryOpKind::Minus,
                        operand: Box::new(Expr::Read)
                    }),
                    body: Box::new(Expr::BinaryOperation {
                        kind: BinaryOpKind::Sub,
                        left_operand: Box::new(Expr::Identifier("x1".to_string())),
                        right_operand: Box::new(Expr::Identifier("x2".to_string()))
                    })
                })
            }
            .to_string(),
            "(let ([x1 (+ x1 5)]) (let ([x2 (- read)]) (- x1 x2)))".to_string()
        );
    }
}
