use crate::*;

pub trait Language {
    type With<R>;

    fn with_clone<R: Clone>(x: &Self::With<R>) -> Self::With<R>;
}

pub struct Term<L: Language>(L::With<Box<Term<L>>>);

impl<L: Language> Clone for Term<L> {
    fn clone(&self) -> Self {
        Term(L::with_clone(&self.0))
    }
}


/// Example:
struct MyLang;

#[derive(Clone)]
enum MyNode<R> {
    Var(u32),
    App(R, R),
}

impl Language for MyLang {
    type With<R2> = MyNode<R2>;

    fn with_clone<R: Clone>(x: &Self::With<R>) -> Self::With<R> { x.clone() }
}

fn foo() {
    let x: Term<MyLang> = Term(MyNode::Var(20));
    let y = x.clone();
}
