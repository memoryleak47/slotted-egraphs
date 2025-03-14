use crate::*;

pub struct LambdaRealNative;

impl Realization for LambdaRealNative {
    fn get_rewrites() -> Vec<Rewrite<Lambda>> {
        vec![beta()]
    }
}

unpack_tests!(LambdaRealNative);

pub fn rewrite_native(eg: &mut EGraph<Lambda>) {
    apply_rewrites(eg, &[beta()]);
}

fn beta() -> Rewrite<Lambda> {
    let pat = "(app (lam $1 ?b) ?t)";
    let outpat = "?b[(var $1) := ?t]";
    Rewrite::new("beta", pat, outpat)
}
