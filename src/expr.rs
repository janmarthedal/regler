#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Num(i64),
    Func(String, Vec<Expr>),
}
