use crate::*;

pub trait Language {
    type With<R>;
}

enum MyLang<R=AppliedId> {
    Var(u32),
    App(R, R),
}

impl Language for MyLang {
    type With<R2> = MyLang<R2>;
}
