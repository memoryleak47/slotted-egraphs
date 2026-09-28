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

    /// Called by [apply_rewrites] before a round's rewrites are applied, and by
    /// [EGraph::begin_round] for a loop of your own. A method that reads the e-graph as
    /// the round began records what it needs here; the default does nothing.
    fn begin_round(&mut self, _eg: &EGraph<L, N>) {}
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

/// A [SubstMethod] that substitutes into the smallest term of the e-class -- by
/// [AstSize], ties broken by a canonical spelling of the term -- as the e-graph stood
/// when the round began.
///
/// Which term a substitution rewrites decides which e-nodes the e-graph holds from
/// then on, so two runs that make different choices end with different e-graphs.
/// [SynExprSubst] takes the node a class was created with, and [ExtractionSubst] reads
/// the e-graph after the round's earlier unions and breaks ties by extraction order;
/// both depend on the order in which classes were built and rewrites applied. This
/// method's choice is a function of the e-graph alone: two e-graphs holding the same
/// classes substitute into the same terms, in any order.
///
/// The table is built once per round, in [SubstMethod::begin_round], which
/// [apply_rewrites] calls; a loop that applies rewrites itself calls
/// [EGraph::begin_round]. Outside a round, or for a class the round created, the
/// class's syntactic term is used, as [SynExprSubst] would.
#[derive(Default)]
pub struct SnapshotSubst<L: Language> {
    best: HashMap<Id, L>,
}

impl<L: Language + 'static, N: Analysis<L>> SubstMethod<L, N> for SnapshotSubst<L> {
    fn new_boxed() -> Box<dyn SubstMethod<L, N>> {
        Box::new(SnapshotSubst { best: HashMap::default() })
    }

    fn begin_round(&mut self, eg: &EGraph<L, N>) {
        self.best = smallest_terms(eg);
    }

    fn subst(
        &mut self,
        b: AppliedId,
        x: AppliedId,
        t: AppliedId,
        eg: &mut EGraph<L, N>,
    ) -> AppliedId {
        let term = match snapshot_term(&self.best, &b) {
            Some(term) => term,
            None => eg.get_syn_expr(&eg.synify_app_id(b)),
        };
        do_term_subst(eg, &term, &x, &t)
    }
}

/// The chosen node's term under an invocation, its private slots fresh, as
/// [Extractor::extract] builds one.
fn snapshot_term<L: Language>(best: &HashMap<Id, L>, i: &AppliedId) -> Option<RecExpr<L>> {
    let node = best.get(&i.id)?.apply_slotmap_fresh(&i.m);
    let mut children = Vec::new();
    for child in node.applied_id_occurrences() {
        children.push(snapshot_term(best, &child)?);
    }
    Some(RecExpr { node, children })
}

fn ast_size<L: Language>(node: &L, cost: &HashMap<Id, u64>) -> Option<u64> {
    node.applied_id_occurrences()
        .iter()
        .try_fold(1u64, |total, child| Some(total.saturating_add(*cost.get(&child.id)?)))
}

