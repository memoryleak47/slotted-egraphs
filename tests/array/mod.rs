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

pub fn rules() -> Vec<Rewrite<ArrayLang>> { vec![
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
    if !slot_free_in("x", "f"))
] }
