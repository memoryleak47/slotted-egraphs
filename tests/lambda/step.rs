use crate::lambda::*;

pub fn lam_run(re: &RecExpr<Lambda>) -> RecExpr<Lambda> {
    let mut re = re.clone();
    while let Some(x) = lam_step(&re) {
        re = x;
    }

    re
}

pub fn lam_run_n(re: &RecExpr<Lambda>, n: u32) -> RecExpr<Lambda> {
    let mut re = re.clone();
    for _ in 0..n {
        if let Some(x) = lam_step(&re) {
            re = x;
        }
    }

    re
}

pub fn lam_step(re: &RecExpr<Lambda>) -> Option<RecExpr<Lambda>> {
    match &re.node {
        Lambda::Lam(_) => {
            let [b] = &*re.children else { panic!() };
            let b = lam_step(b)?;

            Some(RecExpr {
                node: re.node.clone(),
                children: vec![b],
            })
        },
        Lambda::App(_, _) => {
            let [l, r] = &*re.children else { panic!() };

            // beta-reduce
            if let Lambda::Lam(Bind{slot:x, ..}) = &l.node {
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
        Lambda::Var(_) => None,
        Lambda::Let(..) => panic!(),
    }
}

fn lam_subst(re: &RecExpr<Lambda>, x: Slot, t: &RecExpr<Lambda>) -> RecExpr<Lambda> {
    match &re.node {
        Lambda::Lam(Bind{slot:y, ..}) => {
            let [b] = &*re.children else { panic!() };
            if x == *y {
                re.clone()
            } else {
                let f = lam_free_variables(t);

                let mut y: Slot = *y;
                let mut b: RecExpr<Lambda> = b.clone();

                if f.contains(&y) {
                    let y2 = (0..).map(|i| Slot::numeric(i))
                                  .filter(|i| !f.contains(i))
                                  .next()
                                  .unwrap();

                    let y2_node = RecExpr {
                        node: Lambda::Var(y2),
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
        Lambda::App(_, _) => {
            let [l, r] = &*re.children else { panic!() };
            RecExpr {
                node: re.node.clone(),
                children: vec![lam_subst(l, x, t), lam_subst(r, x, t)],
            }
        },
        Lambda::Var(y) => {
            if x == *y {
                t.clone()
            } else {
                re.clone()
            }
        }
        Lambda::Let(..) => panic!(),
    }
}

fn lam_free_variables(re: &RecExpr<Lambda>) -> HashSet<Slot> {
    match &re.node {
        Lambda::Lam(Bind{slot:x, ..}) => {
            let [b] = &*re.children else { panic!() };
            &lam_free_variables(b) - &singleton_set(*x)
        }
        Lambda::App(_, _) => {
            let [l, r] = &*re.children else { panic!() };
            &lam_free_variables(l) | &lam_free_variables(r)
        },
        Lambda::Var(x) => singleton_set(*x),
        Lambda::Let(..) => panic!(),
    }
}
