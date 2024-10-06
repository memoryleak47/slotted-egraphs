use crate::*;

pub fn pvar(s: &str) -> Pattern<ArithENode> {
    Pattern {
        node: ENodeOrPVar::PVar(s.to_string()),
        children: vec![],
    }
}

pub fn app(l: Pattern<ArithENode>, r: Pattern<ArithENode>) -> Pattern<ArithENode> {
    Pattern {
        node: ENodeOrPVar::ENode(ArithENode::App(AppliedId::null(), AppliedId::null())),
        children: vec![l, r],
    }
}

pub fn var(s: usize) -> Pattern<ArithENode> {
    var_slot(Slot::new(s))
}

pub fn var_slot(s: Slot) -> Pattern<ArithENode> {
    Pattern {
        node: ENodeOrPVar::ENode(ArithENode::Var(s)),
        children: vec![],
    }
}

pub fn lam(s: usize, b: Pattern<ArithENode>) -> Pattern<ArithENode> {
    lam_slot(Slot::new(s), b)
}

pub fn lam_slot(s: Slot, b: Pattern<ArithENode>) -> Pattern<ArithENode> {
    Pattern {
        node: ENodeOrPVar::ENode(ArithENode::Lam(s, AppliedId::null())),
        children: vec![b],
    }
}

pub fn let_(s: usize, t: Pattern<ArithENode>, b: Pattern<ArithENode>) -> Pattern<ArithENode> {
    Pattern {
        node: ENodeOrPVar::ENode(ArithENode::Let(Slot::new(s), AppliedId::null(), AppliedId::null())),
        children: vec![t, b],
    }
}

pub fn add2(l: Pattern<ArithENode>, r: Pattern<ArithENode>) -> Pattern<ArithENode> {
    Pattern {
        node: ENodeOrPVar::ENode(ArithENode::Add(AppliedId::null(), AppliedId::null())),
        children: vec![l, r],
    }
}

pub fn mul2(l: Pattern<ArithENode>, r: Pattern<ArithENode>) -> Pattern<ArithENode> {
    Pattern {
        node: ENodeOrPVar::ENode(ArithENode::Mul(AppliedId::null(), AppliedId::null())),
        children: vec![l, r],
    }
}


pub fn num(i: u32) -> Pattern<ArithENode> {
    Pattern {
        node: ENodeOrPVar::ENode(ArithENode::Number(i)),
        children: vec![],
    }
}

pub fn symb(s: &str) -> Pattern<ArithENode> {
    Pattern {
        node: ENodeOrPVar::ENode(ArithENode::Symbol(Symbol::new(s))),
        children: vec![],
    }
}
