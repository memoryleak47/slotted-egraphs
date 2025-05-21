use std::time::Duration;

use criterion::{criterion_group, Criterion};
use sdql::*;
use slotted_egraphs::*;

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("SDQL");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(20));
    group.bench_function("batax_v7_csr_dense_unfused_esat", |b| {
        b.iter(|| batax_v7_csr_dense_unfused_esat())
    });
}

criterion_group!(benches, criterion_benchmark);
fn main() {
    #[cfg(not(feature = "profile"))]
    {
        benches();
        criterion::Criterion::default()
            .configure_from_args()
            .final_summary();
    }
    #[cfg(feature = "profile")]
    batax_v7_csr_dense_unfused_esat();
}

fn batax_v7_csr_dense_unfused_esat() -> Report {
    let prog = "
(lambda $var_01
  (lambda $var_02
    (lambda $var_03
      (lambda $var_04
        (lambda $var_05
          (lambda $var_06
            (*
             (var $var_01)
             (sum
              (var $var_02)
              $var_07
              $var_08
              (* (sum
                  (subarray (var $var_03) (var $var_08) (- (get (var $var_02) (+ (var $var_07) 1)) 1))
                  $var_09
                  $var_10
                  (sing (var $var_10) (get (var $var_04) (var $var_09))))
                 (sum
                  (subarray (var $var_03) (var $var_08) (- (get (var $var_02) (+ (var $var_07) 1)) 1))
                  $var_09
                  $var_10
                  (* (get (var $var_04) (var $var_09)) (get (var $var_05) (var $var_10)))))))))))))
";

    let prog: RecExpr<Sdql> = RecExpr::parse(&prog).unwrap();
    let mut eg = EGraph::<Sdql, SdqlKind>::new();
    let rewrites = sdql_rules();
    let _id1 = eg.add_syn_expr(prog.clone());
    let report = run_eqsat(&mut eg, rewrites, 30, 5, move |_egraph| Ok(()));
    report
}

mod sdql {
    use slotted_egraphs::*;

    define_language! {
        pub enum Sdql {
            Lam(Bind<AppliedId>) = "lambda",
            Var(Slot) = "var",
            Sing(AppliedId, AppliedId) = "sing",
            Add(AppliedId, AppliedId) = "+",
            Mult(AppliedId, AppliedId) = "*",
            Sub(AppliedId, AppliedId) = "-",
            Equality(AppliedId, AppliedId) = "eq",
            Get(AppliedId, AppliedId) = "get",
            Range(AppliedId, AppliedId) = "range",
            App(AppliedId, AppliedId) = "apply",
            IfThen(AppliedId, AppliedId) = "ifthen",
            Binop(AppliedId, AppliedId, AppliedId) = "binop",
            SubArray(AppliedId, AppliedId, AppliedId) = "subarray",
            Unique(AppliedId) = "unique",
            Sum(
                /*  range: */ AppliedId,
                /*   body: */ Bind<Bind<AppliedId>>,
            ) = "sum",
            Merge(
                /* range1: */ AppliedId,
                /* range2: */ AppliedId,
                /*   body: */ Bind<Bind<Bind<AppliedId>>>,
            ) = "merge",
            Let(
                /*      v: */ AppliedId,
                /*   body: */ Bind<AppliedId>,
             ) = "let",
            Num(u32),
            Symbol(Symbol),
        }
    }

    #[derive(PartialEq, Eq, Clone, Debug)]
    pub struct SdqlKind {
        pub might_be_vector: bool,
        pub might_be_dict: bool,
        pub might_be_scalar: bool,
        pub might_be_bool: bool,
    }

    impl Analysis<Sdql> for SdqlKind {
        fn make(_eg: &slotted_egraphs::EGraph<Sdql, Self>, enode: &Sdql) -> Self {
            let mut out = SdqlKind {
                might_be_vector: false,
                might_be_dict: false,
                might_be_scalar: false,
                might_be_bool: false,
            };
            match enode {
                Sdql::SubArray(..) | Sdql::Range(..) => {
                    out.might_be_vector = true;
                }
                Sdql::Equality(..) => {
                    out.might_be_bool = true;
                }
                Sdql::Num(..) => {
                    out.might_be_scalar = true;
                }
                Sdql::Sing(..) => {
                    out.might_be_dict = true;
                }
                // Sdql::Sum(_, _, _, body) => {
                //     out = eg.analysis_data(body.id).clone();
                // }
                _ => {}
            }
            out
        }

