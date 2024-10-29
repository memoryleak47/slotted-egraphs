use crate::*;

#[cfg(feature = "explanations")]
mod proof;
#[cfg(feature = "explanations")]
pub use proof::*;

#[cfg(feature = "explanations")]
mod front;
#[cfg(feature = "explanations")]
pub use front::*;

#[cfg(feature = "explanations")]
mod registry;
#[cfg(feature = "explanations")]
pub use registry::*;

mod wrapper;
pub use wrapper::*;

#[cfg(feature = "explanations")]
mod show;
#[cfg(feature = "explanations")]
pub use show::*;

#[cfg(not(feature = "explanations"))]
mod mock;
#[cfg(not(feature = "explanations"))]
pub use mock::*;

#[cfg(feature = "explanations")]
impl<L: Language, N: Analysis<L>> EGraph<L, N> {
    #[instrument(level = "trace", skip_all)]
    pub fn explain_equivalence(&mut self, t1: RecExpr<L>, t2: RecExpr<L>) -> ProvenEq {
        let i1 = self.add_syn_expr(t1);
        let i2 = self.add_syn_expr(t2);

        if !self.eq(&i1, &i2) { panic!("Can't explain an equivalence that does not hold!"); }

        let pai1 = self.proven_find_applied_id(&i1);
        let ProvenAppliedId { elem: l1, proof: prf1 } = &pai1;

        let pai2 = self.proven_find_applied_id(&i2);
        let ProvenAppliedId { elem: l2, proof: prf2 } = &pai2;

        if CHECKS {
            assert_eq!(l1.id, l2.id);
        }
        let id = l1.id;

        let bij = l2.m.compose(&l1.m.inverse());
        let symmetry_prf = &self.classes[&id].group.proven_contains(&bij).unwrap();
        let ProvenAppliedId { elem: l1, proof: prf1 } = self.chain_pai_pp(&pai1, symmetry_prf);

        let prf2 = self.prove_symmetry(prf2.clone());

        let final_eq = Equation { l: i1, r: i2 };
        let p = TransitivityProof(prf1, prf2.clone()).check(&final_eq, &self.proof_registry);

        if CHECKS {
            assert_proves_equation(&p, &final_eq);
        }

        p
    }
}
