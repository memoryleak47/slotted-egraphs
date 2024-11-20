use crate::lambda::*;

pub struct LambdaRealSmall;

impl Realization for LambdaRealSmall {
    fn step(eg: &mut EGraph<Lambda>) {
        rewrite_small_step(eg);
    }
}

unpack_tests!(LambdaRealSmall);


pub fn rewrite_small_step(eg: &mut EGraph<Lambda>) {
    let mut future_unions = Vec::new();
    for cand in crate::lambda::big_step::candidates(eg) {
        let app_id = eg.lookup(&cand.app).unwrap();

        // L0 = Lambda::App(l, t).slots() -- "the root level"
        // t.slots(), l.slots(), app_id.slots() :: L0

        // L1 = Lambda::Lam(x, b).slots() = slots(l.id)

        let Lambda::App(l, t) = cand.app.clone() else { panic!() };
        let Lambda::Lam(Bind{slot: x, elem: b}) = cand.lam.clone() else { panic!() };
        assert_eq!(x, Slot::numeric(0));

        // b.m :: slots(b.id) -> L1
        // l.m :: slots(l.id) -> L0 (and thus L1 -> L0)

        // The L0-equivalent of x.
        let x_root = Slot::fresh();

        let mut l_m = l.m.clone();
        l_m.insert(x, x_root);
        let b = b.apply_slotmap(&l_m);

        assert!(eg.is_alive(b.id));
        for b_node in eg.enodes_applied(&b) {
            let new = step(x_root, t.clone(), &b_node, eg);
            future_unions.push((new, app_id.clone()));
        }
    }

    for (x, y) in future_unions {
        eg.union_justified(&x, &y, Some("beta-rewrite-small-step".to_string()));
    }
}

// everything here has L0 slot-names.
fn step(x: Slot, t: AppliedId, b: &Lambda, eg: &mut EGraph<Lambda>) -> AppliedId {
    if !b.slots().contains(&x) {
        return eg.lookup(b).unwrap();
    }

    match b {
        Lambda::Var(_) => t,
        Lambda::App(l, r) => {
            let mut pack = |lr: &AppliedId| {
                let a1 = eg.add(Lambda::Lam(Bind{slot:x, elem:lr.clone()}));
                let a2 = eg.add(Lambda::App(a1, t.clone()));
                a2
            };
            let l = pack(l);
            let r = pack(r);
            eg.add(Lambda::App(l, r))
        },
        Lambda::Lam(Bind{slot:y, elem:bb}) => {
            let a1 = eg.add(Lambda::Lam(Bind{slot:x, elem:bb.clone()}));
            let a2 = eg.add(Lambda::App(a1, t.clone()));
            let a3 = eg.add(Lambda::Lam(Bind{slot:*y, elem:a2}));
            a3
        },
        Lambda::Let(..) => panic!(),
    }
}
