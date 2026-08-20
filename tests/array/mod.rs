use crate::*;

define_language! {
    pub enum ArrayLang {
        // lambda calculus:
        Lam(Slot, AppliedId) = "lam",
        App(AppliedId, AppliedId) = "app",
        Var(Slot) = "var",
        Let(Bind<AppliedId>, AppliedId) = "let",

        // rest:
        Number(u32),
        Symbol(Symbol),
    }
}

pub fn rules() -> Vec<Rewrite<ArrayLang>> {
    vec![
        // lambda calculus:
        rw!("eta"; "(lam $x (app ?f (var $x)))" => "?f", if !slot_free_in("x", "f")),
        rw!("beta"; "(app (lam $x ?body) ?e)" => "?body[(var $x) := ?e]"),
        // perform substitution explicitly as an alternative to beta:
        rw!("let-intro"; "(app (lam $x ?body) ?e)" => "(let $x ?body ?e)"),
        rw!("let-unused"; "(let $x ?b ?e)" => "?b", if !slot_free_in("x", "b")),
        rw!("let-var-same"; "(let $x (var $x) ?e)" => "?e"),
        rw!("let-app";"(let $x (app ?a ?b) ?e)"=>"(app (let $x ?a ?e) (let $x ?b ?e))",
    if or(slot_free_in("x", "a"), slot_free_in("x", "b"))),
        rw!("let-lam-diff"; "(let $x (lam $y ?body) ?e)"=>"(lam $y (let $x ?body ?e))",
    if slot_free_in("x", "body")),
        // map fusion and fission:
        rw!("map-fusion"; "(app (app map ?f) (app (app map ?g) ?arg))" =>
    "(app (app map (lam $x (app ?f (app ?g (var $x))))) ?arg)"),
        rw!("map-fission"; "(app map (lam $x (app ?f ?gx)))" =>
    "(lam $in (app (app map ?f) (app (app map (lam $x ?gx)) (var $in))))",
    if !slot_free_in("x", "f")),
    ]
}

/// `eta` rewrites `(lam $x (app ?f (var $x)))` to `?f` only when `$x` is not free
/// in `?f`, so `slot_free_in` has to mean what its name says. It used to return the
/// negation, which inverted this guard and the four others in this file.
#[test]
fn eta_needs_the_slot_absent() {
    // $0 is not free in `(var $1)`, so eta applies.
    let mut eg: EGraph<ArrayLang> = EGraph::default();
    let lam = id("(lam $0 (app (var $1) (var $0)))", &mut eg);
    let f = id("(var $1)", &mut eg);
    let mut runner = Runner::<ArrayLang>::default().with_egraph(eg);
    runner.run(&rules()[..]);
    assert!(
        runner.egraph.eq(&lam, &f),
        "eta should fire when the bound slot is absent from ?f"
    );

    // $0 IS free in `(var $0)`, so eta must not apply.
    let mut eg2: EGraph<ArrayLang> = EGraph::default();
    let lam2 = id("(lam $0 (app (var $0) (var $0)))", &mut eg2);
    let self_app = id("(lam $0 (var $0))", &mut eg2);
    let mut runner2 = Runner::<ArrayLang>::default().with_egraph(eg2);
    runner2.run(&rules()[..]);
    assert!(
        !runner2.egraph.eq(&lam2, &self_app),
        "eta must not fire when the bound slot occurs in ?f"
    );
}
