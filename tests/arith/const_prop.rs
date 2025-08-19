use crate::*;

#[derive(Default)]
pub struct ConstProp;

impl Analysis<Arith> for ConstProp {
    type Data = Option<u32>;

    fn merge(x: Option<u32>, y: Option<u32>) -> Option<u32> {
        match (x, y) {
            (Some(x), Some(y)) => {
                assert_eq!(x, y);
                Some(x)
            }
            (Some(x), _) => Some(x),
            (_, Some(x)) => Some(x),
            (_, _) => None,
        }
    }

    fn make(eg: &EGraph<Arith, Self>, sh: &Arith) -> Option<u32> {
        match sh {
            Arith::Number(x) => Some(*x),
            Arith::Add(x, y) => get_both(eg, x, y).map(|(x, y)| x + y),
            Arith::Mul(x, y) => get_both(eg, x, y).map(|(x, y)| x * y),
            _ => None,
        }
    }
}

fn get_both(eg: &EGraph<Arith, ConstProp>, x: &AppliedId, y: &AppliedId) -> Option<(u32, u32)> {
    Some(((*eg.analysis_data(x.id))?, (*eg.analysis_data(y.id))?))
}

#[test]
fn const_prop() {
    let start = RecExpr::parse("(add 2 (mul 2 3))").unwrap();

    let mut eg = EGraph::<Arith, ConstProp>::default();
    let i = eg.add_expr(start.clone());

    assert_eq!(eg.analysis_data(i.id), &Some(8));
}

#[test]
fn const_prop_union() {
    let mut eg = EGraph::<Arith, ConstProp>::default();
    let a = eg.add_expr(RecExpr::parse("a").unwrap());
    let b = eg.add_expr(RecExpr::parse("42").unwrap());
    eg.union(&a, &b);

    assert_eq!(eg.analysis_data(a.id), &Some(42));
}
