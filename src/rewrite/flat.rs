use std::ops::Index;

use crate::{Id, Language, PatternAst};

use super::{AppliedId, SlotMap};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ENodeOrVar<L> {
    /// An enode from the underlying [Language]
    ENode(L),
    /// A pattern variable
    Var(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RecExprFlat<L> {
    pub nodes: Vec<L>,
}
impl<L: Language> Language for ENodeOrVar<L> {
    fn all_slot_occurrences_mut(&mut self) -> Vec<&mut super::Slot> {
        todo!()
    }

    fn public_slot_occurrences_mut(&mut self) -> Vec<&mut super::Slot> {
        todo!()
    }

    fn applied_id_occurrences_mut(&mut self) -> Vec<&mut AppliedId> {
        match self {
            ENodeOrVar::ENode(l) => l.applied_id_occurrences_mut(),
            ENodeOrVar::Var(_) => vec![],
        }
    }

    fn all_slot_occurrences(&self) -> Vec<super::Slot> {
        todo!()
    }

    fn public_slot_occurrences(&self) -> Vec<super::Slot> {
        todo!()
    }

    fn applied_id_occurrences(&self) -> Vec<&AppliedId> {
        match self {
            ENodeOrVar::ENode(l) => l.applied_id_occurrences(),
            ENodeOrVar::Var(_) => vec![],
        }
    }

    fn to_syntax(&self) -> Vec<super::SyntaxElem> {
        todo!()
    }

    fn from_syntax(_: &[super::SyntaxElem]) -> Option<Self> {
        todo!()
    }

    fn slots(&self) -> super::SmallHashSet<super::Slot> {
        todo!()
    }

    fn weak_shape_inplace(&mut self) -> super::Bijection {
        todo!()
    }
}

pub type PatternAstFlat<L> = RecExprFlat<ENodeOrVar<L>>;

impl<L: Language> RecExprFlat<L> {
    /// Creates a new, empty RecExprFlat
    ///
    pub fn new() -> Self {
        RecExprFlat { nodes: Vec::new() }
    }

    /// Default implementation creates an empty expression
    pub fn default() -> Self {
        Self::new()
    }

    /// Adds a node to the RecExprFlat
    /// and returns its Id
    /// This method enforces the invariant that children must come before parents
    pub fn add(&mut self, node: L) -> AppliedId {
        // For ENodeOrVar, we need to validate child IDs if it's an ENode
        //
        // if let Some(children) = node_children(&node) {
        //     // Check that all child IDs refer to nodes that already exist
        //     for &child_id in children {
        //         let child_idx: usize = child_id.into();
        //         assert!(
        //             child_idx < self.nodes.len(),
        //             "Invalid child ID: {} (expression has {} nodes)",
        //             child_idx,
        //             self.nodes.len()
        //         );
        //     }
        // }
        let id = AppliedId::new(Id(self.nodes.len()), SlotMap::new());

        debug_assert!(
            node.applied_id_occurrences()
                .iter()
                .all(|id| id <= &&self.root()),
            "node {:?} has children not in this expr: {:?}",
            node,
            self
        );

        self.nodes.push(node);
        // }
        // if let ENodeOrVar::ENode(elem) = node {
        //     panic!()
        // };

        // // Add the node and return its ID
        // self.nodes.push(node);
        id
    }

    pub(crate) fn extract(&self, new_root: AppliedId) -> Self {
        self[new_root].build_recexpr(|id| self[id].clone())
    }

    // Additional helper methods
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn root(&self) -> AppliedId {
        AppliedId {
            id: Id(self.nodes.len() - 1),
            m: SlotMap::new(),
        }
    }
}

impl<L> IntoIterator for RecExprFlat<L> {
    type Item = L;
    type IntoIter = std::vec::IntoIter<L>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.into_iter()
    }
}

impl<L: Language> Index<AppliedId> for RecExprFlat<L> {
    type Output = L;
    fn index(&self, id: AppliedId) -> &L {
        &self.nodes[id.id.0]
    }
}

// Implement indexing for RecExprFlat
//
// impl<L> std::ops::Index<AppliedId> for PatternAstFlat<L> {
//     type Output = ENodeOrVar<L>;

//     fn index(&self, id: AppliedId) -> &Self::Output {
//         let idx: usize = id.id.0;
//         &self.nodes[idx]
//     }
// }

/// Converts a PatternAst to the flattened PatternAstFlat representation
pub fn pattern_ast_to_flat<L: Language + Clone>(pattern: &PatternAst<L>) -> PatternAstFlat<L> {
    let mut result = RecExprFlat::default();

    // Helper closure for post-order traversal
    fn build_flat<L: Language + Clone>(
        node: &PatternAst<L>,
        expr: &mut RecExprFlat<ENodeOrVar<L>>,
    ) -> AppliedId {
        match node {
            PatternAst::PVar(name) => {
                // Create a variable node
                let var = name.clone();
                expr.add(ENodeOrVar::Var(var))
            }
            PatternAst::ENode(op, children) => {
                // Convert children first
                let child_ids: Vec<_> = children
                    .iter()
                    .map(|child| build_flat(child, expr))
                    .collect();

                let mut new_op = op.clone();
                new_op
                    .applied_id_occurrences_mut()
                    .iter_mut()
                    .zip(child_ids)
                    .for_each(|(aid, cid)| aid.id = cid.id);
                // self.nodes.push(ENodeOrVar::ENode(new_node));
                // Then add this node
                expr.add(ENodeOrVar::ENode(new_op))
            }
            PatternAst::Subst(_body, _varr, _replacementnt) => {
                // // For substitution, we need to flatten the components in order
                // let body_id = build_flat(body, expr);
                // let var_id = build_flat(var, expr);
                // let replacement_id = build_flat(replacement, expr);

                // // Assuming your language has a way to encode substitutions
                // // Either through a special operation or metadata
                // expr.add(ENodeOrVar::ENode(
                //     L::substitution_operator(),
                //     vec![body_id, var_id, replacement_id],
                // ))
                panic!()
            }
        }
    }

    build_flat(pattern, &mut result);
    result
}
