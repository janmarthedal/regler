use num_bigint::BigInt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Ident(String),
    Int(BigInt),
    BinOp(Op, Box<Expr>, Box<Expr>),
    UnaryOp(UnaryOp, Box<Expr>),
    /// `∀ vars ∈ domain. body`
    Forall(Vec<String>, Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
}

impl UnaryOp {
    pub fn symbol(self) -> &'static str {
        match self {
            UnaryOp::Neg => "-",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Implies,
    Or,
    And,
    Eq,
    Ne,
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

impl Op {
    pub fn prec(self) -> u8 {
        match self {
            Op::Implies => 10,
            Op::Or => 20,
            Op::And => 30,
            Op::Eq | Op::Ne => 40,
            Op::Add | Op::Sub => 50,
            Op::Mul | Op::Div => 60,
            Op::Pow => 70,
        }
    }

    pub fn right_assoc(self) -> bool {
        matches!(self, Op::Pow | Op::Implies)
    }

    pub fn symbol(self) -> &'static str {
        match self {
            Op::Implies => "⇒",
            Op::Or => "∨",
            Op::And => "∧",
            Op::Eq => "=",
            Op::Ne => "≠",
            Op::Add => "+",
            Op::Sub => "-",
            Op::Mul => "·",
            Op::Div => "/",
            Op::Pow => "^",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Let(String, Expr),
    /// `fact [name :] proposition [if condition]`
    Fact(Option<String>, Expr, Option<Expr>),
    Print(Expr),
    Evaluate(Expr),
    Simplify(Expr),
    /// `apply name to expr` — apply named fact LHS→RHS to first matching subterm
    Apply(String, Expr),
    /// `apply ← name to expr` — apply named fact RHS→LHS
    ApplyRev(String, Expr),
}
