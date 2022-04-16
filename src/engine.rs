use std::cmp::Ordering;
use std::collections::HashMap;

use crate::builtin;
use crate::pexpr::PExpr;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Symbol {
    // enum: function, constant, set
    associative: bool,
    commutative: bool,
}

impl PartialOrd for Symbol {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Symbol {
    fn cmp(&self, other: &Self) -> Ordering {
        let a = 2 * (self.associative as i32) + (self.commutative as i32);
        let b = 2 * (other.associative as i32) + (other.commutative as i32);
        a.cmp(&b)
    }
}

#[derive(Debug, Eq)]
pub enum Node {
    Num(i64),
    Func(String, Symbol, Vec<Box<Node>>),
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Node::Num(i1), Node::Num(i2)) => i1.cmp(i2),
            (Node::Num(_), Node::Func(_, _, _)) => Ordering::Greater,
            (Node::Func(_, _, _), Node::Num(_)) => Ordering::Less,
            (Node::Func(n1, s1, e1), Node::Func(n2, s2, e2)) => {
                let c = n1.cmp(n2);
                if c != Ordering::Equal {
                    return c;
                }
                let c = s1.cmp(s2);
                if c != Ordering::Equal {
                    return c;
                }
                e1.cmp(e2)
            }
        }
    }
}

fn read_expr_visit(expr: &PExpr, symbols: &HashMap<String, Symbol>) -> Node {
    match expr {
        PExpr::Num(n) => Node::Num(*n),
        PExpr::Func(name, exprs) => {
            let symbol = symbols.get(name).unwrap();
            let child_nodes: Vec<_> = exprs
                .iter()
                .map(|e| Box::new(read_expr_visit(e, symbols)))
                .collect();
            Node::Func(name.clone(), *symbol, child_nodes)
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

pub fn normalize_visit(node: &Node) -> Node {
    match node {
        Node::Num(i) => Node::Num(*i),
        Node::Func(name, symbols, children) => {
            let mut new_children: Vec<Box<Node>> = Vec::with_capacity(children.len());
            for child in children {
                let n = normalize_visit(child);
                match n {
                    Node::Func(child_name, _, child_children)
                        if symbols.associative && *name == child_name =>
                    {
                        new_children.extend(child_children)
                    }
                    _ => new_children.push(Box::new(n)),
                };
            }
            if symbols.commutative {
                new_children.sort();
            }
            Node::Func(name.clone(), *symbols, new_children)
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
        self.symbols.insert(
            builtin::ADD.to_string(),
            Symbol {
                associative: true,
                commutative: true,
            },
        );
        self.symbols.insert(
            builtin::MUL.to_string(),
            Symbol {
                associative: true,
                commutative: true,
            },
        );
        self.symbols.insert(
            builtin::SUB.to_string(),
            Symbol {
                associative: false,
                commutative: false,
            },
        );
        self.symbols.insert(
            builtin::DIV.to_string(),
            Symbol {
                associative: false,
                commutative: false,
            },
        );
        self.symbols.insert(
            builtin::POW.to_string(),
            Symbol {
                associative: false,
                commutative: false,
            },
        );
        self.symbols.insert(
            builtin::NEG.to_string(),
            Symbol {
                associative: false,
                commutative: false,
            },
        );
    }

    pub fn read_expr(&self, expr: &PExpr) -> Node {
        read_expr_visit(expr, &self.symbols)
    }

    pub fn print_tree(node: &Node) {
        print_node(node, 0);
    }

    pub fn normalize(node: &Node) -> Node {
        normalize_visit(node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_cmp() {
        let n1 = Node::Func(
            "foo".to_string(),
            Symbol {
                associative: false,
                commutative: false,
            },
            vec![],
        );
        let n2 = Node::Num(2);
        assert_eq!(n1.cmp(&n2), Ordering::Less);
        assert_eq!(n2.cmp(&n1), Ordering::Greater);
    }

    #[test]
    fn node_sort() {
        let n1 = Node::Func(
            "foo".to_string(),
            Symbol {
                associative: false,
                commutative: false,
            },
            vec![],
        );
        let n2 = Node::Num(2);
        let n3 = Node::Num(5);
        let mut v = vec![n3, n1, n2];
        v.sort();
        assert!(match (&v[0], &v[1], &v[2]) {
            (Node::Func(_, _, _), Node::Num(2), Node::Num(5)) => true,
            _ => false,
        })
    }
}
