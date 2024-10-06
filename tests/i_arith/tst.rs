use crate::*;
use crate::i_arith::build::*;

fn assert_reaches(start: RecExpr<ArithENode>, goal: RecExpr<ArithENode>, steps: usize) {
    let mut eg = EGraph::new();
    eg.add_expr(start.clone());
    for _ in 0..steps {
        rewrite_arith(&mut eg);
        if let Some(i2) = lookup_rec_expr(&goal, &eg) {
            let i1 = lookup_rec_expr(&start, &eg).unwrap();
            if eg.eq(&i1, &i2) {
                eg.explain_equivalence(start, goal).show_expr(&eg);
                return;
            }
        }
    }

    eg.dump();
    assert!(false);
}


#[test]
fn arith_test1() { // x+y = y+x
    let x = 0;
    let y = 1;

    let a = add2(var(x), var(y));
    let a = pattern_to_re(&a);

    let b = add2(var(y), var(x));
    let b = pattern_to_re(&b);

    assert_reaches(a, b, 3);
}

#[test]
fn arith_test2() { // (x+y) * (x+y) = (x+y) * (y+x)
    let x = 0;
    let y = 1;
    let z = 2;

    let a = mul2(
                add2(var(x), var(y)),
                add2(var(x), var(y))
            );
    let a = pattern_to_re(&a);

    let b = mul2(
                add2(var(x), var(y)),
                add2(var(y), var(x))
            );
    let b = pattern_to_re(&b);

    assert_reaches(a, b, 3);
}

#[test]
fn arith_test3() { // (x+y) * (y+z) = (z+y) * (y+x)
    let x = 0;
    let y = 1;
    let z = 2;

    let a = mul2(
                add2(var(x), var(y)),
                add2(var(y), var(z))
            );
    let a = pattern_to_re(&a);

    let b = mul2(
                add2(var(z), var(y)),
                add2(var(y), var(x))
            );
    let b = pattern_to_re(&b);

    assert_reaches(a, b, 3);
}

#[test]
fn arith_test4() { // (x+y)**2 = x**2 + x*y + x*y + y**2
    let x = 0;
    let y = 1;
    let z = 2;

    let a = mul2(
                add2(var(x), var(y)),
                add2(var(x), var(y))
            );
    let a = pattern_to_re(&a);

    let b = add2(
                mul2(var(x), var(x)),
                add2(
                    mul2(var(x), var(y)),
                    add2(
                        mul2(var(x), var(y)),
                        mul2(var(y), var(y)),
                    ),
                )
            );
    let b = pattern_to_re(&b);

    assert_reaches(a, b, 10);
}

fn add_chain(it: impl Iterator<Item=usize>) -> Pattern<ArithENode> {
    let mut it = it.map(var);
    let mut x = it.next().unwrap();
    for y in it {
        x = add2(x, y);
    }
    x
}

#[test]
fn arith_test5() { // x0+...+xN = xN+...+x0
    // This times out for larger N!
    // TODO reset N to 7.
    const N: usize = 4;

    let a = add_chain(0..=N);
    let a = pattern_to_re(&a);

    let b = add_chain((0..=N).rev());
    let b = pattern_to_re(&b);

    assert_reaches(a, b, 10);
}

#[test]
fn arith_test6() { // z*(x+y) = z*(y+x)
    let x = 0;
    let y = 1;
    let z = 2;

    let a = mul2(var(z), add2(var(x), var(y)));
    let a = pattern_to_re(&a);

    let b = mul2(var(z), add2(var(y), var(x)));
    let b = pattern_to_re(&b);

    assert_reaches2(a, b, 10);


    // assert_reaches, but only using add_comm!
    fn assert_reaches2(start: RecExpr<ArithENode>, goal: RecExpr<ArithENode>, steps: usize) {
        let mut eg = EGraph::new();
        eg.add_expr(start.clone());
        for _ in 0..steps {
            add_comm(&mut eg);
            if let Some(i2) = lookup_rec_expr(&goal, &eg) {
                let i1 = lookup_rec_expr(&start, &eg).unwrap();
                if eg.eq(&i1, &i2) {
                    eg.explain_equivalence(start, goal).show_expr(&eg);
                    return;
                }
            }
        }

        eg.dump();
        assert!(false);
    }
}
