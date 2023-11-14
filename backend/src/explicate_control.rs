use crate::ir::cvar::{Atom, Expr as CExpr, Program as CProgram};
use frontend::Expr as LExpr;

struct ExplicateImpl {
    result_program: CProgram,
}

impl ExplicateImpl {
    fn new() -> Self {
        Self {
            result_program: CProgram::new(),
        }
    }

    fn gen_atom(expr: LExpr) -> Atom {
        match expr {
            LExpr::Integer(val) => Atom::Integer(val as i64),
            LExpr::Identifier(name) => Atom::Variable(name),
            _ => unreachable!(),
        }
    }

    fn explicate_tail(mut self, expr: LExpr) -> Self {
        match expr {
            LExpr::Let {
                variable_name,
                init_expr,
                body,
            } => {
                let rhs = self.explicate_assign(*init_expr);
                self.result_program
                    .create_local_variable(variable_name.clone());
                self.result_program.create_assign(variable_name, rhs);
                self = self.explicate_tail(*body);
            }

            other => {
                let operand = self.explicate_assign(other);
                self.result_program.create_terminator(operand);
            }
        }

        self
    }

    fn explicate_assign(&mut self, expr: LExpr) -> CExpr {
        match expr {
            LExpr::Integer(val) => Atom::Integer(val as i64).into(),

            LExpr::Read => CExpr::Read,

            LExpr::Identifier(name) => Atom::Variable(name).into(),

            LExpr::UnaryOperation { kind, operand } => CExpr::UnaryOperation {
                kind: kind.into(),
                operand: Self::gen_atom(*operand),
            },

            LExpr::BinaryOperation {
                kind,
                left_operand,
                right_operand,
            } => CExpr::BinaryOperation {
                kind: kind.into(),
                left_operand: Self::gen_atom(*left_operand),
                right_operand: Self::gen_atom(*right_operand),
            },

            LExpr::Let {
                variable_name,
                init_expr,
                body,
            } => {
                let init = self.explicate_assign(*init_expr);
                self.result_program
                    .create_local_variable(variable_name.clone());
                self.result_program.create_assign(variable_name, init);
                self.explicate_assign(*body)
            }
        }
    }
}

pub(crate) fn explicate_control(expr: LExpr) -> CProgram {
    ExplicateImpl::new().explicate_tail(expr).result_program
}

#[cfg(test)]
mod test {
    use frontend::parse_expr;

    use super::*;

    #[test]
    fn test_explicate_control() {
        assert_eq!(
            explicate_control(parse_expr("+ 1 2").unwrap()).to_string(),
            r#"
start:
    return (+ 1 2);
"#
            .trim_start()
        );

        assert_eq!(
            explicate_control(
                parse_expr("let ([y (let ([x1 20]) (let ([x2 22]) (+ x1 x2)))]) y").unwrap()
            )
            .to_string(),
            r#"
local: [x1, x2, y]
start:
    x1 = 20;
    x2 = 22;
    y = (+ x1 x2);
    return y;
"#
            .trim_start()
        );

        assert_eq!(
            explicate_control(
                parse_expr(
                    r#"(let ([tmp0 (- 3)])
                    (let ([tmp1 (- 2)])
                        (let ([tmp2 (+ 1 tmp1)])
                            (let ([tmp3 (+ tmp0 tmp2)])
                                (let ([tmp4 (+ 1 2)])
                                    (let ([tmp5 (- 1)])
                                        (let ([tmp6 (- tmp4 tmp5)])
                                            (+ tmp3 tmp6)
                                        )
                                    )
                                )
                            )
                        )
                    )
                )"#
                )
                .unwrap()
            )
            .to_string(),
            r#"
local: [tmp0, tmp1, tmp2, tmp3, tmp4, tmp5, tmp6]
start:
    tmp0 = (- 3);
    tmp1 = (- 2);
    tmp2 = (+ 1 tmp1);
    tmp3 = (+ tmp0 tmp2);
    tmp4 = (+ 1 2);
    tmp5 = (- 1);
    tmp6 = (- tmp4 tmp5);
    return (+ tmp3 tmp6);
"#
            .trim_start()
        );
    }
}
