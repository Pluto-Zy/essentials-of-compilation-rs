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
    UnaryOperation {
        kind: UnaryOpKind,
        operand: Box<Expr>,
    },
    BinaryOperation {
        kind: BinaryOpKind,
        left_operand: Box<Expr>,
        right_operand: Box<Expr>,
    },
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Program {
    pub body: Expr,
}
