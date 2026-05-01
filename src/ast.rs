use num_bigint::BigInt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Ident(String),
    Int(BigInt),
    BinOp(Op, Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Eq,
    Add,
    Mul,
    Pow,
}

impl Op {
    pub fn prec(self) -> u8 {
        match self {
            Op::Eq => 1,
            Op::Add => 2,
            Op::Mul => 3,
            Op::Pow => 4,
        }
    }

    pub fn right_assoc(self) -> bool {
        matches!(self, Op::Pow)
    }

    pub fn symbol(self) -> &'static str {
        match self {
            Op::Eq => "=",
            Op::Add => "+",
            Op::Mul => "·",
            Op::Pow => "^",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Let(String, Expr),
    Fact(Expr),
    Print(Expr),
    Evaluate(Expr),
    Simplify(Expr),
}
