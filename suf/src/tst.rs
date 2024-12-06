use crate::*;

fn set<const C: usize>(x: [&'static str; C]) -> HashSet<Slot> {
    x.iter().map(Slot::named).collect()
}

#[test]
fn simple() {
    let mut suf = Suf::new();
    let a = suf.add(set(["a_1", "a_2"]));
    let b = suf.add(set(["b_1", "b_2"]));
    // suf.union(a, b);
}
