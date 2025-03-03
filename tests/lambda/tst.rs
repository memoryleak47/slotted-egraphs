use crate::lambda::*;

#[macro_export]
macro_rules! unpack_tests {
    ($R:ty) => {
        #[test]
        fn cannot_simplify() {
            use crate::lambda::*;

            let s = [
                "(lam $0 (var $0))",
                "(lam $0 (lam $1 (var $0)))",
                "(lam $0 (lam $1 (var $1)))",
                "(lam $0 (lam $1 (app (var $0) (var $1))))",
            ];

            for p in s {
                let out = simplify::<$R>(p);
                assert_alpha_eq(&*out, p);
            }
        }

        #[test]
        fn self_rec() {
            use lambda::*;

            // The intereting thing about this test is the following:
            // "\x. (\y. y) x -> \x. x" using beta reduction.
            //
            // and "\x. x -> \y. y" by alpha conversion.
            //
            // Thus, we suddenly have a self-recursive EClass, for Realizations that share across alpha-equivalence.
            // C = \y. y | \z. C z
            //
            // This sometimes causes infinite loops, if you iterate by depth-first-search.
            let s = "(lam $0 (app (lam $1 (var $1)) (var $0)))";
            check_simplify_to_nf::<$R>(&s);
        }

        #[test]
        fn t_shift() {
            use lambda::*;

            // This caught a bug. The "lam 0" (aka "lam z z") was shifted to "lam 0 1" incorrectly.
            let l = "(lam $0 (lam $1 (var $0)))";
            let r = "(lam $2 (var $2))";
            let s = format!("(app {l} {r})");
            check_simplify_to_nf::<$R>(&s);
        }

        #[test]
        fn nested_identity1() {
            use lambda::*;

            let p = "(app (lam $0 (var $0)) (lam $1 (var $1)))";
            check_simplify_to_nf::<$R>(p);
        }

        #[test]
        fn nested_identity2() {
            use lambda::*;

            let p = "(app (lam $0 (var $0)) (lam $1 (app (var $1) (var $1))))";
            check_simplify_to_nf::<$R>(p);
        }

        #[test]
        fn nested_identity3() {
            use lambda::*;

            let p = "(app (lam $0 (app (var $0) (var $0))) (lam $1 (var $1)))";
            check_simplify_to_nf::<$R>(p);
        }

        #[test]
        fn simple_beta() {
            use lambda::*;

            let p = "(lam $0 (lam $1
                (app
                    (lam $2 (app (var $0) (var $2)))
                (var $1))
            ))";
            check_simplify_to_nf::<$R>(p);
        }

        #[test]
        fn redundant_slot() {
            use lambda::*;

            // y is unused, and hence x is effectively redundant.
            let p = "(lam $0 (app (lam $1 (lam $2 (var $2))) (var $0)))";
            check_simplify_to_nf::<$R>(p);
        }

        #[test]
        fn redundant_slot2() {
            use lambda::*;

            // y is unused, and hence x is effectively redundant.
            let p = "(lam $0 (lam $2 (app (lam $1 (var $2)) (var $0))))";
            check_simplify_to_nf::<$R>(p);
        }

        #[test]
        fn inf_loop() {
            use lambda::*;

            let p = "(app (lam $0 (app (var $0) (var $0))) (lam $1 (app (var $1) (var $1))))";
            let out = simplify::<$R>(p);
            assert_alpha_eq(&out, p);
        }

        // A y-combinator example that directly yields "f x = x" without looping.
        #[test]
        #[ignore = "too slow"]
        fn y_identity() {
            use lambda::*;

            let p = "(lam $0 (lam $1 (var $1)))";
            let s = app(y(), String::from(p));

            let out = simplify::<$R>(&s);
            assert_alpha_eq(&out, "(lam $0 (var $0))");
        }

        #[test]
        #[ignore = "too slow"]
        fn add00() {
            use lambda::*;

            let s = app(app(add(), num(0)), num(0));
            check_simplify_to_nf::<$R>(&s);
        }

        #[test]
        #[ignore = "too slow"]
        fn add01() {
            use lambda::*;

            let s = app(app(add(), num(0)), num(1));
            check_simplify_to_nf::<$R>(&s);
        }

        #[test]
        fn add_y_step() {
            use lambda::*;

            let s1 = app(add_impl(), add());
            let s2 = add();
            check_eq::<$R>(&s1, &s2);
        }
    };
}
