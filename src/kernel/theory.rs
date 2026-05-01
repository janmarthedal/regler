//! Theory state assembled from facts: rewrite rules earned by KBO orientation,
//! plus the AC and identity-element marks that are promoted dynamically as
//! commutativity, associativity, and identity-shape facts arrive.
//!
//! See the design notes in `CLAUDE.md`:
//! - "AC marking is earned dynamically" — a function `f` becomes AC once both a
//!   commutativity-shape fact and an associativity-shape fact for `f` have
//!   been seen.
//! - "Identity-element marking is earned similarly" — `f(x, e) = x` registers
//!   `e` as a right identity for `f`; `f(e, x) = x` registers it as a left
//!   identity. For AC operators the two coincide, so a single fact covers both
//!   sides.

use std::collections::{HashMap, HashSet};

use crate::kernel::kbo::{kbo, KboOrd};
use crate::kernel::term::{Symbol, Term};

/// A rewrite rule oriented by KBO: `lhs` strictly dominates `rhs`, so every
/// rewrite step strictly decreases the term in the order. Variables in `lhs`
/// act as pattern variables; KBO orientation guarantees every variable in
/// `rhs` also appears in `lhs`.
#[derive(Debug, Clone)]
pub struct Rule {
    pub lhs: Term,
    pub rhs: Term,
}

/// Outcome of trying to install an equality `l = r` as a rewrite rule.
#[derive(Debug)]
pub enum Orient {
    Rule(Rule),
    Trivial,
    Incomparable,
}

/// Attempt to orient an equality into a rewrite rule by KBO. The larger side
/// becomes the lhs. Equalities whose two sides are KBO-incomparable cannot be
/// auto-applied and are reported as such; trivial equalities (`l = l`) are
/// reported separately.
pub fn orient(l: &Term, r: &Term) -> Orient {
    match kbo(l, r) {
        KboOrd::Gt => Orient::Rule(Rule {
            lhs: l.clone(),
            rhs: r.clone(),
        }),
        KboOrd::Lt => Orient::Rule(Rule {
            lhs: r.clone(),
            rhs: l.clone(),
        }),
        KboOrd::Eq => Orient::Trivial,
        KboOrd::Incomparable => Orient::Incomparable,
    }
}

/// Outcome(s) produced by installing a single fact. One fact may trigger more
/// than one effect (e.g. installing commutativity may also promote `f` to AC if
/// associativity was already seen).
#[derive(Debug)]
pub enum FactEffect {
    NotEquality,
    Trivial,
    Incomparable,
    RuleInstalled,
    Commutativity(Symbol),
    Associativity(Symbol),
    LeftIdentity(Symbol, Term),
    RightIdentity(Symbol, Term),
    AcPromoted(Symbol),
    AlreadyKnown,
}

#[derive(Debug, Default)]
pub struct Theory {
    pub rules: Vec<Rule>,
    ac: HashSet<Symbol>,
    saw_comm: HashSet<Symbol>,
    saw_assoc: HashSet<Symbol>,
    left_id: HashMap<Symbol, Term>,
    right_id: HashMap<Symbol, Term>,
}

impl Theory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_ac(&self, f: &Symbol) -> bool {
        self.ac.contains(f)
    }

    pub fn left_identity(&self, f: &Symbol) -> Option<&Term> {
        self.left_id.get(f)
    }

    pub fn right_identity(&self, f: &Symbol) -> Option<&Term> {
        self.right_id.get(f)
    }

    /// Inspect `t`, recognise commutativity / associativity / identity shapes,
    /// and otherwise hand the equality off to KBO orientation. Returns every
    /// effect the install produced so the REPL can report them.
    pub fn install_fact(&mut self, t: &Term) -> Vec<FactEffect> {
        let (l, r) = match t {
            Term::App(head, args) if head.as_ref() == "=" && args.len() == 2 => {
                (&args[0], &args[1])
            }
            _ => return vec![FactEffect::NotEquality],
        };

        if let Some(f) = match_commutativity(l, r) {
            return self.note_commutativity(f);
        }
        if let Some(f) = match_associativity(l, r) {
            return self.note_associativity(f);
        }
        if let Some((f, e)) = match_right_identity(l, r) {
            return self.note_right_identity(f, e);
        }
        if let Some((f, e)) = match_left_identity(l, r) {
            return self.note_left_identity(f, e);
        }

        match orient(l, r) {
            Orient::Rule(rule) => {
                self.rules.push(rule);
                vec![FactEffect::RuleInstalled]
            }
            Orient::Trivial => vec![FactEffect::Trivial],
            Orient::Incomparable => vec![FactEffect::Incomparable],
        }
    }

    fn note_commutativity(&mut self, f: Symbol) -> Vec<FactEffect> {
        if !self.saw_comm.insert(f.clone()) {
            return vec![FactEffect::AlreadyKnown];
        }
        let mut out = vec![FactEffect::Commutativity(f.clone())];
        if self.saw_assoc.contains(&f) && self.ac.insert(f.clone()) {
            self.merge_identities_after_ac(&f);
            out.push(FactEffect::AcPromoted(f));
        }
        out
    }

    fn note_associativity(&mut self, f: Symbol) -> Vec<FactEffect> {
        if !self.saw_assoc.insert(f.clone()) {
            return vec![FactEffect::AlreadyKnown];
        }
        let mut out = vec![FactEffect::Associativity(f.clone())];
        if self.saw_comm.contains(&f) && self.ac.insert(f.clone()) {
            self.merge_identities_after_ac(&f);
            out.push(FactEffect::AcPromoted(f));
        }
        out
    }

    fn note_right_identity(&mut self, f: Symbol, e: Term) -> Vec<FactEffect> {
        let prior = self.right_id.insert(f.clone(), e.clone());
        if prior.as_ref() == Some(&e) {
            return vec![FactEffect::AlreadyKnown];
        }
        if self.is_ac(&f) {
            self.left_id.insert(f.clone(), e.clone());
        }
        vec![FactEffect::RightIdentity(f, e)]
    }

    fn note_left_identity(&mut self, f: Symbol, e: Term) -> Vec<FactEffect> {
        let prior = self.left_id.insert(f.clone(), e.clone());
        if prior.as_ref() == Some(&e) {
            return vec![FactEffect::AlreadyKnown];
        }
        if self.is_ac(&f) {
            self.right_id.insert(f.clone(), e.clone());
        }
        vec![FactEffect::LeftIdentity(f, e)]
    }

    /// Once `f` becomes AC, left and right identity coincide; copy whichever
    /// side has been seen into the other so `simplify` can absorb either.
    fn merge_identities_after_ac(&mut self, f: &Symbol) {
        if let Some(e) = self.left_id.get(f).cloned() {
            self.right_id.entry(f.clone()).or_insert(e);
        }
        if let Some(e) = self.right_id.get(f).cloned() {
            self.left_id.entry(f.clone()).or_insert(e);
        }
    }
}

