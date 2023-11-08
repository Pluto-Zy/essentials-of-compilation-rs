use std::fmt::{Display, Formatter};

#[derive(Debug, Eq, PartialEq, Clone)]
pub(crate) enum Atom {
    Integer(i64),
    Variable(String),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub(crate) enum UnaryOpKind {
    Minus, // -
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub(crate) enum BinaryOpKind {
    Add, // +
    Sub, // -
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub(crate) enum Expr {
    Atom(Atom),
    Read,
    UnaryOperation {
        kind: UnaryOpKind,
        operand: Atom,
    },
    BinaryOperation {
        kind: BinaryOpKind,
        left_operand: Atom,
        right_operand: Atom,
    },
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub(crate) enum Stmt {
    Assign { lhs: String, rhs: Expr },
    Return(Expr),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub(crate) struct Program {
    pub(crate) locals: Vec<String>,
    pub(crate) body: Vec<Stmt>,
}

impl Program {
    pub(crate) fn new() -> Self {
        Self {
            locals: Vec::new(),
            body: Vec::new(),
        }
    }

    pub(crate) fn create_terminator(&mut self, expr: Expr) {
        self.body.push(Stmt::Return(expr));
    }

    pub(crate) fn create_assign(&mut self, lhs: String, rhs: Expr) {
        self.body.push(Stmt::Assign { lhs, rhs });
    }

    pub(crate) fn create_local_variable(&mut self, name: String) {
        self.locals.push(name);
    }
}

impl From<Atom> for Expr {
    fn from(value: Atom) -> Self {
        Self::Atom(value)
    }
}

impl From<frontend::UnaryOpKind> for UnaryOpKind {
    fn from(value: frontend::UnaryOpKind) -> Self {
        match value {
            frontend::UnaryOpKind::Minus => UnaryOpKind::Minus,
        }
    }
}

impl From<frontend::BinaryOpKind> for BinaryOpKind {
    fn from(value: frontend::BinaryOpKind) -> Self {
        match value {
            frontend::BinaryOpKind::Add => BinaryOpKind::Add,
            frontend::BinaryOpKind::Sub => BinaryOpKind::Sub,
        }
    }
}

impl Display for Atom {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Atom::Integer(val) => write!(f, "{}", val),
            Atom::Variable(name) => write!(f, "{}", name),
        }
    }
}

impl Display for UnaryOpKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOpKind::Minus => write!(f, "-"),
        }
    }
}

impl Display for BinaryOpKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOpKind::Add => write!(f, "+"),
            BinaryOpKind::Sub => write!(f, "-"),
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use Expr::*;

        match self {
            Atom(atom) => write!(f, "{}", atom),
            Read => write!(f, "read"),
            UnaryOperation { kind, operand } => write!(f, "({} {})", kind, operand),
            BinaryOperation {
                kind,
                left_operand,
                right_operand,
            } => write!(f, "({} {} {})", kind, left_operand, right_operand),
        }
    }
}

impl Display for Stmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Stmt::Assign { lhs, rhs } => write!(f, "{} = {};", lhs, rhs),
            Stmt::Return(expr) => write!(f, "return {};", expr),
        }
    }
}

impl Display for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if !self.locals.is_empty() {
            writeln!(f, "local: {:?}", self.locals)?;
        }

        writeln!(f, "start:")?;
        self.body
            .iter()
            .try_for_each(|stmt| writeln!(f, "    {}", stmt))
    }
}
