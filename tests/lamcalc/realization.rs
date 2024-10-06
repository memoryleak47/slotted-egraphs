use crate::lamcalc::*;

const NO_ITERS: usize = 400;
const NO_ENODES: usize = 10000;

pub trait Realization: Sized {
    type Id: Clone + Eq;

    fn new() -> Self;
    fn add_ast(&mut self, ast: &Ast) -> Self::Id;
    fn extract_ast(&self, id: Self::Id) -> Ast;
    fn find(&self, id: Self::Id) -> Self::Id;
    fn step(&mut self);
    fn enode_count(&self) -> usize;
    fn eclass_count(&self) -> usize;
    fn explain_equivalence(&mut self, ast: Ast, ast2: Ast);
}

// stops when the desired output has been reached.
pub fn simplify_to_nf<R: Realization>(s: &str) -> String {
    let orig_ast = Ast::parse(s);
    let mut ast = orig_ast.clone();
    let mut eg = R::new();
    let i = eg.add_ast(&ast);
    for _ in 0..NO_ITERS {
        eg.step();

        ast = eg.extract_ast(i.clone());
        if ast.step().is_none() {
            eg.explain_equivalence(orig_ast, ast.clone());
            return ast.to_string();
        };

        if eg.enode_count() > NO_ENODES {
            break;
        }
    }
    panic!("failed to reach NF! Or the beta-NF is just AstSize-suboptimal!");
}


pub fn simplify<R: Realization>(s: &str) -> String {
    let ast = Ast::parse(s);
    let mut eg = R::new();
    let i = eg.add_ast(&ast);
    for _ in 0..NO_ITERS {
        eg.step();
        if eg.enode_count() > NO_ENODES {
            break;
        }
    }
    let out = eg.extract_ast(i.clone());

    eg.explain_equivalence(ast.clone(), out.clone());

    let out = out.to_string();

    out
}

// TODO the smallest term isn't necessarily the beta-NF.
pub fn check_simplify<R: Realization>(p: &str) {
    let out1 = simplify::<R>(p);
    let out2 = run(p);
    assert_alpha_eq(&*out1, &*out2);
}

pub fn check_simplify_to_nf<R: Realization>(p: &str) {
    let out1 = simplify_to_nf::<R>(p);
    let out2 = run(p);
    assert_alpha_eq(&*out1, &*out2);
}

// checks whether simplify has valid output, even though it might not be able to finish the whole computation.
pub fn check_simplify_incomplete<R: Realization>(p: &str) {
    let out1 = run(&simplify::<R>(p));
    let out2 = run(p);
    assert_alpha_eq(&*out1, &*out2);
}

pub fn check_eq<R: Realization>(s1: &str, s2: &str) {
    let s1 = Ast::parse(s1);
    let s2 = Ast::parse(s2);
    let mut eg = R::new();
    let i1 = eg.add_ast(&s1);
    let i2 = eg.add_ast(&s2);
    for _ in 0..NO_ITERS {
        if eg.find(i1.clone()) == eg.find(i2.clone()) {
            eg.explain_equivalence(s1.clone(), s2.clone());
            return;
        }

        eg.step();

        if eg.enode_count() > NO_ENODES {
            break;
        }
    }
    panic!("equality could not be found!");
}

// Non-Realization functions:

pub fn norm(s: &str) -> String {
    Ast::parse(s).normalize().to_string()
}

pub fn run(s: &str) -> String {
    Ast::parse(s).run().normalize().to_string()
}

pub fn assert_alpha_eq(s1: &str, s2: &str) {
    assert_eq!(norm(s1), norm(s2));
}

pub fn assert_run_eq(s1: &str, s2: &str) {
    assert_eq!(run(s1), run(s2));
}
