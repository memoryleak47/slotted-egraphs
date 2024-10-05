use crate::*;

impl<L: Language> std::fmt::Display for RecExpr<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (x, rest) = self.node.to_op();

        if rest.is_empty() {
            return write!(f, "{}", x);
        }

        write!(f, "({} ", x)?;
        let mut child_idx = 0;
        let n = rest.len();
        for (i, r) in rest.into_iter().enumerate() {
            match r {
                Child::AppliedId(_) => {
                    write!(f, "{}", &self.children[child_idx])?;
                    child_idx += 1;
                },
                Child::Slot(slot) => {
                    write!(f, "{}", slot.to_string())?;
                },
            }
            if i != n-1 { write!(f, " ")?; }
        }
        write!(f, ")")
    }
}

impl<L: Language> std::fmt::Debug for RecExpr<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RE({})", self)
    }
}

impl<L: Language> RecExpr<L> {
    pub fn parse(s: &str) -> Option<Self> {
        let (re, rest) = parse_rec_expr(s)?;
        assert!(rest.is_empty());
        Some(re)
    }
}

fn parse_rec_expr<L: Language>(s: &str) -> Option<(RecExpr<L>, &str)> {
    let s = s.trim();
    if s.starts_with('(') {
        let s = s[1..].trim();
        let (op, rest) = op_str(s);
        let mut rest = rest.trim();
        let mut children = Vec::new();
        while !rest.starts_with(")") {
            let (child, rest2) = parse_child(rest)?;
            rest = rest2.trim();
            children.push(child);
        }
        assert!(rest.starts_with(")"));
        rest = rest[1..].trim();

        let children_mock = children.iter().map(|x|
            match x {
                ChildImpl::Slot(s) => Child::Slot(*s),
                ChildImpl::RecExpr(_) => Child::AppliedId(AppliedId::null()),
            }
        ).collect();
        let node = L::from_op(op, children_mock)?;
        let children = children.into_iter().filter_map(|x| match x {
            ChildImpl::RecExpr(re) => Some(re),
            ChildImpl::Slot(_) => None,
        }).collect();
        let re = RecExpr { node, children };
        Some((re, rest))
    } else {
        let (op, rest) = op_str(s);
        let node = L::from_op(op, vec![])?;
        let re = RecExpr { node, children: Vec::new() };
        Some((re, rest))
    }
}

enum ChildImpl<L: Language> {
    RecExpr(RecExpr<L>),
    Slot(Slot),
}

fn parse_child<L: Language>(s: &str) -> Option<(ChildImpl<L>, &str)> {
    if let Some((slot, rest)) = parse_slot(s) {
        return Some((ChildImpl::Slot(slot), rest));
    }

    parse_rec_expr::<L>(s).map(|(x, rest)| (ChildImpl::RecExpr(x), rest))
}

fn parse_slot(s: &str) -> Option<(Slot, &str)> {
    let (op, rest) = op_str(s);
    if !op.starts_with("s") { return None; }
    let num = &op[1..];

    let i = num.parse::<i64>().ok()?;
    Some((Slot::new_unchecked(i), rest))
}

// Returns the relevant substring for op parsing.
// The operator is anything delimited by ' ', '(', ')', or '\n'.
fn op_str(s: &str) -> (&str, &str) {
    if let Some((i, _)) = s.char_indices().find(|(_, c)| c.is_whitespace() || *c == '(' || *c == ')') {
        (&s[..i], &s[i..])
    } else { (s, "") }
}
