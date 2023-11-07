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

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Program {
    pub body: Expr,
}
