use crate::*;

pub fn rewrite_small_step(eg: &mut EGraph<ENode>) {
    for cand in candidates(eg) {
        let app_id = eg.lookup(&cand.app).unwrap();

        // L0 = ENode::App(l, t).slots() -- "the root level"
        // t.slots(), l.slots(), app_id.slots() :: L0

        // L1 = ENode::Lam(x, b).slots() = slots(l.id)

        let ENode::App(l, t) = cand.app.clone() else { panic!() };
        let ENode::Lam(x, b) = cand.lam.clone() else { panic!() };
        assert_eq!(x, Slot::new(0));

        // b.m :: slots(b.id) -> L1
        // l.m :: slots(l.id) -> L0 (and thus L1 -> L0)

        // The L0-equivalent of x.
        let x_root = Slot::fresh();

        let mut l_m = l.m.clone();
        l_m.insert(x, x_root);
        let b = b.apply_slotmap(&l_m);

        for b_node in eg.enodes_applied(&b) {
            let new = step(x_root, t.clone(), &b_node, eg);
            eg.union(&new, &app_id);
        }
    }
}

// everything here has L0 slot-names.
fn step(x: Slot, t: AppliedId, b: &ENode, eg: &mut EGraph<ENode>) -> AppliedId {
    if !b.slots().contains(&x) {
        return eg.lookup(b).unwrap();
    }

    match b {
        ENode::Var(_) => t,
        ENode::App(l, r) => {
            let mut pack = |lr: &AppliedId| {
                let a1 = eg.add(ENode::Lam(x, lr.clone()));
                let a2 = eg.add(ENode::App(a1, t.clone()));
                a2
            };
            let l = pack(l);
            let r = pack(r);
            eg.add(ENode::App(l, r))
        },
        ENode::Lam(y, bb) => {
            let a1 = eg.add(ENode::Lam(x, bb.clone()));
            let a2 = eg.add(ENode::App(a1, t.clone()));
            let a3 = eg.add(ENode::Lam(*y, a2));
            a3
        },
    }
}

pub struct LambdaRealSmall(EGraph<ENode>);

impl Realization for LambdaRealSmall {
    type Id = AppliedId;

    fn new() -> Self {
        LambdaRealSmall(EGraph::new())
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
        rewrite_small_step(&mut self.0);
    }

    fn enode_count(&self) -> usize { self.0.total_number_of_nodes() }
    fn eclass_count(&self) -> usize { self.0.ids().len() } 

    fn explain_equivalence(&mut self, ast1: Ast, ast2: Ast) {
        let re1 = RecExpr::<ENode>::parse2(&ast1.to_string());
        let re2 = RecExpr::<ENode>::parse2(&ast2.to_string());
        self.0.explain_equivalence(re1, re2).show_expr(&self.0);
    }
}

unpack_tests!(LambdaRealSmall);
