// The Y combinator.
pub fn y() -> String {
    let a = format!("(lam $0 (app (var $1) (app (var $0) (var $0))))");

    format!("(lam $1 (app {a} {a}))")
}

pub fn zero() -> String {
    format!("(lam $0 (lam $1 (var $0)))")
}

pub fn suc() -> String {
    format!("(lam $0 (lam $1 (lam $2 (app (var $2) (var $0)))))")
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
    let add = "$0";
    let x = "$1";
    let y = "$2";
    let z = "$3";

    format!("(lam {add} (lam {x} (lam {y}
        (app (app (var {x}) (var {y})) (lam {z} (app (app (var {add}) (var {z})) (app {s} (var {y})))))
    )))")
}
