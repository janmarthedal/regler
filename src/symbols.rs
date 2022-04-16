use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FuncAttr {
    associative: bool,
    commutative: bool,
}

impl PartialOrd for FuncAttr {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FuncAttr {
    fn cmp(&self, other: &Self) -> Ordering {
        let a = 2 * (self.associative as i32) + (self.commutative as i32);
        let b = 2 * (other.associative as i32) + (other.commutative as i32);
        a.cmp(&b)
    }
}

impl FuncAttr {
    pub fn new(associative: bool, commutative: bool) -> Self {
        FuncAttr {
            associative,
            commutative,
        }
    }
    pub fn is_associative(&self) -> bool {
        self.associative
    }
    pub fn is_commutative(&self) -> bool {
        self.commutative
    }
}

pub struct Symbols {
    functions: HashMap<String, FuncAttr>,
}

impl Symbols {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }

    pub fn add_function(&mut self, name: String, attrs: FuncAttr) {
        self.functions.insert(name, attrs);
    }

    pub fn get_function(&self, name: &String) -> Option<&FuncAttr> {
        self.functions.get(name)
    }
}
