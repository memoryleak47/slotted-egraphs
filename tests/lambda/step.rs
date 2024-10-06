use crate::lambda::*;

pub fn lam_run(re: &RecExpr<LetENode>) -> RecExpr<LetENode> {
    let mut re = re.clone();
    while let Some(x) = lam_step(&re) {
        re = x;
    }

    re
}

pub fn lam_run_n(re: &RecExpr<LetENode>, n: u32) -> RecExpr<LetENode> {
    let mut re = re.clone();
    for _ in 0..n {
        if let Some(x) = lam_step(&re) {
            re = x;
        }
    }

    re
}

pub fn lam_step(re: &RecExpr<LetENode>) -> Option<RecExpr<LetENode>> {
    match &re.node {
        LetENode::Lam(x, _) => {
            let [b] = &*re.children else { panic!() };
            let b = lam_step(b)?;

            Some(RecExpr {
                node: re.node.clone(),
                children: vec![b],
            })
        },
        LetENode::App(_, _) => {
            let [l, r] = &*re.children else { panic!() };

            // beta-reduce
            if let LetENode::Lam(x, _) = &l.node {
                let [b] = &*l.children else { panic!() };
                return Some(lam_subst(b, *x, r));
            }

            // l-recurse
            if let Some(l_new) = lam_step(l) {
                return Some(RecExpr {
                    node: re.node.clone(),
                    children: vec![l_new, r.clone()],
                });
            }

            // r-recurse
            if let Some(r_new) = lam_step(r) {
                return Some(RecExpr {
                    node: re.node.clone(),
                    children: vec![l.clone(), r_new],
                });
            }

            None
        },
        LetENode::Var(_) => None,
        LetENode::Let(..) => panic!(),
    }
}

fn lam_subst(re: &RecExpr<LetENode>, x: Slot, t: &RecExpr<LetENode>) -> RecExpr<LetENode> {
    match &re.node {
        LetENode::Lam(y, _) => {
            let [b] = &*re.children else { panic!() };
            if x == *y {
                re.clone()
            } else {
                let f = lam_free_variables(t);

                let mut y: Slot = *y;
                let mut b: RecExpr<LetENode> = b.clone();

                if f.contains(&y) {
                    let y2 = (0..).map(|i| Slot::new(i))
                                  .filter(|i| !f.contains(i))
                                  .next()
                                  .unwrap();

                    let y2_node = RecExpr {
                        node: LetENode::Var(y2),
                        children: vec![],
                    };

                    b = lam_subst(&b, y, &y2_node);
                    y = y2;
                }
                RecExpr {
                    node: re.node.clone(),
                    children: vec![lam_subst(&b, x, t)],
                }
            }
        },
        LetENode::App(_, _) => {
            let [l, r] = &*re.children else { panic!() };
            RecExpr {
                node: re.node.clone(),
                children: vec![lam_subst(l, x, t), lam_subst(r, x, t)],
            }
        },
        LetENode::Var(y) => {
            if x == *y {
                t.clone()
            } else {
                re.clone()
            }
        }
        LetENode::Let(..) => panic!(),
    }
}

fn lam_free_variables(re: &RecExpr<LetENode>) -> HashSet<Slot> {
    match &re.node {
        LetENode::Lam(x, _) => {
            let [b] = &*re.children else { panic!() };
            &lam_free_variables(b) - &singleton_set(*x)
        }
        LetENode::App(_, _) => {
            let [l, r] = &*re.children else { panic!() };
            &lam_free_variables(l) | &lam_free_variables(r)
        },
        LetENode::Var(x) => singleton_set(*x),
        LetENode::Let(..) => panic!(),
    }
}
