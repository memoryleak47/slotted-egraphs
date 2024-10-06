use crate::*;

pub fn array_parse(s: &str) -> RecExpr<Array> {
    pattern_to_re(&array_parse_pattern(s))
}

pub fn array_parse_pattern(s: &str) -> Pattern<Array> {
    translate(Pattern::parse(s).unwrap())
}

fn translate(p: Pattern<Sym>) -> Pattern<Array> {
    match p.node {
        ENodeOrPVar::PVar(x) => {
            RecExpr {
                node: ENodeOrPVar::PVar(x),
                children: Vec::new(),
            }
        },
        ENodeOrPVar::ENode(Sym { op, children }) => {
            assert_eq!(children.len(), p.children.len());
            match (&*op.to_string(), &*p.children) {
                ("o", [f, g]) => {
                    let f = translate(f.clone()).to_string();
                    let g = translate(g.clone()).to_string();
                    let s = Slot::fresh().to_string();
                    Pattern::parse(&format!("(lam {s} (app {f} (app {g} (var {s}))))")).unwrap()
                },
                ("o", _) => panic!(),
                (x, children) => {
                    let mut re = RecExpr {
                        node: ENodeOrPVar::ENode(Array::Symbol(Symbol::from(x))),
                        children: Vec::new(),
                    };
                    for c in children {
                        re = RecExpr {
                            node: ENodeOrPVar::ENode(Array::App(AppliedId::null(), AppliedId::null())),
                            children: vec![re, translate(c.clone())],
                        };
                    }
                    re
                },
            }
        },
    }
}
