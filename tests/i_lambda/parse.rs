use crate::*;

pub trait Tmp {
    fn parse2(s: &str) -> Self;
    fn to_string2(&self) -> String;
}

impl Tmp for RecExpr<ENode> {
    fn parse2(s: &str) -> Self {
        let ast = Ast::parse(s);
        parse_impl(&ast, &mut Default::default())
    }

    fn to_string2(&self) -> String {
        let mut name_id = 0;
        let mut namegen = || {
            let name = format!("x{name_id}");
            name_id += 1;

            name
        };
        let mut map: HashMap<Slot, String> = Default::default();
        let mut m = |s: Slot| {
            match map.get(&s) {
                Some(name) => name.clone(),
                None => {
                    let name = namegen();
                    map.insert(s, name.clone());
                    name
                },
            }
        };
        to_string_impl(self, &mut m)
    }    
}


fn parse_impl(ast: &Ast, m: &mut HashMap<String, Slot>) -> RecExpr<ENode> {
    let mut getname = |n: &str| -> Slot {
        match m.get(n) {
            Some(i) => *i,
            None => {
                let f = Slot::fresh();
                m.insert(n.to_string(), f);
                f
            }
        }
    };
    match ast {
        Ast::Var(x) => {
            let x = getname(x);
            RecExpr {
                node: ENode::Var(x),
                children: Vec::new(),
            }
        },
        Ast::App(l, r) => {
            RecExpr {
                node: ENode::App(AppliedId::null(), AppliedId::null()),
                children: vec![parse_impl(l, m), parse_impl(r, m)],
            }
        },
        Ast::Lam(x, b) => {
            let x = getname(x);
            RecExpr {
                node: ENode::Lam(x, AppliedId::null()),
                children: vec![parse_impl(b, m)],
            }
        },
    }
}

fn to_string_impl(re: &RecExpr<ENode>, m: &mut impl FnMut(Slot) -> String) -> String {
    match &re.node {
        ENode::Var(x) => m(*x),
        ENode::App(_, _) => format!("(app {} {})", to_string_impl(&re.children[0], m), to_string_impl(&re.children[1], m)),
        ENode::Lam(x, _) => format!("(lam {} {})", m(*x), to_string_impl(&re.children[0], m)),
    }
}

#[test]
fn test_parse_roundtrip2() {
    let s1 = "(app (lam x0 x0) (lam x1 x1))";
    let p = RecExpr::<ENode>::parse2(s1);
    let s2 = p.to_string2();
    assert_eq!(s1, s2);
}
