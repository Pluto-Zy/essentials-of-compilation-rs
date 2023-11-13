use crate::ir::{
    cvar::{Atom, BinaryOpKind, Expr, Program, Stmt, UnaryOpKind},
    x86::{Block, Reg, VarArg, VarInstr, VarProgram},
};

struct SelectInstrImpl {
    result_program: VarProgram,
}

impl SelectInstrImpl {
    fn new() -> Self {
        Self {
            result_program: VarProgram::new(),
        }
    }

    fn read_int_func_name() -> String {
        "read_int".to_string()
    }

    fn rax_reg() -> VarArg {
        Reg::RAX.into()
    }

    fn generate_result_target(actual_result: VarArg, expected_result: VarArg) -> Option<VarInstr> {
        match actual_result {
            VarArg::Variable(_) if actual_result == expected_result => None,
            _ => Some(VarInstr::Movq {
                from: expected_result,
                to: actual_result,
            }),
        }
    }

    fn handle_atom(atom: Atom) -> VarArg {
        match atom {
            Atom::Integer(val) => VarArg::Imm(val),
            Atom::Variable(name) => VarArg::Variable(name),
        }
    }

    fn handle_expr(expr: Expr, result: VarArg, target_block: &mut Block<VarArg>) {
        match expr {
            Expr::Atom(atom) => target_block.add_instr(VarInstr::Movq {
                from: Self::handle_atom(atom),
                to: result,
            }),

            Expr::Read => {
                target_block.add_instr(VarInstr::Callq {
                    callee: Self::read_int_func_name(),
                });
                target_block.add_instr(VarInstr::Movq {
                    from: Self::rax_reg(),
                    to: result,
                });
            }

            Expr::UnaryOperation { kind, operand } => {
                if let Some(instr) =
                    Self::generate_result_target(result.clone(), Self::handle_atom(operand))
                {
                    target_block.add_instr(instr);
                }

                match kind {
                    UnaryOpKind::Minus => {
                        target_block.add_instr(VarInstr::Negq { operand: result })
                    }
                }
            }

            Expr::BinaryOperation {
                kind,
                left_operand,
                right_operand,
            } => {
                if let Some(instr) =
                    Self::generate_result_target(result.clone(), Self::handle_atom(left_operand))
                {
                    target_block.add_instr(instr);
                }

                match kind {
                    BinaryOpKind::Add => target_block.add_instr(VarInstr::Addq {
                        lhs: result,
                        rhs: Self::handle_atom(right_operand),
                    }),

                    BinaryOpKind::Sub => target_block.add_instr(VarInstr::Subq {
                        lhs: result,
                        rhs: Self::handle_atom(right_operand),
                    }),
                }
            }
        }
    }

    fn handle_stmt(stmt: Stmt, target_block: &mut Block<VarArg>) {
        match stmt {
            Stmt::Assign { lhs, rhs } => {
                Self::handle_expr(rhs, VarArg::Variable(lhs), target_block);
            }

            Stmt::Return(operand) => {
                Self::handle_expr(operand, Self::rax_reg(), target_block);
                target_block.add_instr(VarInstr::Jmp {
                    target: "conclusion".to_string(),
                });
            }
        }
    }

    fn handle_program(mut self, program: Program) -> Self {
        // Create new blocks.
        let mut main_block: Block<VarArg> = Block::new("main".to_string());
        let conclusion_block: Block<VarArg> = Block::new("conclusion".to_string());

        program
            .body
            .into_iter()
            .for_each(|stmt| Self::handle_stmt(stmt, &mut main_block));

        self.result_program.body.push(main_block);
        self.result_program.body.push(conclusion_block);

        self
    }
}

pub(crate) fn select_instructions(program: Program) -> VarProgram {
    SelectInstrImpl::new()
        .handle_program(program)
        .result_program
}

#[cfg(test)]
mod test {
    use frontend::parse_expr;

    use crate::explicate_control::explicate_control;

    use super::*;

    fn prepare_program(code: &str) -> Program {
        explicate_control(parse_expr(code).unwrap())
    }

    #[test]
    fn select_instructions_test() {
        assert_eq!(
            select_instructions(prepare_program(
                "let ([y (let ([x1 (- 20)]) (let ([x2 22]) (+ x1 x2)))]) y"
            ))
            .to_string()
            .trim(),
            r#"
main:
    movq    $0x14, x1
    negq    x1
    movq    $0x16, x2
    movq    x1, y
    addq    x2, y
    movq    y, %rax
    jmp     conclusion
conclusion:
    "#
            .trim()
        );

        assert_eq!(
            select_instructions(prepare_program(
                "let ([x1 read]) (let ([x2 (- x1 15)]) (+ x1 x2))"
            ))
            .to_string()
            .trim(),
            r#"
main:
    callq   read_int
    movq    %rax, x1
    movq    x1, x2
    subq    $0xf, x2
    movq    x1, %rax
    addq    x2, %rax
    jmp     conclusion
conclusion:
    "#
            .trim()
        );
    }
}
