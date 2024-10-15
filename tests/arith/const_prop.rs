use crate::*;

#[derive(Clone, PartialEq, Eq, Debug)]
struct ConstProp(Option<u32>);

impl Analysis<Arith> for ConstProp {
    fn merge(x: ConstProp, y: ConstProp) -> ConstProp {
        match (x.0, y.0) {
            (Some(x), Some(y)) => {
                assert_eq!(x, y);
                ConstProp(Some(x))
            },
            (Some(x), _) => ConstProp(Some(x)),
            (_, Some(x)) => ConstProp(Some(x)),
            (_, _) => ConstProp(None),
        }
    }

    fn make(eg: &EGraph<Arith, Self>, sh: &Arith) -> ConstProp {
        match sh {
            Arith::Number(x) => ConstProp(Some(*x)),
            Arith::Add(x, y) => {
                ConstProp(
                    get_both(eg, x, y)
                        .map(|(x, y)| x+y)
                )
            },
            Arith::Mul(x, y) => {
                ConstProp(
                    get_both(eg, x, y)
                        .map(|(x, y)| x*y)
                )
            },
            _ => ConstProp(None),
        }
    }
}

fn get_both(eg: &EGraph<Arith, ConstProp>, x: &AppliedId, y: &AppliedId) -> Option<(u32, u32)> {
    Some((
        eg.analysis_data(x.id).0?,
        eg.analysis_data(y.id).0?,
    ))
}

#[test]
fn const_prop() {
    let start = RecExpr::parse("(add 2 (mul 2 3))").unwrap();

    let mut eg = EGraph::<Arith, ConstProp>::new();
    let i = eg.add_expr(start.clone());

    assert_eq!(eg.analysis_data(i.id), &ConstProp(Some(8)));
}