        fn merge(a: Self, b: Self) -> Self {
            SdqlKind {
                might_be_vector: a.might_be_vector || b.might_be_vector,
                might_be_dict: a.might_be_dict || b.might_be_dict,
                might_be_scalar: a.might_be_scalar || b.might_be_scalar,
                might_be_bool: a.might_be_bool || b.might_be_bool,
            }
        }
    }

    type SdqlRewrite = Rewrite<Sdql, SdqlKind>;

    fn mult_assoc1() -> SdqlRewrite {
        Rewrite::new("mult-assoc1", "(* (* ?a ?b) ?c)", "(* ?a (* ?b ?c))")
    }
    fn mult_assoc2() -> SdqlRewrite {
        Rewrite::new("mult-assoc2", "(* ?a (* ?b ?c))", "(* (* ?a ?b) ?c)")
    }
    fn sub_identity() -> SdqlRewrite {
        Rewrite::new("sub-identity", "(- ?e ?e)", "0")
    }
    fn add_zero() -> SdqlRewrite {
        Rewrite::new("add-zero", "(+ ?e 0)", "?e")
    }
    fn sub_zero() -> SdqlRewrite {
        Rewrite::new("sub-zero", "(- ?e 0)", "?e")
    }
    fn eq_comm() -> SdqlRewrite {
        Rewrite::new("eq-comm", "(eq ?a ?b)", "(eq ?b ?a)")
    }
    fn mult_app1() -> SdqlRewrite {
        Rewrite::new("mult-app1", "(* ?a ?b)", "(binop mult ?a ?b)")
    }
    fn mult_app2() -> SdqlRewrite {
        Rewrite::new("mult-app2", "(binop mult ?a ?b)", "(* ?a ?b)")
    }
    fn add_app1() -> SdqlRewrite {
        Rewrite::new("add-app1", "(+ ?a ?b)", "(binop add ?a ?b)")
    }
    fn add_app2() -> SdqlRewrite {
        Rewrite::new("add-app2", "(binop add ?a ?b)", "(+ ?a ?b)")
    }
    fn sub_app1() -> SdqlRewrite {
        Rewrite::new("sub-app1", "(- ?a ?b)", "(binop sub ?a ?b)")
    }
    fn sub_app2() -> SdqlRewrite {
        Rewrite::new("sub-app2", "(binop sub ?a ?b)", "(- ?a ?b)")
    }
    fn get_app1() -> SdqlRewrite {
        Rewrite::new("get-app1", "(get ?a ?b)", "(binop getf ?a ?b)")
    }
    fn get_app2() -> SdqlRewrite {
        Rewrite::new("get-app2", "(binop getf ?a ?b)", "(get ?a ?b)")
    }
    fn sing_app1() -> SdqlRewrite {
        Rewrite::new("sing-app1", "(sing ?a ?b)", "(binop singf ?a ?b)")
    }
    fn sing_app2() -> SdqlRewrite {
        Rewrite::new("sing-app2", "(binop singf ?a ?b)", "(sing ?a ?b)")
    }
    fn unique_app1() -> SdqlRewrite {
        Rewrite::new("unique-app1", "(unique ?a)", "(apply uniquef ?a)")
    }
    fn unique_app2() -> SdqlRewrite {
        Rewrite::new("unique-app2", "(apply uniquef ?a)", "(unique ?a)")
    }

    fn let_binop3() -> SdqlRewrite {
        Rewrite::new(
            "let-binop3",
            "(let ?e1 $x (binop ?f ?e2 ?e3))",
            "(binop ?f (let ?e1 $x ?e2) (let ?e1 $x ?e3))",
        )
    }
    fn let_binop4() -> SdqlRewrite {
        Rewrite::new(
            "let-binop4",
            "(binop ?f (let ?e1 $x ?e2) (let ?e1 $x ?e3))",
            "(let ?e1 $x (binop ?f ?e2 ?e3))",
        )
    }

    fn let_apply1() -> SdqlRewrite {
        Rewrite::new(
            "let-apply1",
            "(let ?e1 $x (apply ?e2 ?e3))",
            "(apply ?e2 (let ?e1 $x ?e3))",
        )
    }
    fn let_apply2() -> SdqlRewrite {
        Rewrite::new(
            "let-apply2",
            "(apply ?e2 (let ?e1 $x ?e3))",
            "(let ?e1 $x (apply ?e2 ?e3))",
        )
    }

