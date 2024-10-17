use crate::*;

pub fn array_parse(s: &str) -> RecExpr<Array> {
    pattern_to_re(&array_parse_pattern(s))
}

pub fn array_parse_pattern(s: &str) -> Pattern<Array> {
    translate(Pattern::parse(s).unwrap())
}

fn translate(p: Pattern<Sym>) -> Pattern<Array> {
    match p {
        Pattern::PVar(x) => Pattern::PVar(x),
        Pattern::ENode(Sym { op, children }, p_children) => {
            assert_eq!(children.len(), p_children.len());
            match (&*op.to_string(), &*p_children) {
                ("o", [f, g]) => {
                    let f = translate(f.clone()).to_string();
                    let g = translate(g.clone()).to_string();
                    let s = Slot::fresh().to_string();
                    Pattern::parse(&format!("(lam {s} (app {f} (app {g} (var {s}))))")).unwrap()
                },
                ("o", _) => panic!(),
                (x, children) => {
                    let mut pat = Pattern::ENode(
                        Array::Symbol(Symbol::from(x)),
                        Vec::new(),
                    );
                    for c in children {
                        pat = Pattern::ENode(
                            Array::App(AppliedId::null(), AppliedId::null()),
                            vec![pat, translate(c.clone())],
                        );
                    }
                    pat
                },
            }
        }
    }
}
