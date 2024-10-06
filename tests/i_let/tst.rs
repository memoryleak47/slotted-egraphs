use crate::*;

pub struct LetReal(EGraph<LetENode>);

impl Realization for LetReal {
    type Id = AppliedId;

    fn new() -> Self {
        LetReal(EGraph::new())
    }

    fn add_ast(&mut self, ast: &Ast) -> Self::Id {
        let re = RecExpr::<LetENode>::parse3(&ast.to_string());
        self.0.add_expr(re)
    }

    fn extract_ast(&self, id: Self::Id) -> Ast {
        let out = extract::<LetENode, AstSizeNoLet>(id, &self.0);
        Ast::parse(&out.to_string3())
    }

    fn find(&self, id: Self::Id) -> Self::Id {
        self.0.find_applied_id(&id)
    }

    fn step(&mut self) {
        rewrite_let(&mut self.0);
    }

    fn enode_count(&self) -> usize { self.0.total_number_of_nodes() }
    fn eclass_count(&self) -> usize { self.0.ids().len() } 

    fn explain_equivalence(&mut self, ast1: Ast, ast2: Ast) {
        let re1 = RecExpr::<LetENode>::parse3(&ast1.to_string());
        let re2 = RecExpr::<LetENode>::parse3(&ast2.to_string());
        self.0.explain_equivalence(re1, re2).show_expr(&self.0);
    }
}

pub trait Tmp3 {
    fn to_string3(&self) -> String;
    fn parse3(s: &str) -> Self;
}

impl Tmp3 for RecExpr<LetENode> {
    fn to_string3(&self) -> String {
        from_let(self).to_string2()
    }

    fn parse3(s: &str) -> Self {
        to_let(&RecExpr::<ENode>::parse2(s))
    }
}

fn to_let(re: &RecExpr<ENode>) -> RecExpr<LetENode> {
    let new_node = match re.node.clone() {
        ENode::Var(x) => LetENode::Var(x),
        ENode::App(l, r) => LetENode::App(l, r),
        ENode::Lam(x, b) => LetENode::Lam(x, b),
    };

    RecExpr {
        node: new_node,
        children: re.children.iter().map(to_let).collect(),
    }
}

fn from_let(re: &RecExpr<LetENode>) -> RecExpr<ENode> {
    let new_node = match re.node.clone() {
        LetENode::Var(x) => ENode::Var(x),
        LetENode::App(l, r) => ENode::App(l, r),
        LetENode::Lam(x, b) => ENode::Lam(x, b),
        LetENode::Let(..) => panic!("it contains let!"),
    };

    RecExpr {
        node: new_node,
        children: re.children.iter().map(from_let).collect(),
    }
}

unpack_tests!(LetReal);
