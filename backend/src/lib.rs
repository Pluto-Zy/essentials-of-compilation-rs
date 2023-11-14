mod assign_homes;
mod explicate_control;
mod ir;
mod remove_complex_operands;
mod select_instructions;
mod uniquify;

pub(crate) struct NameGenerator {
    prefix: String,
    index: u32,
}

impl NameGenerator {
    pub(crate) fn new(prefix: String) -> Self {
        Self { prefix, index: 0 }
    }

    pub(crate) fn generate(&mut self) -> String {
        let result = format!("{}{}", self.prefix, self.index);
        self.index += 1;
        result
    }
}
