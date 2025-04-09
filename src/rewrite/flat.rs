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

pub type PatternAstFlat<L> = RecExprFlat<ENodeOrVar<L>>;

impl<L: Language> PatternAstFlat<L> {
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
    pub fn add(&mut self, node: ENodeOrVar<L>, child_ids: Option<Vec<AppliedId>>) -> AppliedId {
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

        match node {
            ENodeOrVar::ENode(n) => {
                let mut new_node = n.clone();
                new_node
                    .applied_id_occurrences_mut()
                    .iter_mut()
                    .zip(child_ids.unwrap())
                    .for_each(|(aid, cid)| aid.id = cid.id);
                self.nodes.push(ENodeOrVar::ENode(new_node));
                //
            }
            n @ ENodeOrVar::Var(_) => {
                self.nodes.push(n);
            }
        }
        // if let ENodeOrVar::ENode(elem) = node {
        //     panic!()
        // };

        // // Add the node and return its ID
        // self.nodes.push(node);
        id
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

// Implement indexing for RecExprFlat
//
impl<L> std::ops::Index<AppliedId> for PatternAstFlat<L> {
    type Output = ENodeOrVar<L>;

    fn index(&self, id: AppliedId) -> &Self::Output {
        let idx: usize = id.id.0;
        &self.nodes[idx]
    }
}

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
                expr.add(ENodeOrVar::Var(var), None)
            }
            PatternAst::ENode(op, children) => {
                // Convert children first
                let child_ids: Vec<_> = children
                    .iter()
                    .map(|child| build_flat(child, expr))
                    .collect();

                // Then add this node
                expr.add(ENodeOrVar::ENode(op.clone()), Some(child_ids))
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
