use crate::ir::x86::{Reg, VarArg, VarBlock, VarInstr, VarProgram};

fn transform_block(block: VarBlock) -> VarBlock {
    let mut result = VarBlock::new(block.label);

    block
        .instructions
        .into_iter()
        .for_each(|instr| match instr {
            VarInstr::Addq {
                lhs: VarArg::Deref(reg_lhs, offset_lhs),
                rhs: VarArg::Deref(reg_rhs, offset_rhs),
            } => {
                result.add_instr(VarInstr::Movq {
                    from: VarArg::Deref(reg_rhs, offset_rhs),
                    to: VarArg::Reg(Reg::RAX),
                });
                result.add_instr(VarInstr::Addq {
                    lhs: VarArg::Deref(reg_lhs, offset_lhs),
                    rhs: VarArg::Reg(Reg::RAX),
                });
            }

            VarInstr::Subq {
                lhs: VarArg::Deref(reg_lhs, offset_lhs),
                rhs: VarArg::Deref(reg_rhs, offset_rhs),
            } => {
                result.add_instr(VarInstr::Movq {
                    from: VarArg::Deref(reg_rhs, offset_rhs),
                    to: VarArg::Reg(Reg::RAX),
                });
                result.add_instr(VarInstr::Subq {
                    lhs: VarArg::Deref(reg_lhs, offset_lhs),
                    rhs: VarArg::Reg(Reg::RAX),
                });
            }

            VarInstr::Movq {
                from: VarArg::Deref(reg_rhs, offset_rhs),
                to: VarArg::Deref(reg_lhs, offset_lhs),
            } => {
                result.add_instr(VarInstr::Movq {
                    from: VarArg::Deref(reg_rhs, offset_rhs),
                    to: VarArg::Reg(Reg::RAX),
                });
                result.add_instr(VarInstr::Movq {
                    from: VarArg::Reg(Reg::RAX),
                    to: VarArg::Deref(reg_lhs, offset_lhs),
                });
            }

            VarInstr::Addq {
                lhs: VarArg::Deref(reg_lhs, offset_lhs),
                rhs: VarArg::Imm(value),
            } if value > 0x10000 => {
                result.add_instr(VarInstr::Movq {
                    from: VarArg::Imm(value),
                    to: VarArg::Reg(Reg::RAX),
                });
                result.add_instr(VarInstr::Addq {
                    lhs: VarArg::Deref(reg_lhs, offset_lhs),
                    rhs: VarArg::Reg(Reg::RAX),
                });
            }

            VarInstr::Subq {
                lhs: VarArg::Deref(reg_lhs, offset_lhs),
                rhs: VarArg::Imm(value),
            } if value > 0x10000 => {
                result.add_instr(VarInstr::Movq {
                    from: VarArg::Imm(value),
                    to: VarArg::Reg(Reg::RAX),
                });
                result.add_instr(VarInstr::Subq {
                    lhs: VarArg::Deref(reg_lhs, offset_lhs),
                    rhs: VarArg::Reg(Reg::RAX),
                });
            }

            VarInstr::Movq {
                from: VarArg::Imm(value),
                to: VarArg::Deref(reg_lhs, offset_lhs),
            } if value > 0x10000 => {
                result.add_instr(VarInstr::Movq {
                    from: VarArg::Imm(value),
                    to: VarArg::Reg(Reg::RAX),
                });
                result.add_instr(VarInstr::Addq {
                    lhs: VarArg::Deref(reg_lhs, offset_lhs),
                    rhs: VarArg::Reg(Reg::RAX),
                });
            }

            other => result.add_instr(other),
        });

    result
}

pub(crate) fn patch_instructions(program: VarProgram) -> VarProgram {
    VarProgram {
        local_variables: program.local_variables,
        body: program
            .body
            .into_iter()
            .map(|block| transform_block(block))
            .collect(),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn generate_test_program(instructions: Vec<VarInstr>) -> VarProgram {
        VarProgram {
            local_variables: Vec::new(),
            body: vec![VarBlock {
                label: "test".to_string(),
                instructions,
            }],
        }
    }

    #[test]
    fn patch_instructions_test() {
        use VarArg::{Deref, Imm};

        assert_eq!(
            patch_instructions(generate_test_program(vec![
                VarInstr::Movq {
                    from: Deref(Reg::RBP, -8),
                    to: Deref(Reg::RBP, -16)
                },
                VarInstr::Subq {
                    lhs: Deref(Reg::RBP, -24),
                    rhs: Deref(Reg::RBP, -32)
                },
                VarInstr::Addq {
                    lhs: Deref(Reg::RBP, -40),
                    rhs: Imm(65537)
                },
            ]))
            .body[0]
                .instructions,
            vec![
                VarInstr::Movq {
                    from: Deref(Reg::RBP, -8),
                    to: Reg::RAX.into(),
                },
                VarInstr::Movq {
                    from: Reg::RAX.into(),
                    to: Deref(Reg::RBP, -16)
                },
                VarInstr::Movq {
                    from: Deref(Reg::RBP, -32),
                    to: Reg::RAX.into(),
                },
                VarInstr::Subq {
                    lhs: Deref(Reg::RBP, -24),
                    rhs: Reg::RAX.into(),
                },
                VarInstr::Movq {
                    from: Imm(65537),
                    to: Reg::RAX.into(),
                },
                VarInstr::Addq {
                    lhs: Deref(Reg::RBP, -40),
                    rhs: Reg::RAX.into(),
                },
            ]
        );
    }
}
