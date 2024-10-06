use crate::lamcalc::*;

impl Ast {
    pub fn normalize(&self) -> Ast {
        self.normalize_impl(&mut 0, Default::default())
    }

    // map :: original name -> normalized name.
    fn normalize_impl(&self, counter: &mut u32, map: HashMap<String, String>) -> Ast {
        let mut alloc_var = || {
            let out = format!("x{}", *counter);
            *counter += 1;
            out
        };

        match self {
            Ast::Lam(x, b) => {
                let mut map = map.clone();
                let norm_x = alloc_var();
                map.insert(x.clone(), norm_x.clone());

                let b = b.normalize_impl(counter, map);

                Ast::Lam(norm_x, Box::new(b))
            },
            Ast::App(l, r) => {
                let l = l.normalize_impl(counter, map.clone());
                let r = r.normalize_impl(counter, map.clone());

                Ast::App(Box::new(l), Box::new(r))
            },
            Ast::Var(x) => {
                let norm_x = String::from(&map[x]);

                Ast::Var(norm_x)
            },
        }
    }
}
