use crate::*;

mod cost;
pub use cost::*;

mod with_ord;
pub use with_ord::*;

use std::collections::BinaryHeap;

pub struct Extractor<L: Language, CF: CostFunction<L>> {
    pub(crate) map: HashMap<Id, WithOrdRev<L, CF::Cost>>,
}

impl<L: Language, CF: CostFunction<L>> Extractor<L, CF> {
    pub fn new<N: Analysis<L>>(eg: &EGraph<L, N>) -> Self {
        eg.check();

        // all the L in `map` and `queue` have to be
        // - in "normal-form", i.e. calling lookup on them yields an identity AppliedId.
        // - every internal slot needs to be refreshed.

        // maps eclass id to their optimal RecExpr.
        let mut map: HashMap<Id, WithOrdRev<L, CF::Cost>> = HashMap::default();
        let mut queue: BinaryHeap<WithOrdRev<L, CF::Cost>> = BinaryHeap::new();

        for id in eg.ids() {
            for x in eg.enodes(id) {
                if x.applied_id_occurences().is_empty() {
                    let x = eg.class_nf(&x);
                    let c = CF::cost(&x, |_| panic!());
                    queue.push(WithOrdRev(x, c));
                }
            }
        }

        while let Some(WithOrdRev(enode, c)) = queue.pop() {
            let i = eg.lookup(&enode).unwrap();
            if map.contains_key(&i.id) {
                continue;
            }
            map.insert(i.id, WithOrdRev(enode, c));

            for x in eg.usages(i.id).clone() {
                if x.applied_id_occurences().iter().all(|i| map.contains_key(&i.id)) {
                    if eg.lookup(&x).map(|i| map.contains_key(&i.id)).unwrap_or(false) {
                        continue;
                    }
                    let x = eg.class_nf(&x);
                    let c = CF::cost(&x, |i| map[&i].1.clone());
                    queue.push(WithOrdRev(x, c));
                }
            }
        }

        Self { map }
    }

    pub fn extract<N: Analysis<L>>(&self, i: AppliedId, eg: &EGraph<L, N>) -> RecExpr<L> {
        let i = eg.find_applied_id(&i);

        let mut children = Vec::new();

        // do I need to refresh some slots here?
        let l = self.map[&i.id].0.apply_slotmap(&i.m);
        for child in l.applied_id_occurences() {
            let n = self.extract(child, eg);
            children.push(n);
        }

        RecExpr {
            node: l,
            children,
        }
    }
}

pub fn ast_size_extract<L: Language, N: Analysis<L>>(i: AppliedId, eg: &EGraph<L, N>) -> RecExpr<L> {
    extract::<L, N, AstSize>(i, eg)
}

// `i` is not allowed to have free variables, hence prefer `Id` over `AppliedId`.
pub fn extract<L: Language, N: Analysis<L>, CF: CostFunction<L>>(i: AppliedId, eg: &EGraph<L, N>) -> RecExpr<L> {
    Extractor::<L, CF>::new(eg).extract(i, eg)
}