/// The smallest term of every class, as a node per class in the class's own slot
/// names: the least [AstSize], and among nodes of that size the least [spelling].
fn smallest_terms<L: Language, N: Analysis<L>>(eg: &EGraph<L, N>) -> HashMap<Id, L> {
    let ids = eg.ids();
    let nodes: HashMap<Id, Vec<L>> = ids
        .iter()
        .map(|id| {
            let mut ns: Vec<L> = eg.enodes(*id).into_iter().collect();
            ns.sort_by_key(|n| format!("{n:?}"));
            (*id, ns)
        })
        .collect();
    let mut cost: HashMap<Id, u64> = HashMap::default();
    loop {
        let mut changed = false;
        for id in &ids {
            for node in &nodes[id] {
                let Some(total) = ast_size(node, &cost) else {
                    continue;
                };
                if cost.get(id).is_none_or(|prev| total < *prev) {
                    cost.insert(*id, total);
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    // children before parents, so a node is spelt from its children's spellings
    let mut by_size: Vec<(u64, Id)> = cost.iter().map(|(id, c)| (*c, *id)).collect();
    by_size.sort();
    let mut templates: HashMap<Id, Vec<Tok>> = HashMap::default();
    let mut best: HashMap<Id, L> = HashMap::default();
    for (size, id) in by_size {
        let public = eg.slots(id);
        let mut chosen: Option<(String, L, Vec<Tok>)> = None;
        for node in &nodes[&id] {
            if ast_size(node, &cost) != Some(size) {
                continue;
            }
            let tpl = template(node, &public, &templates);
            let key = spelling(&tpl);
            if chosen.as_ref().is_none_or(|(least, _, _)| key < *least) {
                chosen = Some((key, node.clone(), tpl));
            }
        }
        let (_, node, tpl) = chosen.expect("a class with a size has a node of that size");
        best.insert(id, node);
        templates.insert(id, tpl);
    }
    best
}

/// One piece of a term's spelling.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Tok {
    Text(String),
    /// a slot of the class, by its name there; a parent carries it through its edge
    Public(Slot),
    /// a slot internal to the term: bound, or private to one node
    Inner(usize),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum InnerKey {
    Bound(Slot),
    Private(Slot),
    ChildInner(usize, usize),
}

/// A node's term, spelt from its children's templates, with every slot classified in
/// the node's frame: bound by the node, a slot of the class, or private to the node.
fn template<L: Language>(node: &L, public: &SmallHashSet<Slot>, templates: &HashMap<Id, Vec<Tok>>) -> Vec<Tok> {
    let syntax = node.to_syntax();
    let mut inner: std::collections::BTreeMap<InnerKey, usize> = Default::default();
    let mut intern = |key: InnerKey| {
        let next = inner.len();
        Tok::Inner(*inner.entry(key).or_insert(next))
    };
    // a slot the node names outright is an occurrence like a child's; a slot that is
    // not a public occurrence is one the node binds
    let direct: SmallHashSet<Slot> = node.public_slot_occurrences().into_iter().collect();
    let mut out = Vec::new();
    for (col, elem) in syntax.iter().enumerate() {
        match elem {
            SyntaxElem::String(text) => out.push(Tok::Text(text.clone())),
            SyntaxElem::Slot(s) if direct.contains(s) => out.push(if public.contains(s) {
                Tok::Public(*s)
            } else {
                intern(InnerKey::Private(*s))
            }),
            SyntaxElem::Slot(s) => {
                out.push(Tok::Text("bind".to_owned()));
                out.push(intern(InnerKey::Bound(*s)));
            }
            SyntaxElem::AppliedId(child) => {
                out.push(Tok::Text("(".to_owned()));
                let tpl = templates
                    .get(&child.id)
                    .expect("a smallest node's children are spelt before it");
                for tok in tpl {
                    out.push(match tok {
                        Tok::Text(text) => Tok::Text(text.clone()),
                        Tok::Inner(j) => intern(InnerKey::ChildInner(col, *j)),
                        Tok::Public(slot) => {
                            let name = child.m[*slot];
                            if public.contains(&name) {
                                Tok::Public(name)
                            } else if direct.contains(&name) {
                                intern(InnerKey::Private(name))
                            } else {
                                intern(InnerKey::Bound(name))
                            }
                        }
                    });
                }
                out.push(Tok::Text(")".to_owned()));
            }
        }
        out.push(Tok::Text(" ".to_owned()));
    }
    out
}

/// A template as text, every slot numbered by first occurrence, so alpha-equivalent
/// spellings coincide and the e-graph's slot names play no part.
fn spelling(template: &[Tok]) -> String {
    let mut public: std::collections::BTreeMap<Slot, usize> = Default::default();
    let mut inner: std::collections::BTreeMap<usize, usize> = Default::default();
    let mut out = String::new();
    for tok in template {
        match tok {
            Tok::Text(text) => out.push_str(text),
            Tok::Public(slot) => {
                let next = public.len();
                out.push_str(&format!("$p{}", public.entry(*slot).or_insert(next)));
            }
            Tok::Inner(index) => {
                let next = inner.len();
                out.push_str(&format!("$i{}", inner.entry(*index).or_insert(next)));
            }
        }
    }
    out
}
