use crate::*;

pub struct LambdaRealBig;

impl Realization for LambdaRealBig {
    fn step(eg: &mut EGraph<Lambda>) {
        rewrite_big_step(eg);
    }
}

// TODO re-add.
// unpack_tests!(LambdaRealBig);


// candidate for beta reduction.
// Both Lambdas are computed by "sh.apply_slotmap(bij)", where (sh, bij) in EClass::nodes from their respective classes.
pub struct Candidate {
    pub app: Lambda,
    pub lam: Lambda,
}

// applies rewrites (only beta-reduction) for all applicable situations.
pub fn rewrite_big_step(eg: &mut EGraph<Lambda>) {
    for cand in candidates(eg) {
        let app_id = eg.lookup(&cand.app).unwrap();

        let Lambda::App(l, t) = cand.app.clone() else { panic!() };
        let Lambda::Lam(x, b) = cand.lam.clone() else { panic!() };
        assert_eq!(x, Slot::numeric(0));

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

pub fn candidates(eg: &EGraph<Lambda>) -> Vec<Candidate> {
    // find all lambdas:
    let mut lambdas: HashMap<Id, Vec<Lambda>> = Default::default();
    for c in eg.ids() {
        let mut v = Vec::new();
        assert!(eg.is_alive(c));
        for enode in eg.enodes(c) {
            if matches!(enode, Lambda::Lam(..)) {
                v.push(enode.clone());
            }
        }

        lambdas.insert(c, v);
    }

    // find apps:
    let mut candidates = Vec::new();

    for c in eg.ids() {
        for enode in eg.enodes(c) {
            if let Lambda::App(l, _t) = &enode {
                for lam in lambdas[&l.id].clone() {
                    candidates.push(Candidate { app: enode.clone(), lam });
                }
            }
        }
    }

    candidates
}
