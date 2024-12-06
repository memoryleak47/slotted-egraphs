use crate::*;

fn set<const C: usize>(l: [&'static str; C]) -> HashSet<Slot> {
    l.iter()
     .map(|x| Slot::named(*x))
     .collect()
}

fn map<const C: usize>(l: [(&'static str, &'static str); C]) -> SlotMap {
    l.iter()
     .map(|(x, y)| (Slot::named(*x), Slot::named(*y)))
     .collect()
}

#[test]
fn simple_suf() {
    let mut suf = Suf::new();
    let a = suf.add(set(["a_1", "a_2"]));
    let b = suf.add(set(["b_1", "b_2"]));
    let m = map([("a_1", "b_1"), ("a_2", "b_2")]);
    suf.dump();
    suf.union(a, b, &m);
    suf.dump();
    assert!(suf.is_equal(a, b, m));
}
