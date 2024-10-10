use crate::*;

fn assert_reaches(start: &str, goal: &str, steps: usize) {
    let start = RecExpr::parse(start).unwrap();
    let goal = RecExpr::parse(goal).unwrap();

    let mut eg = EGraph::new();
    eg.add_expr(start.clone());
    for _ in 0..steps {
        rewrite_arith(&mut eg);
        if let Some(i2) = lookup_rec_expr(&goal, &eg) {
            let i1 = lookup_rec_expr(&start, &eg).unwrap();
            if eg.eq(&i1, &i2) {
                #[cfg(feature = "explanations_tmp")]
                eg.explain_equivalence(start, goal).show_expr(&eg);
                return;
            }
        }
    }

    eg.dump();
    assert!(false);
}


#[test]
fn t1() { // x+y = y+x
    let x = "s0";
    let y = "s1";

    let a = &format!("(add (var {x}) (var {y}))");
    let b = &format!("(add (var {y}) (var {x}))");
    assert_reaches(a, b, 3);
}

#[test]
fn t2() { // (x+y) * (x+y) = (x+y) * (y+x)
    let x = "s0";
    let y = "s1";
    let z = "s2";

    let a = &format!("(mul (add (var {x}) (var {y})) (add (var {x}) (var {y})))");
    let b = &format!("(mul (add (var {x}) (var {y})) (add (var {y}) (var {x})))");

    assert_reaches(a, b, 3);
}

#[test]
fn t3() { // (x+y) * (y+z) = (z+y) * (y+x)
    let x = "s0";
    let y = "s1";
    let z = "s2";

    let a = &format!("(mul (add (var {x}) (var {y})) (add (var {y}) (var {z})))");
    let b = &format!("(mul (add (var {z}) (var {y})) (add (var {y}) (var {x})))");
    assert_reaches(a, b, 3);
}

#[test]
fn t4() { // (x+y)**2 = x**2 + x*y + x*y + y**2
    let x = "s0";
    let y = "s1";
    let z = "s2";

    let a = "(mul (add (var {x}) (var {y})) (add (var {x}) (var {y})))";
    let b = "(add (mul (var {x}) (var {x}))
             (add (mul (var {x}) (var {y}))
             (add (mul (var {x}) (var {y}))
                  (mul (var {y}) (var {y}))
             )))";
    assert_reaches(a, b, 10);
}

fn add_chain(it: impl Iterator<Item=usize>) -> String {
    let mut it = it.map(|u| format!("(var s{u})"));
    let mut x = format!("{}", it.next().unwrap());
    for y in it {
        x = format!("(add {x} {y})");
    }
    x
}

#[test]
fn t5() { // x0+...+xN = xN+...+x0
    // This times out for larger N!
    // TODO reset N to 7.
    const N: usize = 4;

    let a = &add_chain(0..=N);
    let b = &add_chain((0..=N).rev());

    assert_reaches(a, b, 10);
}

#[test]
fn t6() { // z*(x+y) = z*(y+x)
    let x = "s0";
    let y = "s1";
    let z = "s2";

    let a = &format!("(mul (var {z}) (add (var {x}) (var {y})))");
    let b = &format!("(mul (var {z}) (add (var {y}) (var {x})))");
    assert_reaches2(a, b, 10);


    // assert_reaches, but only using add_comm!
    fn assert_reaches2(start: &str, goal: &str, steps: usize) {
        let start = RecExpr::parse(start).unwrap();
        let goal = RecExpr::parse(goal).unwrap();

        let mut eg = EGraph::new();
        eg.add_expr(start.clone());
        for _ in 0..steps {
            apply_rewrites(&mut eg, &[add_comm()]);
            if let Some(i2) = lookup_rec_expr(&goal, &eg) {
                let i1 = lookup_rec_expr(&start, &eg).unwrap();
                if eg.eq(&i1, &i2) {
                    #[cfg(feature = "explanations_tmp")]
                    eg.explain_equivalence(start, goal).show_expr(&eg);
                    return;
                }
            }
        }

        eg.dump();
        assert!(false);
    }
}
