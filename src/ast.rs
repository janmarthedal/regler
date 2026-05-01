use num_bigint::BigInt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Ident(String),
    Int(BigInt),
    /// Function application: `f(a, b, ...)`
    App(String, Vec<Expr>),
    BinOp(Op, Box<Expr>, Box<Expr>),
    UnaryOp(UnaryOp, Box<Expr>),
    /// `∀ vars ∈ domain. body`
    Forall(Vec<String>, Box<Expr>, Box<Expr>),
    /// `{var ∈ domain | pred}` — predicate-subset comprehension
    SetBuilder(String, Box<Expr>, Box<Expr>),
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
    Arrow,   // →  function type, prec 45, right-assoc
    Implies, // ⇒  prec 10, right-assoc
    Or,      // ∨  prec 20
    And,     // ∧  prec 30
    Eq,      // =  prec 40, non-assoc
    Ne,      // ≠  prec 40, non-assoc
    Subset,  // ⊆  prec 40, non-assoc
    In,      // ∈  prec 40, non-assoc
    Lt,      // <  prec 40, non-assoc
    Gt,      // >  prec 40, non-assoc
    Le,      // ≤  prec 40, non-assoc
    Ge,      // ≥  prec 40, non-assoc
    Add,     // +  prec 50
    Sub,     // -  prec 50
    Mul,     // ·  prec 60
    Div,     // /  prec 60
    Pow,     // ^  prec 70, right-assoc
}

impl Op {
    pub fn prec(self) -> u8 {
        match self {
            Op::Implies => 10,
            Op::Or => 20,
            Op::And => 30,
            Op::Eq | Op::Ne | Op::Subset | Op::In | Op::Lt | Op::Gt | Op::Le | Op::Ge => 40,
            Op::Add | Op::Sub => 50,
            Op::Mul | Op::Div => 60,
            Op::Pow => 70,
            Op::Arrow => 45,
        }
    }

    pub fn right_assoc(self) -> bool {
        matches!(self, Op::Pow | Op::Implies | Op::Arrow)
    }

    pub fn symbol(self) -> &'static str {
        match self {
            Op::Arrow => "→",
            Op::Implies => "⇒",
            Op::Or => "∨",
            Op::And => "∧",
            Op::Eq => "=",
            Op::Ne => "≠",
            Op::Subset => "⊆",
            Op::In => "∈",
            Op::Lt => "<",
            Op::Gt => ">",
            Op::Le => "≤",
            Op::Ge => "≥",
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
    /// `let name [: ty] [= rhs]` — declaration or definition
    Let(String, Option<Expr>, Option<Expr>),
    /// `fact [name :] proposition [if condition]`
    Fact(Option<String>, Expr, Option<Expr>),
    Print(Expr),
    Evaluate(Expr),
    Simplify(Expr),
    /// `apply name to expr`
    Apply(String, Expr),
    /// `apply ← name to expr`
    ApplyRev(String, Expr),
}
