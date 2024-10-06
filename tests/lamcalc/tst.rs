use crate::lamcalc::*;

#[macro_export]
macro_rules! unpack_tests {
    ($R:ty) => {
        #[test]
        fn test_cannot_simplify() {
            use lamcalc::*;

            let s = [
                "(lam x0 x0)",
                "(lam x0 (lam x1 x0))",
                "(lam x0 (lam x1 x1))",
                "(lam x0 (lam x1 (app x0 x1)))",
            ];

            for p in s {
                let out = simplify::<$R>(p);
                assert_alpha_eq(&*out, p);
            }
        }

        #[test]
        fn test_self_rec() {
            use lamcalc::*;

            // The intereting thing about this test is the following:
            // "\x. (\y. y) x -> \x. x" using beta reduction.
            //
            // and "\x. x -> \y. y" by alpha conversion.
            //
            // Thus, we suddenly have a self-recursive EClass, for Realizations that share across alpha-equivalence.
            // C = \y. y | \z. C z
            //
            // This sometimes causes infinite loops, if you iterate by depth-first-search.
            let s = "(lam x (app (lam y y) x))";
            check_simplify_to_nf::<$R>(&s);
        }

        #[test]
        fn test_t_shift() {
            use lamcalc::*;

            // This caught a bug. The "lam 0" (aka "lam z z") was shifted to "lam 0 1" incorrectly.
            let l = "(lam x (lam a x))";
            let r = "(lam z z)";
            let s = format!("(app {l} {r})");
            check_simplify_to_nf::<$R>(&s);
        }

        #[test]
        fn test_nested_identity1() {
            use lamcalc::*;

            let p = "(app (lam x0 x0) (lam x1 x1))";
            check_simplify_to_nf::<$R>(p);
        }

        #[test]
        fn test_nested_identity2() {
            use lamcalc::*;

            let p = "(app (lam x0 x0) (lam x1 (app x1 x1)))";
            check_simplify_to_nf::<$R>(p);
        }

        #[test]
        fn test_nested_identity3() {
            use lamcalc::*;

            let p = "(app (lam x0 (app x0 x0)) (lam x1 x1))";
            check_simplify_to_nf::<$R>(p);
        }

        #[test]
        fn test_simple_beta() {
            use lamcalc::*;

            let p = "(lam x (lam y
                (app
                    (lam z (app x z))
                y)
            ))";
            check_simplify_to_nf::<$R>(p);
        }

        #[test]
        fn test_redundant_slot() {
            use lamcalc::*;

            // y is unused, and hence x is effectively redundant.
            let p = "(lam x (app (lam y (lam z z)) x))";
            check_simplify_to_nf::<$R>(p);
        }

        #[test]
        fn test_redundant_slot2() {
            use lamcalc::*;

            // y is unused, and hence x is effectively redundant.
            let p = "(lam x (lam z (app (lam y z) x)))";
            check_simplify_to_nf::<$R>(p);
        }

        #[test]
        fn test_inf_loop() {
            use lamcalc::*;

            let p = "(app (lam x0 (app x0 x0)) (lam x1 (app x1 x1)))";
            let out = simplify::<$R>(p);
            assert_alpha_eq(&out, p);
        }

        // A y-combinator example that directly yields "f x = x" without looping.
        #[test]
        fn test_y_identity() {
            use lamcalc::*;

            let p = "(lam f (lam arg arg))";
            let s = app(y(), String::from(p));

            let out = simplify::<$R>(&s);
            assert_alpha_eq(&out, "(lam x x)");
        }

        #[test]
        fn test_add00() {
            use lamcalc::*;

            let s = app(app(add(), num(0)), num(0));
            check_simplify_to_nf::<$R>(&s);
        }

        #[test]
        fn test_add01() {
            use lamcalc::*;

            let s = app(app(add(), num(0)), num(1));
            check_simplify_to_nf::<$R>(&s);
        }

        #[test]
        fn test_add_y_step() {
            use lamcalc::*;

            let s1 = app(add_impl(), add());
            let s2 = add();
            check_eq::<$R>(&s1, &s2);
        }
    }
}
