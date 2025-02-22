use crate::*;

/// Specifies a certain implementation of how substitution `b[x := t]` is implemented internally.
pub trait SubstMethod<L: Language, N: Analysis<L>> {
    fn new_boxed() -> Box<dyn SubstMethod<L, N>>
    where
        Self: Sized;
    fn subst(
        &mut self,
        b: AppliedId,
        x: AppliedId,
        t: AppliedId,
        eg: &mut EGraph<L, N>,
    ) -> AppliedId;
}

/// A [SubstMethod] that uses the [EGraph::get_syn_expr] of an e-class to do substitution on it.
pub struct SynExprSubst;

impl<L: Language, N: Analysis<L>> SubstMethod<L, N> for SynExprSubst {
    fn new_boxed() -> Box<dyn SubstMethod<L, N>> {
        Box::new(SynExprSubst)
    }

    fn subst(
        &mut self,
        b: AppliedId,
        x: AppliedId,
        t: AppliedId,
        eg: &mut EGraph<L, N>,
    ) -> AppliedId {
        let term = eg.get_syn_expr(&eg.synify_app_id(b));
        do_term_subst(eg, &term, &x, &t)
    }
}

/// A [SubstMethod] that extracts the smallest term (measured by [AstSize]) of an e-class to do substitution on it.
pub struct ExtractionSubst;

impl<L: Language, N: Analysis<L>> SubstMethod<L, N> for ExtractionSubst {
    fn new_boxed() -> Box<dyn SubstMethod<L, N>> {
        Box::new(ExtractionSubst)
    }

    fn subst(
        &mut self,
        b: AppliedId,
        x: AppliedId,
        t: AppliedId,
        eg: &mut EGraph<L, N>,
    ) -> AppliedId {
        let term = ast_size_extract::<L, N>(&b, eg);
        do_term_subst(eg, &term, &x, &t)
    }
}

// returns re[x := t]
fn do_term_subst<L: Language, N: Analysis<L>>(
    eg: &mut EGraph<L, N>,
    re: &RecExpr<L>,
    x: &AppliedId,
    t: &AppliedId,
) -> AppliedId {
    let mut n = re.node.clone();
    let mut refs: Vec<&mut AppliedId> = n.applied_id_occurrences_mut();
    if CHECKS {
        assert_eq!(re.children.len(), refs.len());
    }
    for i in 0..refs.len() {
        *(refs[i]) = do_term_subst(eg, &re.children[i], x, t);
    }
    let app_id = eg.add_syn(n);

    if app_id == *x {
        return t.clone();
    } else {
        app_id
    }
}
