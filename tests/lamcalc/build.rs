use crate::lamcalc::*;

// The Y combinator.
pub fn y() -> String {
    let a = format!("(lam x (app f (app x x)))");

    format!("(lam f (app {a} {a}))") }

pub fn zero() -> String {
    format!("(lam x (lam y x))")
}

pub fn suc() -> String {
    format!("(lam arg (lam x (lam y (app y arg))))")
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
    format!("(lam add (lam x (lam y
        (app (app x y) (lam z (app (app add z) (app {s} y))))
    )))")
}
