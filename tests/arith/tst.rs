use crate::*;

#[test]
fn t1() {
    // x+y = y+x
    let x = "$0";
    let y = "$1";

    let a = &format!("(add (var {x}) (var {y}))");
    let b = &format!("(add (var {y}) (var {x}))");

    assert_reaches(a, b, &get_all_rewrites()[..], 3);
    // todo!();
}

#[test]
fn t2() {
    // (x+y) * (x+y) = (x+y) * (y+x)
    let x = "$0";
    let y = "$1";
    let z = "$2";

    let a = &format!("(mul (add (var {x}) (var {y})) (add (var {x}) (var {y})))");
    let b = &format!("(mul (add (var {x}) (var {y})) (add (var {y}) (var {x})))");

    assert_reaches(a, b, &get_all_rewrites()[..], 2);
}

#[test]
fn t3() {
    // (x+y) * (y+z) = (z+y) * (y+x)
    let x = "$0";
    let y = "$1";
    let z = "$2";

    let a = &format!("(mul (add (var {x}) (var {y})) (add (var {y}) (var {z})))");
    let b = &format!("(mul (add (var {z}) (var {y})) (add (var {y}) (var {x})))");

    assert_reaches(a, b, &get_all_rewrites()[..], 3);
}

#[test]
fn t4() {
    // (x+y)**2 = x**2 + x*y + x*y + y**2
    let a = "(mul (add (var $x) (var $y)) (add (var $x) (var $y)))";
    let b = "(add (mul (var $x) (var $x))
             (add (mul (var $x) (var $y))
             (add (mul (var $x) (var $y))
                  (mul (var $y) (var $y))
             )))";
    assert_reaches(a, b, &get_all_rewrites()[..], 10);
}

fn add_chain(it: impl Iterator<Item = usize>) -> String {
    let mut it = it.map(|u| format!("(var ${u})"));
    let mut x = format!("{}", it.next().unwrap());
    for y in it {
        x = format!("(add {x} {y})");
    }
    x
}

#[test]
#[cfg_attr(feature = "explanations", ignore = "TODO: fails")]
fn t5() {
    // x0+...+xN = xN+...+x0
    // This times out for larger N!
    // TODO reset N to 7.
    const N: usize = 4;

    let a = &add_chain(0..=N);
    let b = &add_chain((0..=N).rev());

    assert_reaches(a, b, &get_all_rewrites()[..], 10);
}

#[test]
fn t6() {
    // z*(x+y) = z*(y+x)
    let x = "$0";
    let y = "$1";
    let z = "$2";

    let a = &format!("(mul (var {z}) (add (var {x}) (var {y})))");
    let b = &format!("(mul (var {z}) (add (var {y}) (var {x})))");
    assert_reaches(a, b, &[add_comm()], 10);
}
