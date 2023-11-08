use frontend::Expr;

use crate::NameGenerator;

struct RCOImpl {
    name_gen: NameGenerator,
}

impl RCOImpl {
    fn new() -> Self {
        Self {
            name_gen: NameGenerator::new("tmp".to_string()),
        }
    }

    fn rco_atom(&mut self, expr: Expr) -> (Expr, Vec<(String, Expr)>) {
        use Expr::*;

        match expr {
            Integer(val) => (Integer(val), Vec::new()),

            Read => (Read, Vec::new()),

            Identifier(name) => (Identifier(name), Vec::new()),

            UnaryOperation { kind, operand } => {
                let (operand, mut subexpr_list) = self.rco_atom(*operand);
                let name = self.name_gen.generate();
                subexpr_list.push((
                    name.clone(),
                    Expr::UnaryOperation {
                        kind,
                        operand: Box::new(operand),
                    },
                ));
                (Expr::Identifier(name), subexpr_list)
            }

            BinaryOperation {
                kind,
                left_operand,
                right_operand,
            } => {
                let (left_operand, mut left_subexpr_list) = self.rco_atom(*left_operand);
                let (right_operand, mut right_subexpr_list) = self.rco_atom(*right_operand);
                left_subexpr_list.append(&mut right_subexpr_list);

                let name = self.name_gen.generate();
                left_subexpr_list.push((
                    name.clone(),
                    Expr::BinaryOperation {
                        kind,
                        left_operand: Box::new(left_operand),
                        right_operand: Box::new(right_operand),
                    },
                ));
                (Expr::Identifier(name), left_subexpr_list)
            }

            Let {
                variable_name,
                init_expr,
                body,
            } => (
                Let {
                    variable_name,
                    init_expr: Box::new(self.rco_expr(*init_expr)),
                    body: Box::new(self.rco_expr(*body)),
                },
                Vec::new(),
            ),
        }
    }

    fn rco_expr(&mut self, expr: Expr) -> Expr {
        use Expr::*;

        match expr {
            Integer(val) => Integer(val),

            Read => Read,

            Identifier(name) => Identifier(name),

            UnaryOperation { kind, operand } => {
                let (operand, subexpr_list) = self.rco_atom(*operand);

                // The following code is equivalent to
                //
                // let mut body = UnaryOperation {
                //     kind,
                //     operand: Box::new(operand),
                // };
                // for (variable_name, init_expr) in subexpr_list.into_iter().rev() {
                //     body = Let {
                //         variable_name,
                //         init_expr: Box::new(init_expr),
                //         body: Box::new(body),
                //     };
                // }
                //
                // body
                subexpr_list.into_iter().rev().fold(
                    UnaryOperation {
                        kind,
                        operand: Box::new(operand),
                    },
                    |body, (variable_name, init_expr)| Let {
                        variable_name,
                        init_expr: Box::new(init_expr),
                        body: Box::new(body),
                    },
                )
            }

            BinaryOperation {
                kind,
                left_operand,
                right_operand,
            } => {
                let (left_operand, left_subexpr_list) = self.rco_atom(*left_operand);
                let (right_operand, right_subexpr_list) = self.rco_atom(*right_operand);

                // The following code is equivalent to
                //
                // let mut body = Expr::BinaryOperation {
                //     kind,
                //     left_operand: Box::new(left_operand),
                //     right_operand: Box::new(right_operand),
                // };
                // for (variable_name, init_expr) in right_subexpr_list
                //     .into_iter()
                //     .rev()
                //     .chain(left_subexpr_list.into_iter().rev())
                // {
                //     body = Expr::Let {
                //         variable_name,
                //         init_expr: Box::new(init_expr),
                //         body: Box::new(body),
                //     };
                // }
                //
                // body
                right_subexpr_list
                    .into_iter()
                    .rev()
                    .chain(left_subexpr_list.into_iter().rev())
                    .fold(
                        BinaryOperation {
                            kind,
                            left_operand: Box::new(left_operand),
                            right_operand: Box::new(right_operand),
                        },
                        |body, (variable_name, init_expr)| Expr::Let {
                            variable_name,
                            init_expr: Box::new(init_expr),
                            body: Box::new(body),
                        },
                    )
            }

            Let {
                variable_name,
                init_expr,
                body,
            } => Let {
                variable_name,
                init_expr: Box::new(self.rco_expr(*init_expr)),
                body: Box::new(self.rco_expr(*body)),
            },
        }
    }
}

pub fn remove_complex_operands(expr: Expr) -> Expr {
    RCOImpl::new().rco_expr(expr)
}

#[cfg(test)]
mod test {
    use frontend::parse_expr;

    use super::*;

    #[test]
    fn test_remove_complex_operands() {
        assert_eq!(
            remove_complex_operands(parse_expr("let ([x (+ 42 (- 10))]) (+ x 10)").unwrap())
                .to_string(),
            "(let ([x (let ([tmp0 (- 10)]) (+ 42 tmp0))]) (+ x 10))"
        );

        assert_eq!(
            remove_complex_operands(parse_expr("let ([a 42]) (let ([b a]) b)").unwrap())
                .to_string(),
            "(let ([a 42]) (let ([b a]) b))"
        );

        assert_eq!(
            parse_expr(
                &remove_complex_operands(
                    parse_expr("+ (+ (- 3) (+ 1 (- 2))) (- (+ 1 2) (- 1))").unwrap()
                )
                .to_string()
            ),
            parse_expr(
                r#" (let ([tmp0 (- 3)])
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
        );
    }
}
