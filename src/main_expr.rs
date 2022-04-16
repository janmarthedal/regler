use std::cmp::Ordering;

use crate::pexpr::PExpr;
use crate::symbols::{FuncAttr, Symbols};

#[derive(Clone)]
enum Node {
    Num(i64),
    Func(String, FuncAttr, Vec<usize>),
}

#[derive(Clone)]
struct NodeContainer {
    nodes: Vec<Node>,
}

pub struct MainExpr {
    nodes: NodeContainer,
    root: usize,
}

impl NodeContainer {
    fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    fn make_num(&mut self, n: i64) -> usize {
        let i = self.nodes.len();
        let node = Node::Num(n);
        self.nodes.push(node);
        i
    }

    fn make_func(&mut self, name: String, attrs: FuncAttr, children: Vec<usize>) -> usize {
        let i = self.nodes.len();
        let node = Node::Func(name, attrs, children);
        self.nodes.push(node);
        i
    }

    fn from_pexpr(&mut self, expr: &PExpr, symbols: &Symbols) -> usize {
        match expr {
            PExpr::Num(n) => self.make_num(*n),
            PExpr::Func(name, exprs) => {
                let attrs = symbols.get_function(name).unwrap();
                let children: Vec<_> = exprs.iter().map(|e| self.from_pexpr(e, symbols)).collect();
                self.make_func(name.clone(), *attrs, children)
            }
        }
    }

    fn print_expr(&self, node_index: usize, indent: usize) {
        print!("{}", " ".repeat(indent));
        let node = &self.nodes[node_index];
        match node {
            Node::Num(n) => println!("{}", n),
            Node::Func(name, _, children) => {
                println!("{}", name);
                for child_index in children {
                    self.print_expr(*child_index, indent + 4);
                }
            }
        }
    }

    fn cmp(&self, i1: usize, i2: usize) -> Ordering {
        let node1 = &self.nodes[i1];
        let node2 = &self.nodes[i2];
        match (node1, node2) {
            (Node::Num(i1), Node::Num(i2)) => i1.cmp(i2),
            (Node::Num(_), Node::Func(_, _, _)) => Ordering::Greater,
            (Node::Func(_, _, _), Node::Num(_)) => Ordering::Less,
            (Node::Func(n1, a1, c1), Node::Func(n2, a2, c2)) => {
                let c = n1.cmp(n2);
                if c != Ordering::Equal {
                    return c;
                }
                let c = a1.cmp(a2);
                if c != Ordering::Equal {
                    return c;
                }
                let c = c1.len().cmp(&c2.len());
                if c != Ordering::Equal {
                    return c;
                }
                for (ci1, ci2) in c1.iter().zip(c2.iter()) {
                    let c = self.cmp(*ci1, *ci2);
                    if c != Ordering::Equal {
                        return c;
                    }
                }
                Ordering::Equal
            }
        }
    }

    fn normalize(&mut self, node_index: usize) -> usize {
        let node = self.nodes[node_index].clone();
        match node {
            Node::Num(_) => node_index,
            Node::Func(name, attrs, children) => {
                let mut new_children: Vec<usize> = Vec::with_capacity(children.len());
                for child_index in children {
                    let new_child_index = self.normalize(child_index);
                    let child = &self.nodes[new_child_index];
                    match child {
                        Node::Func(child_name, _, child_children)
                            if attrs.is_associative() && name == *child_name =>
                        {
                            new_children.extend(child_children)
                        }
                        _ => new_children.push(new_child_index),
                    };
                }
                if attrs.is_commutative() {
                    new_children.sort_by(|a, b| self.cmp(*a, *b));
                }
                self.make_func(name, attrs, new_children)
            }
        }
    }
}

impl MainExpr {
    pub fn from_pexpr(expr: &PExpr, symbols: &Symbols) -> Self {
        let mut nodes = NodeContainer::new();
        let root = nodes.from_pexpr(expr, symbols);
        Self { nodes, root }
    }

    pub fn print_expr(&self) {
        self.nodes.print_expr(self.root, 0)
    }

    pub fn normalize(&mut self) {
        self.root = self.nodes.normalize(self.root);
    }
}