    fn if_mult2() -> SdqlRewrite {
        Rewrite::new(
            "if-mult2",
            "(* ?e1 (ifthen ?e2 ?e3))",
            "(ifthen ?e2 (* ?e1 ?e3))",
        )
    }
    fn if_to_mult() -> SdqlRewrite {
        Rewrite::new("if-to-mult", "(ifthen ?e1 ?e2)", "(* ?e1 ?e2)")
    }
    fn mult_to_if() -> SdqlRewrite {
        Rewrite::new(
            "mult-to-if",
            "(* (eq ?e1_1 ?e1_2) ?e2)",
            "(ifthen (eq ?e1_1 ?e1_2) ?e2)",
        )
    }

    fn beta() -> SdqlRewrite {
        Rewrite::new("beta", "(let ?t $x ?body)", "?body[(var $x) := ?t]")
    }

    fn sum_fact_1() -> SdqlRewrite {
        let pat = "(sum ?R $x $y (* ?e1 ?e2))";
        let outpat = "(* ?e1 (sum ?R $x $y ?e2))";

        Rewrite::new_if("sum-fact-1", pat, outpat, |subst, _| {
            !subst["e1"].slots().contains(&Slot::named("x"))
                && !subst["e1"].slots().contains(&Slot::named("y"))
        })
    }

    fn sum_fact_2() -> SdqlRewrite {
        let pat = "(sum ?R $x $y (* ?e1 ?e2))";
        let outpat = "(* (sum ?R $x $y ?e1) ?e2)";

        Rewrite::new_if("sum-fact-2", pat, outpat, |subst, _| {
            !subst["e2"].slots().contains(&Slot::named("x"))
                && !subst["e2"].slots().contains(&Slot::named("y"))
        })
    }

    fn sum_fact_3() -> SdqlRewrite {
        let pat = "(sum ?R $x $y (sing ?e1 ?e2))";
        let outpat = "(sing ?e1 (sum ?R $x $y ?e2))";

        Rewrite::new_if("sum-fact-3", pat, outpat, |subst, _| {
            !subst["e1"].slots().contains(&Slot::named("x"))
                && !subst["e1"].slots().contains(&Slot::named("y"))
        })
    }

    fn sing_mult_1() -> SdqlRewrite {
        Rewrite::new(
            "sing-mult-1",
            "(sing ?e1 (* ?e2 ?e3))",
            "(* (sing ?e1 ?e2) ?e3)",
        )
    }

    fn sing_mult_2() -> SdqlRewrite {
        Rewrite::new(
            "sing-mult-2",
            "(sing ?e1 (* ?e2 ?e3))",
            "(* ?e2 (sing ?e1 ?e3))",
        )
    }

    fn sing_mult_3() -> SdqlRewrite {
        Rewrite::new(
            "sing-mult-3",
            "(* (sing ?e1 ?e2) ?e3)",
            "(sing ?e1 (* ?e2 ?e3))",
        )
    }

    fn sing_mult_4() -> SdqlRewrite {
        Rewrite::new(
            "sing-mult-4",
            "(* ?e2 (sing ?e1 ?e3))",
            "(sing ?e1 (* ?e2 ?e3))",
        )
    }

    fn sum_fact_inv_1() -> SdqlRewrite {
        Rewrite::new(
            "sum-fact-inv-1",
            "(* ?e1 (sum ?R $k $v ?e2))",
            "(sum ?R $k $v (* ?e1 ?e2))",
        )
    }

    fn sum_fact_inv_3() -> SdqlRewrite {
        Rewrite::new(
            "sum-fact-inv-3",
            "(sing ?e1 (sum ?R $k $v ?e2))",
            "(sum ?R $k $v (sing ?e1 ?e2))",
        )
    }

    fn sum_sum_vert_fuse_1() -> SdqlRewrite {
        let pat = "(sum (sum ?R $k2 $v2 (sing (var $k2) ?body1)) $k1 $v1 ?body2)";
        let outpat = "(sum ?R $k2 $v2 (let (var $k2) $k1 (let ?body1 $v1 ?body2)))";

        Rewrite::new("sum-sum-vert-fuse-1", pat, outpat)
    }

