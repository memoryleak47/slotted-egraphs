use slotted_egraphs::*;

define_language! {
    enum Lambda {
        Lam(Bind<AppliedId>) = "lam",
        App(AppliedId, AppliedId) = "app",
        Var(Slot) = "var",
    }
}

fn add(s: &str, eg: &mut EGraph<Lambda>) -> AppliedId {
    eg.add_syn_expr(RecExpr::parse(s).unwrap())
}

/// A class holding two terms of equal size, built in either order, then
/// `b[$1 := (app (var $2) (var $3))]` on it through the pattern machinery, as a rewrite
/// does it. The result is a new class whose syntactic term is the one the substitution
/// built, which says which of the two terms it rewrote; slot names are replaced by
/// their order of appearance.
fn two_orders<S: SubstMethod<Lambda, ()>>() -> (String, String) {
    let t1 = "(app (lam $0 (var $0)) (var $1))";
    let t2 = "(app (var $1) (lam $0 (var $0)))";
    let mut out = Vec::new();
    for (first, second) in [(t1, t2), (t2, t1)] {
        let mut eg = EGraph::<Lambda>::with_subst_method::<S>(());
        let a = add(first, &mut eg);
        let b = add(second, &mut eg);
        eg.union(&a, &b);
        eg.begin_round();
        let t = add("(app (var $2) (var $3))", &mut eg);
        let subst: Subst = [("b".to_string(), eg.find_applied_id(&a)), ("t".to_string(), t)]
            .into_iter()
            .collect();
        let pat = Pattern::parse("?b[(var $1) := ?t]").unwrap();
        let result = pattern_subst(&mut eg, &pat, &subst);
        out.push(canonical(&eg.get_syn_expr(&result).to_string()));
    }
    (out.remove(0), out.remove(0))
}

fn canonical(s: &str) -> String {
    let mut names: Vec<String> = Vec::new();
    let mut pieces = s.split('$');
    let mut out = pieces.next().unwrap_or("").to_string();
    for piece in pieces {
        let end = piece.find(|c: char| !c.is_alphanumeric()).unwrap_or(piece.len());
        let name = piece[..end].to_string();
        if !names.contains(&name) {
            names.push(name.clone());
        }
        let k = names.iter().position(|n| *n == name).unwrap();
        out.push_str(&format!("${k}{}", &piece[end..]));
    }
    out
}

#[test]
fn snapshot_subst_does_not_depend_on_the_order_terms_were_added() {
    let (first, second) = two_orders::<SnapshotSubst<Lambda>>();
    assert_eq!(first, second);
}

#[test]
fn syn_expr_subst_does() {
    let (first, second) = two_orders::<SynExprSubst>();
    assert_ne!(first, second);
}
