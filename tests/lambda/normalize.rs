use crate::lambda::*;

pub fn lam_normalize(re: &RecExpr<Lambda>) -> RecExpr<Lambda> {
    lam_normalize_impl(re, &mut 0, Default::default())
}


// map :: original name -> normalized name.
fn lam_normalize_impl(re: &RecExpr<Lambda>, counter: &mut usize, map: HashMap<Slot, Slot>) -> RecExpr<Lambda> {
    let mut alloc_slot = || {
        let out = Slot::numeric(*counter as _);
        *counter += 1;
        out
    };

    match &re.node {
        Lambda::Lam(x, _) => {
            let [b] = &*re.children else { panic!() };

            let mut map = map.clone();
            let norm_x = alloc_slot();
            map.insert(x.clone(), norm_x.clone());

            let b = lam_normalize_impl(b, counter, map);

            RecExpr {
                node: Lambda::Lam(norm_x, AppliedId::null()),
                children: vec![b],
            }
        },
        Lambda::App(_, _) => {
            let [l, r] = &*re.children else { panic!() };

            let l = lam_normalize_impl(l, counter, map.clone());
            let r = lam_normalize_impl(r, counter, map.clone());

            RecExpr {
                node: Lambda::App(AppliedId::null(), AppliedId::null()),
                children: vec![l, r],
            }
        },
        Lambda::Var(x) => {
            let norm_x = map[x];

            RecExpr {
                node: Lambda::Var(norm_x),
                children: vec![],
            }
        },
        Lambda::Let(..) => panic!(),
    }
}
