use crate::lamcalc::*;

impl Ast {
    pub fn run(&self) -> Ast {
        let mut ast = self.clone();
        while let Some(x) = ast.step() {
            ast = x;
        }

        ast
    }

    pub fn run_n(&self, n: u32) -> Ast {
        let mut ast = self.clone();
        for _ in 0..n {
            if let Some(x) = ast.step() {
                ast = x;
            }
        }

        ast
    }
}

impl Ast {
    pub fn step(&self) -> Option<Ast> {
        match self {
            Ast::Lam(x, b) => {
                let b = b.step()?;

                Some(Ast::Lam(String::from(x), Box::new(b)))
            },
            Ast::App(l, r) => {
                if let Ast::Lam(x, b) = &**l {
                    Some(b.subst(x, r))
                } else {
                    match l.step() {
                        Some(l) => Some(Ast::App(Box::new(l), r.clone())),
                        None => r.step().map(|r| Ast::App(l.clone(), Box::new(r))),
                    }
                }
            },
            Ast::Var(_) => None,
        }
    }

    fn subst(&self, x: &str, t: &Ast) -> Ast {
        match self {
            Ast::Lam(y, b) => {
                if x == y {
                    self.clone()
                } else {
                    let f = t.free_variables();

                    let mut y: String = (*y).clone();
                    let mut b: Ast = (**b).clone();

                    if f.contains(&y) {
                        let y2 = (0..).map(|i| format!("x{i}"))
                                      .filter(|i| !f.contains(i))
                                      .next()
                                      .unwrap();
                        let y2_node = Ast::Var(y2.clone());
                        b = b.subst(&y, &y2_node);
                        y = y2;
                    }
                    Ast::Lam(String::from(y), Box::new(b.subst(x, t)))
                }
            },
            Ast::App(l, r) => {
                Ast::App(Box::new(l.subst(x, t)), Box::new(r.subst(x, t)))
            },
            Ast::Var(y) => {
                if x == y {
                    t.clone()
                } else {
                    self.clone()
                }
            }
        }
    }

    fn free_variables(&self) -> HashSet<String> {
        match self {
            Ast::Lam(x, b) => &b.free_variables() - &HashSet::from([String::from(x)]),
            Ast::App(l, r) => &l.free_variables() | &r.free_variables(),
            Ast::Var(x) => HashSet::from([String::from(x)]),
        }
    }
}
