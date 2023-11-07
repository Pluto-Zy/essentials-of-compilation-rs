use std::collections::HashMap;

use frontend::Expr;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum PassError {
    UnknownIdentifier(String),
}

struct NameGenerator {
    prefix: String,
    index: u32,
}

impl NameGenerator {
    fn new(prefix: String) -> Self {
        Self { prefix, index: 0 }
    }

    fn generate(&mut self) -> String {
        let result = format!("{}{}", self.prefix, self.index);
        self.index += 1;
        result
    }
}

struct UniquifyImpl {
    name_gen: NameGenerator,
    symbol_table: Vec<HashMap<String, String>>,
}

impl UniquifyImpl {
    fn new() -> Self {
        Self {
            name_gen: NameGenerator::new("x".to_string()),
            symbol_table: Vec::new(),
        }
    }

    fn enter_scope(&mut self) {
        self.symbol_table.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        self.symbol_table.pop();
    }

    fn gen_and_declare_unique_name(&mut self, src_name: &str) -> String {
        let unique_name = self.name_gen.generate();
        self.symbol_table
            .last_mut()
            .unwrap()
            .insert(src_name.to_string(), unique_name.clone());
        unique_name
    }

    fn lookup(&self, name: &str) -> Option<&String> {
        self.symbol_table
            .iter()
            .rev()
            .find_map(|table| table.get(name))
    }

    fn run_on_expr(&mut self, expr: Expr) -> Result<Expr, PassError> {
        use Expr::*;

        match expr {
            Integer(val) => Ok(Integer(val)),

            Read => Ok(Read),

            Identifier(name) => match self.lookup(&name) {
                Some(result) => Ok(Identifier(result.clone())),
                None => Err(PassError::UnknownIdentifier(name)),
            },

            UnaryOperation { kind, operand } => Ok(UnaryOperation {
                kind,
                operand: Box::new(self.run_on_expr(*operand)?),
            }),

            BinaryOperation {
                kind,
                left_operand,
                right_operand,
            } => Ok(BinaryOperation {
                kind,
                left_operand: Box::new(self.run_on_expr(*left_operand)?),
                right_operand: Box::new(self.run_on_expr(*right_operand)?),
            }),

            Let {
                variable_name,
                init_expr,
                body,
            } => {
                // We process the initializer before entering the scope of the let expression, so
                // that the initializer will use the variable in the parent scope.
                let init_expr = Box::new(self.run_on_expr(*init_expr)?);
                self.enter_scope();
                let variable_name = self.gen_and_declare_unique_name(&variable_name);
                let body = Box::new(self.run_on_expr(*body)?);
                self.exit_scope();
                Ok(Let {
                    variable_name,
                    init_expr,
                    body,
                })
            }
        }
    }
}

pub fn uniquify_expr(expr: Expr) -> Result<Expr, PassError> {
    UniquifyImpl::new().run_on_expr(expr)
}

#[cfg(test)]
mod test {
    use frontend::parse_expr;

    use super::*;

    #[test]
    fn uniquify() {
        assert_eq!(
            uniquify_expr(parse_expr("let ([x 0]) x").unwrap())
                .unwrap()
                .to_string(),
            "(let ([x0 0]) x0)"
        );

        assert_eq!(
            uniquify_expr(parse_expr("let ([x0 0]) x0").unwrap())
                .unwrap()
                .to_string(),
            "(let ([x0 0]) x0)"
        );

        assert_eq!(
            uniquify_expr(parse_expr("let ([x1 0]) x1").unwrap())
                .unwrap()
                .to_string(),
            "(let ([x0 0]) x0)"
        );

        // shadow
        assert_eq!(
            uniquify_expr(parse_expr("let ([x (+ 1 2)]) (+ (let ([x x]) x) x)").unwrap())
                .unwrap()
                .to_string(),
            "(let ([x0 (+ 1 2)]) (+ (let ([x1 x0]) x1) x0))"
        );

        assert_eq!(
            uniquify_expr(parse_expr("let ([x0 (let ([x0 1]) (+ x0 2))]) (- x0)").unwrap())
                .unwrap()
                .to_string(),
            "(let ([x1 (let ([x0 1]) (+ x0 2))]) (- x1))"
        );
    }
}
