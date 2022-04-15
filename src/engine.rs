use std::collections::HashMap;

use crate::builtin;
use crate::expr::Expr;

#[derive(Clone, Copy)]
struct Symbol {
    // enum: function, constant, set
    associative: bool,
    commutative: bool,
}

pub enum Node {
    Num(i64),
    Func(String, Symbol, Vec<Box<Node>>)
}

fn read_expr_visit(expr: &Expr, symbols: &HashMap<String, Symbol>) -> Node {
    match expr {
        Expr::Num(n) => Node::Num(*n),
        Expr::Func(name, exprs) => {
            let symbol = symbols.get(name).unwrap();
            let child_nodes: Vec<_> = exprs.iter().map(|e| Box::new(read_expr_visit(e, symbols))).collect();
            Node::Func(name.clone(), symbol.clone(), child_nodes)
        }
    }
}

fn print_node(node: &Node, indent: usize) {
    print!("{}", " ".repeat(indent));
    match node {
        Node::Num(n) => println!("{}", n),
        Node::Func(name, _, children) => {
            println!("{}", name);
            for child in children {
                print_node(child, indent + 4);
            }
        }
    }
}

pub struct Engine {
    symbols: HashMap<String, Symbol>,
}

impl Engine {

    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
        }
    }

    pub fn init(&mut self) {
        self.symbols.insert(builtin::ADD.to_string(), Symbol {
            associative: true,
            commutative: true,
        });
        self.symbols.insert(builtin::MUL.to_string(), Symbol {
            associative: true,
            commutative: true,
        });
        self.symbols.insert(builtin::SUB.to_string(), Symbol {
            associative: false,
            commutative: false,
        });
        self.symbols.insert(builtin::DIV.to_string(), Symbol {
            associative: false,
            commutative: false,
        });
        self.symbols.insert(builtin::POW.to_string(), Symbol {
            associative: false,
            commutative: false,
        });
        self.symbols.insert(builtin::NEG.to_string(), Symbol {
            associative: false,
            commutative: false,
        });
    }

    pub fn read_expr(&self, expr: &Expr) -> Node {
        read_expr_visit(expr, &self.symbols)
    }

    pub fn print_tree(node: &Node) {
        print_node(node, 0);
    }

}
