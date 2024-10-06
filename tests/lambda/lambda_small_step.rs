use crate::lambda::*;

pub struct LambdaRealSmall;

impl Realization for LambdaRealSmall {
    fn step(eg: &mut EGraph<LetENode>) {
        rewrite_small_step(eg);
    }
}

unpack_tests!(LambdaRealSmall);


pub fn rewrite_small_step(eg: &mut EGraph<LetENode>) {
    for cand in crate::lambda::big_step::candidates(eg) {
        let app_id = eg.lookup(&cand.app).unwrap();

        // L0 = LetENode::App(l, t).slots() -- "the root level"
        // t.slots(), l.slots(), app_id.slots() :: L0

        // L1 = LetENode::Lam(x, b).slots() = slots(l.id)

        let LetENode::App(l, t) = cand.app.clone() else { panic!() };
        let LetENode::Lam(x, b) = cand.lam.clone() else { panic!() };
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
            eg.union_justified(&new, &app_id, Some("beta-rewrite-small-step".to_string()));
        }
    }
}

// everything here has L0 slot-names.
fn step(x: Slot, t: AppliedId, b: &LetENode, eg: &mut EGraph<LetENode>) -> AppliedId {
    if !b.slots().contains(&x) {
        return eg.lookup(b).unwrap();
    }

    match b {
        LetENode::Var(_) => t,
        LetENode::App(l, r) => {
            let mut pack = |lr: &AppliedId| {
                let a1 = eg.add(LetENode::Lam(x, lr.clone()));
                let a2 = eg.add(LetENode::App(a1, t.clone()));
                a2
            };
            let l = pack(l);
            let r = pack(r);
            eg.add(LetENode::App(l, r))
        },
        LetENode::Lam(y, bb) => {
            let a1 = eg.add(LetENode::Lam(x, bb.clone()));
            let a2 = eg.add(LetENode::App(a1, t.clone()));
            let a3 = eg.add(LetENode::Lam(*y, a2));
            a3
        },
        LetENode::Let(..) => panic!(),
    }
}
