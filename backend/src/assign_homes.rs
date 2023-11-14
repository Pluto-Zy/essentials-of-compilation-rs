use std::collections::HashMap;

use crate::ir::x86::{Reg, VarArg, VarBlock, VarInstr, VarProgram};

struct AssignHomesImpl {
    // Map variable to the offset of its storage relative to %rbp.
    variable_locations: HashMap<String, i64>,
}

impl AssignHomesImpl {
    fn new() -> Self {
        Self {
            variable_locations: HashMap::new(),
        }
    }

    fn rbp_reg(offset: i64) -> VarArg {
        VarArg::Deref(Reg::RBP, offset)
    }

    fn assign_homes_for_variables(&mut self, variables: Vec<String>) {
        let mut offset = -8;
        variables.into_iter().for_each(|name| {
            self.variable_locations.insert(name, offset);
            offset -= 8;
        });
    }

    fn modify_arg(&self, arg: &mut VarArg) {
        match arg {
            VarArg::Variable(name) => {
                *arg = Self::rbp_reg(*self.variable_locations.get(name).unwrap())
            }
            _ => (),
        };
    }

    fn modify_block(&self, block: &mut VarBlock) {
        block.instructions.iter_mut().for_each(|instr| match instr {
            VarInstr::Addq { lhs, rhs }
            | VarInstr::Subq { lhs, rhs }
            | VarInstr::Movq { from: lhs, to: rhs } => {
                self.modify_arg(lhs);
                self.modify_arg(rhs);
            }

            VarInstr::Negq { operand }
            | VarInstr::Pushq { operand }
            | VarInstr::Popq { operand } => {
                self.modify_arg(operand);
            }

            // Make sure that we won't miss some cases if we modify the VarInstr enum.
            VarInstr::Callq { callee: _ } | VarInstr::Retq | VarInstr::Jmp { target: _ } => (),
        });
    }

    fn modify_program(&self, program_body: &mut Vec<VarBlock>) {
        program_body
            .iter_mut()
            .for_each(|block| self.modify_block(block));
    }
}

pub(crate) fn assign_homes(mut program: VarProgram) -> VarProgram {
    let mut pass_impl = AssignHomesImpl::new();
    pass_impl.assign_homes_for_variables(program.local_variables);
    program.local_variables = Vec::new();
    pass_impl.modify_program(&mut program.body);
    program
}

#[cfg(test)]
mod test {
    use frontend::parse_expr;

    use crate::{explicate_control::explicate_control, select_instructions::select_instructions};

    use super::*;

    fn prepare_program(code: &str) -> VarProgram {
        select_instructions(explicate_control(parse_expr(code).unwrap()))
    }

    #[test]
    fn assign_homes_test() {
        assert_eq!(
            assign_homes(prepare_program("let ([a 42]) (let ([b a]) b)"))
                .to_string()
                .trim(),
            r#"
main:
    movq    $0x2a, -8(%rbp)
    movq    -8(%rbp), -16(%rbp)
    movq    -16(%rbp), %rax
    jmp     conclusion
conclusion:
    "#
            .trim()
        );

        assert_eq!(
            assign_homes(prepare_program(
                "let ([y (let ([x1 (- 20)]) (let ([x2 22]) (+ x1 x2)))]) y"
            ))
            .to_string()
            .trim(),
            r#"
main:
    movq    $0x14, -8(%rbp)
    negq    -8(%rbp)
    movq    $0x16, -16(%rbp)
    movq    -8(%rbp), -24(%rbp)
    addq    -16(%rbp), -24(%rbp)
    movq    -24(%rbp), %rax
    jmp     conclusion
conclusion:
    "#
            .trim()
        );
    }
}
