use crate::*;

// candidate for beta reduction.
// Both ENodes are computed by "sh.apply_slotmap(bij)", where (sh, bij) in EClass::nodes from their respective classes.
pub struct Candidate {
    pub app: ENode,
    pub lam: ENode,
}

// applies rewrites (only beta-reduction) for all applicable situations.
pub fn rewrite_big_step(eg: &mut EGraph<ENode>) {
    for cand in candidates(eg) {
        let app_id = eg.lookup(&cand.app).unwrap();

        let ENode::App(l, t) = cand.app.clone() else { panic!() };
        let ENode::Lam(x, b) = cand.lam.clone() else { panic!() };
        assert_eq!(x, Slot::new(0));

        // l.m :: slots(lam) -> slots(app)
        let mut m = l.m.clone();

        // if x is a public slot of "app", we'd need to rename. But as x should always be s0 this shouldn't come up.
        assert!(!m.contains_key(x));

        m.insert(x, x);

        let b = b.apply_slotmap(&m);

        let new_id = subst(b, x, t, eg);
        eg.union_justified(&new_id, &app_id, Some("big-step-beta-reduction".to_string()));
    }
}

pub fn candidates(eg: &EGraph<ENode>) -> Vec<Candidate> {
    // find all lambdas:
    let mut lambdas: HashMap<Id, Vec<ENode>> = Default::default();
    for c in eg.ids() {
        let mut v = Vec::new();
        for enode in eg.enodes(c) {
            if matches!(enode, ENode::Lam(..)) {
                v.push(enode.clone());
            }
        }

        lambdas.insert(c, v);
    }

    // find apps:
    let mut candidates = Vec::new();

    for c in eg.ids() {
        for enode in eg.enodes(c) {
            if let ENode::App(l, _t) = &enode {
                for lam in lambdas[&l.id].clone() {
                    candidates.push(Candidate { app: enode.clone(), lam });
                }
            }
        }
    }

    candidates
}

pub struct LambdaRealBig(EGraph<ENode>);

impl Realization for LambdaRealBig {
    type Id = AppliedId;

    fn new() -> Self {
        LambdaRealBig(EGraph::new())
    }

    fn add_ast(&mut self, ast: &Ast) -> Self::Id {
        let re = RecExpr::<ENode>::parse2(&ast.to_string());
        self.0.add_expr(re)
    }

    fn extract_ast(&self, id: Self::Id) -> Ast {
        let out = extract::<ENode, AstSize>(id, &self.0);
        Ast::parse(&out.to_string2())
    }

    fn find(&self, id: Self::Id) -> Self::Id {
        self.0.find_applied_id(&id)
    }

    fn step(&mut self) {
        rewrite_big_step(&mut self.0);
    }

    fn enode_count(&self) -> usize { self.0.total_number_of_nodes() }
    fn eclass_count(&self) -> usize { self.0.ids().len() } 

    fn explain_equivalence(&mut self, ast1: Ast, ast2: Ast) {
        let re1 = RecExpr::<ENode>::parse2(&ast1.to_string());
        let re2 = RecExpr::<ENode>::parse2(&ast2.to_string());
        self.0.explain_equivalence(re1, re2).show_expr(&self.0);
    }
}

// TODO re-enable.
// unpack_tests!(LambdaRealBig);
