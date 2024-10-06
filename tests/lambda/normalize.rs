use crate::lambda::*;

pub fn lam_normalize(re: &RecExpr<LetENode>) -> RecExpr<LetENode> {
    lam_normalize_impl(re, &mut 0, Default::default())
}


// map :: original name -> normalized name.
fn lam_normalize_impl(re: &RecExpr<LetENode>, counter: &mut usize, map: HashMap<Slot, Slot>) -> RecExpr<LetENode> {
    let mut alloc_slot = || {
        let out = Slot::new(*counter);
        *counter += 1;
        out
    };

    match &re.node {
        LetENode::Lam(x, _) => {
            let [b] = &*re.children else { panic!() };

            let mut map = map.clone();
            let norm_x = alloc_slot();
            map.insert(x.clone(), norm_x.clone());

            let b = lam_normalize_impl(b, counter, map);

            RecExpr {
                node: LetENode::Lam(norm_x, AppliedId::null()),
                children: vec![b],
            }
        },
        LetENode::App(_, _) => {
            let [l, r] = &*re.children else { panic!() };

            let l = lam_normalize_impl(l, counter, map.clone());
            let r = lam_normalize_impl(r, counter, map.clone());

            RecExpr {
                node: LetENode::App(AppliedId::null(), AppliedId::null()),
                children: vec![l, r],
            }
        },
        LetENode::Var(x) => {
            let norm_x = map[x];

            RecExpr {
                node: LetENode::Var(norm_x),
                children: vec![],
            }
        },
        LetENode::Let(..) => panic!(),
    }
}
