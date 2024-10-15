use crate::*;

#[derive(Clone, PartialEq, Eq)]
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

    // TODO also support Add & Mul.
    fn make(sh: &Arith) -> ConstProp {
        match sh {
            Arith::Number(x) => ConstProp(Some(*x)),
            _ => ConstProp(None),
        }
    }
}
