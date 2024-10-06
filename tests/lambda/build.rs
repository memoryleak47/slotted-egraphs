use crate::lambda::*;

// The Y combinator.
pub fn y() -> String {
    let a = format!("(lam s0 (app (var s1) (app (var s0) (var s0))))");

    format!("(lam s1 (app {a} {a}))") }

pub fn zero() -> String {
    format!("(lam s0 (lam s1 (var s0)))")
}

pub fn suc() -> String {
    format!("(lam s0 (lam s1 (lam s2 (app (var s2) (var s0)))))")
}

pub fn num(x: u32) -> String {
    let mut out = zero();
    for _ in 0..x {
        out = app(suc(), out);
    }
    out
}

pub fn app(x: String, y: String) -> String {
    format!("(app {x} {y})")
}

// add 0 y = y
// add (x+1) y = add x (y+1)
// 
// add = Y add_impl
// 
// add_impl add x y = (x y) (\z. add z (suc y))
pub fn add() -> String {
    app(y(), add_impl())
}

pub fn add_impl() -> String {
    let s = suc();
    let add = "s0";
    let x = "s1";
    let y = "s2";
    let z = "s3";

    format!("(lam {add} (lam {x} (lam {y}
        (app (app (var {x}) (var {y})) (lam {z} (app (app (var {add}) (var {z})) (app {s} (var {y})))))
    )))")
}
