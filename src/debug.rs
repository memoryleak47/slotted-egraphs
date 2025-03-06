use crate::*;
use std::fmt::*;

impl Debug for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "id{}", self.0)
    }
}

#[cfg(feature = "explanations")]
impl Debug for Equation {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:?} = {:?}", self.l, self.r)
    }
}

impl Debug for SlotMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "[")?;
        let n = self.len();
        for (i, (x, y)) in self.iter().enumerate() {
            write!(f, "{x:?} -> {y:?}")?;
            if i < n - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, "]")
    }
}

impl Debug for AppliedId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:?}{:?}", self.id, self.m)
    }
}

impl<L: Language, N: Analysis<L>> Debug for EGraph<L, N> {
    fn fmt(&self, _f: &mut Formatter<'_>) -> Result {
        todo!()
    }
}