    fn sum_sum_vert_fuse_2() -> SdqlRewrite {
        let pat = "(sum (sum ?R $k2 $v2 (sing (unique ?key) ?body1)) $k1 $v1 ?body2)";
        let outpat = "(sum ?R $k2 $v2 (let (unique ?key) $k1 (let ?body1 $v1 ?body2)))";

        Rewrite::new("sum-sum-vert-fuse-2", pat, outpat)
    }

    #[allow(unused)]
    fn get_sum_vert_fuse_1() -> SdqlRewrite {
        let pat = "(get (sum $k $v ?R (sing (var $k) ?body1)) ?body2)";
        let outpat = "(let $k ?body2 (let $v (get ?R (var $k)) ?body1))";
        Rewrite::new("get-sum-vert-fuse-1", pat, outpat)
    }

    fn sum_range_1() -> SdqlRewrite {
        Rewrite::new(
            "sum-range-1",
            "(sum (range ?st ?en) $k $v (ifthen (eq (var $v) ?key) ?body))",
            "(sum (range ?st ?en) $k $v (ifthen (eq (var $k) (- ?key (- ?st 1))) ?body))",
        )
    }

    fn sum_range_2() -> SdqlRewrite {
        Rewrite::new_if(
            "sum-range-2",
            "(sum (range ?st ?en) $k $v (ifthen (eq (var $k) ?key) ?body))",
            "(let ?key $k (let (+ (var $k) (- ?st 1)) $v ?body))",
            |subst, _| {
                !subst["key"].slots().contains(&Slot::named("k"))
                    && !subst["key"].slots().contains(&Slot::named("v"))
            },
        )
    }

    fn sum_merge() -> SdqlRewrite {
        Rewrite::new(
            "sum-merge",
            "(sum ?R $k1 $v1 (sum ?S $k2 $v2 (ifthen (eq (var $v1) (var $v2)) ?body)))",
            "(merge ?R ?S $k1 $k2 $v1 (let (var $v1) $v2 ?body))",
        )
    }

    fn get_to_sum() -> SdqlRewrite {
        Rewrite::new(
            "get-to-sum",
            "(get ?dict ?key)",
            "(sum ?dict $k $v (ifthen (eq (var $k) ?key) (var $v)))",
        )
    }

    fn sum_to_get() -> SdqlRewrite {
        Rewrite::new_if(
            "sum-to-get",
            "(sum ?dict $k $v (ifthen (eq (var $k) ?key) ?body))",
            "(let ?key $k (let (get ?dict (var $k)) $v ?body))",
            |subst, _| {
                !subst["key"].slots().contains(&Slot::named("k"))
                    && !subst["key"].slots().contains(&Slot::named("v"))
            },
        )
    }

    fn get_range() -> SdqlRewrite {
        Rewrite::new(
            "get-range",
            "(get (range ?st ?en) ?idx)",
            "(+ ?idx (- ?st 1))",
        )
    }

    fn sum_sing() -> SdqlRewrite {
        Rewrite::new(
            "sum-sing",
            "(sum ?e1 $k $v (sing (var $k) (var $v)))",
            "?e1",
        )
    }

    fn unique_rm() -> SdqlRewrite {
        Rewrite::new("unique-rm", "(unique ?e)", "?e")
    }

    pub fn sdql_rules() -> Vec<SdqlRewrite> {
        vec![
            mult_assoc1(),
            mult_assoc2(),
            sub_identity(),
            add_zero(),
            sub_zero(),
            eq_comm(),
            mult_app1(),
            mult_app2(),
            add_app1(),
            add_app2(),
            sub_app1(),
            sub_app2(),
            get_app1(),
            get_app2(),
            sing_app1(),
            sing_app2(),
            unique_app1(),
            unique_app2(),
            let_binop3(),
            let_binop4(),
            let_apply1(),
            let_apply2(),
            if_mult2(),
            if_to_mult(),
            mult_to_if(),
            beta(),
            sum_fact_1(),
            sum_fact_2(),
            sum_fact_3(),
            sing_mult_1(),
            sing_mult_2(),
            sing_mult_3(),
            sing_mult_4(),
            sum_fact_inv_1(),
            sum_fact_inv_3(),
            sum_sum_vert_fuse_1(),
            sum_sum_vert_fuse_2(),
            sum_range_1(),
            sum_merge(),
            get_to_sum(),
            sum_to_get(),
            get_range(),
            sum_sing(),
            unique_rm(),
        ]
    }
}
