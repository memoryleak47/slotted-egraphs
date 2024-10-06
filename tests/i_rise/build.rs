use crate::*;

// advanced functions:
pub fn map0() -> Pattern<RiseENode> { symb("map") }
pub fn map1(x: Pattern<RiseENode>) -> Pattern<RiseENode> { app(map0(), x) }
pub fn map2(x: Pattern<RiseENode>, y: Pattern<RiseENode>) -> Pattern<RiseENode> { app(map1(x), y) }
pub fn map3(x: Pattern<RiseENode>, y: Pattern<RiseENode>, z: Pattern<RiseENode>) -> Pattern<RiseENode> { app(map2(x, y), z) }

pub fn transpose0() -> Pattern<RiseENode> { symb("transpose") }
pub fn transpose1(x: Pattern<RiseENode>) -> Pattern<RiseENode> { app(transpose0(), x) }

pub fn slide0() -> Pattern<RiseENode> { symb("slide") }
pub fn slide1(x: Pattern<RiseENode>) -> Pattern<RiseENode> { app(slide0(), x) }
pub fn slide2(x: Pattern<RiseENode>, y: Pattern<RiseENode>) -> Pattern<RiseENode> { app(slide1(x), y) }
pub fn slide3(x: Pattern<RiseENode>, y: Pattern<RiseENode>, z: Pattern<RiseENode>) -> Pattern<RiseENode> { app(slide2(x, y), z) }

pub fn fst0() -> Pattern<RiseENode> { symb("fst") }
pub fn fst1(x: Pattern<RiseENode>) -> Pattern<RiseENode> { app(fst0(), x) }

pub fn snd0() -> Pattern<RiseENode> { symb("snd") }
pub fn snd1(x: Pattern<RiseENode>) -> Pattern<RiseENode> { app(snd0(), x) }

pub fn reduce0() -> Pattern<RiseENode> { symb("reduce") }
pub fn reduce1(x: Pattern<RiseENode>) -> Pattern<RiseENode> { app(reduce0(), x) }
pub fn reduce2(x: Pattern<RiseENode>, y: Pattern<RiseENode>) -> Pattern<RiseENode> { app(reduce1(x), y) }
pub fn reduce3(x: Pattern<RiseENode>, y: Pattern<RiseENode>, z: Pattern<RiseENode>) -> Pattern<RiseENode> { app(reduce2(x, y), z) }

pub fn zip0() -> Pattern<RiseENode> { symb("zip") }
pub fn zip1(x: Pattern<RiseENode>) -> Pattern<RiseENode> { app(zip0(), x) }
pub fn zip2(x: Pattern<RiseENode>, y: Pattern<RiseENode>) -> Pattern<RiseENode> { app(zip1(x), y) }
pub fn zip3(x: Pattern<RiseENode>, y: Pattern<RiseENode>, z: Pattern<RiseENode>) -> Pattern<RiseENode> { app(zip2(x, y), z) }

pub fn join0() -> Pattern<RiseENode> { symb("join") }
pub fn join1(x: Pattern<RiseENode>) -> Pattern<RiseENode> { app(join0(), x) }

pub fn add0() -> Pattern<RiseENode> { symb("add") }
pub fn add1(x: Pattern<RiseENode>) -> Pattern<RiseENode> { app(add0(), x) }
pub fn add2(x: Pattern<RiseENode>, y: Pattern<RiseENode>) -> Pattern<RiseENode> { app(add1(x), y) }

pub fn mul0() -> Pattern<RiseENode> { symb("mul") }
pub fn mul1(x: Pattern<RiseENode>) -> Pattern<RiseENode> { app(mul0(), x) }
pub fn mul2(x: Pattern<RiseENode>, y: Pattern<RiseENode>) -> Pattern<RiseENode> { app(mul1(x), y) }

pub fn dot2(a: Pattern<RiseENode>, b: Pattern<RiseENode>) -> Pattern<RiseENode> {
    // dot uses pipe! See egg-rise/main.rs:316

    let x = Slot::fresh();

    let t1 = zip2(a, b);
    let t2 = map1(lam_slot(x,
        mul2(
                fst1(var_slot(x)),
                snd1(var_slot(x))
        )
    ));
    let t3 = reduce2(add0(), num(0));

    pipe(pipe(t1, t2), t3)
}

// f >> g
pub fn then(f: Pattern<RiseENode>, g: Pattern<RiseENode>) -> Pattern<RiseENode> {
    let x = Slot::fresh();
    lam_slot(x, app(g, app(f, var_slot(x))))
}

// x |> f
pub fn pipe(x: Pattern<RiseENode>, f: Pattern<RiseENode>) -> Pattern<RiseENode> {
    app(f, x)
}

// base functions:

pub fn pvar(s: &str) -> Pattern<RiseENode> {
    Pattern {
        node: ENodeOrPVar::PVar(s.to_string()),
        children: vec![],
    }
}

pub fn app(l: Pattern<RiseENode>, r: Pattern<RiseENode>) -> Pattern<RiseENode> {
    Pattern {
        node: ENodeOrPVar::ENode(RiseENode::App(AppliedId::null(), AppliedId::null())),
        children: vec![l, r],
    }
}

pub fn var(s: usize) -> Pattern<RiseENode> {
    var_slot(Slot::new(s))
}

pub fn var_slot(s: Slot) -> Pattern<RiseENode> {
    Pattern {
        node: ENodeOrPVar::ENode(RiseENode::Var(s)),
        children: vec![],
    }
}


pub fn lam(s: usize, b: Pattern<RiseENode>) -> Pattern<RiseENode> {
    lam_slot(Slot::new(s), b)
}

pub fn lam_slot(s: Slot, b: Pattern<RiseENode>) -> Pattern<RiseENode> {
    Pattern {
        node: ENodeOrPVar::ENode(RiseENode::Lam(s, AppliedId::null())),
        children: vec![b],
    }
}

pub fn let_(s: usize, t: Pattern<RiseENode>, b: Pattern<RiseENode>) -> Pattern<RiseENode> {
    Pattern {
        node: ENodeOrPVar::ENode(RiseENode::Let(Slot::new(s), AppliedId::null(), AppliedId::null())),
        children: vec![t, b],
    }
}

pub fn num(i: u32) -> Pattern<RiseENode> {
    Pattern {
        node: ENodeOrPVar::ENode(RiseENode::Number(i)),
        children: vec![],
    }
}

pub fn symb(s: &str) -> Pattern<RiseENode> {
    Pattern {
        node: ENodeOrPVar::ENode(RiseENode::Symbol(Symbol::new(s))),
        children: vec![],
    }
}
