use crate::*;

pub trait Language {
    type With<R>;

    fn with_clone<R: Clone>(x: &Self::With<R>) -> Self::With<R>;

    fn weak_shape(n: &Node<Self>) -> Applied<Shape<Self>> where Node<Self>: Clone, Self: Sized, Self::With<AppliedId>: Access<Slot> {
        let mut n = n.clone();
        let m = Self::weak_shape_inplace(&mut n);
        Applied(m, n)
    }

    fn weak_shape_inplace(n: &mut Node<Self>) -> SlotMap where Self: Sized, Self::With<AppliedId>: Access<Slot> {
        struct WeakShapeHandler;

        impl<'a> Handler<&'a mut Slot> for WeakShapeHandler {
            type R = SlotMap;
            fn call(self, it: impl Iterator<Item=&'a mut Slot>) -> SlotMap {
                let mut m = SlotMap::new();
                for x in it {
                    // TODO fix double binary search of m[x].
                    if let Some(y) = m.get(*x) {
                        *x = y;
                    } else {
                        let y = Slot::numeric(m.len() as _);
                        m.insert(*x, y);
                        *x = y;
                    }
                }
                m
            }
        }

        n.0.access_mut(WeakShapeHandler)
    }
}

pub struct Term<L: Language>(L::With<Box<Term<L>>>);
pub struct Node<L: Language>(L::With<AppliedId>);
pub type Shape<L> = Node<L>;

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