/// Detect `f(a, b) = f(b, a)` with two distinct pattern variables.
fn match_commutativity(l: &Term, r: &Term) -> Option<Symbol> {
    let (f, la, lb) = bin_app_of_two_vars(l)?;
    let (g, ra, rb) = bin_app_of_two_vars(r)?;
    if f != g || la == lb {
        return None;
    }
    if la == rb && lb == ra {
        Some(f.clone())
    } else {
        None
    }
}

/// Detect either `f(f(a, b), c) = f(a, f(b, c))` or its mirror, with three
/// distinct pattern variables.
fn match_associativity(l: &Term, r: &Term) -> Option<Symbol> {
    if let Some(f) = assoc_left_to_right(l, r) {
        return Some(f);
    }
    assoc_left_to_right(r, l)
}

fn assoc_left_to_right(l: &Term, r: &Term) -> Option<Symbol> {
    let (f, lhs1, c) = bin_app(l)?;
    let (f2, a1, b1) = bin_app(lhs1)?;
    if f != f2 {
        return None;
    }
    let (f3, a2, rhs2) = bin_app(r)?;
    if f != f3 {
        return None;
    }
    let (f4, b2, c2) = bin_app(rhs2)?;
    if f != f4 {
        return None;
    }
    let a = as_var(a1)?;
    let b = as_var(b1)?;
    let cc = as_var(c)?;
    if a == b || b == cc || a == cc {
        return None;
    }
    if as_var(a2)? == a && as_var(b2)? == b && as_var(c2)? == cc {
        Some(f.clone())
    } else {
        None
    }
}

/// Detect `f(x, e) = x` where `x` is a pattern variable and `e` is a closed
/// (variable-free) term.
fn match_right_identity(l: &Term, r: &Term) -> Option<(Symbol, Term)> {
    let (f, a, b) = bin_app(l)?;
    let x = as_var(a)?;
    let rx = as_var(r)?;
    if x != rx || !is_closed(b) {
        return None;
    }
    Some((f.clone(), b.clone()))
}

/// Detect `f(e, x) = x`.
fn match_left_identity(l: &Term, r: &Term) -> Option<(Symbol, Term)> {
    let (f, a, b) = bin_app(l)?;
    let x = as_var(b)?;
    let rx = as_var(r)?;
    if x != rx || !is_closed(a) {
        return None;
    }
    Some((f.clone(), a.clone()))
}

fn bin_app(t: &Term) -> Option<(&Symbol, &Term, &Term)> {
    match t {
        Term::App(f, args) if args.len() == 2 => Some((f, &args[0], &args[1])),
        _ => None,
    }
}

fn bin_app_of_two_vars(t: &Term) -> Option<(&Symbol, &Symbol, &Symbol)> {
    let (f, a, b) = bin_app(t)?;
    Some((f, as_var(a)?, as_var(b)?))
}

fn as_var(t: &Term) -> Option<&Symbol> {
    match t {
        Term::Var(s) => Some(s),
        _ => None,
    }
}

fn is_closed(t: &Term) -> bool {
    match t {
        Term::Nat(_) => true,
        Term::Var(_) => false,
        Term::App(_, args) => args.iter().all(is_closed),
    }
}
