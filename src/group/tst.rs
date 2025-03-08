use crate::*;

#[test]
fn group_test1() {
    let perm1 = flip(4, 0, 1);
    let perm2 = flip(4, 2, 3);
    check_group([perm1, perm2]);
}

#[test]
fn group_test2() {
    let perm1 = shift(4);
    let perm2 = flip(4, 0, 1);
    check_group([perm1, perm2]);
}

#[test]
fn group_test3() {
    let perm1 = shift(4);
    let perm2 = flip(4, 0, 2);
    check_group([perm1, perm2]);
}

// perms:

fn shift(n: usize) -> Perm {
    mk_perm(n, |i| (i + 1) % n)
}

fn flip(n: usize, x: usize, y: usize) -> Perm {
    mk_perm(n, |i| {
        if i == x {
            y
        } else if i == y {
            x
        } else {
            i
        }
    })
}

// helper fns:

fn check_group(generators: impl IntoIterator<Item = Perm>) {
    let generators: HashSet<Perm> = generators.into_iter().collect();
    let omega: SmallHashSet<_> = generators.iter().next().unwrap().values();
    let identity = SlotMap::identity(&omega);
    let l = Group::new(&identity, generators.clone()).all_perms();
    let r = enrich(generators);
    assert_eq!(l, r);
}

fn enrich(perms: HashSet<Perm>) -> HashSet<Perm> {
    let mut perms = perms;
    assert!(perms.len() > 0); // We can't add the identity, because we don't have omega here.

    loop {
        let copy = perms.clone();
        for x in &copy {
            for y in &copy {
                perms.insert(x.compose(y));
            }
        }
        if copy.len() == perms.len() {
            break;
        }
    }
    perms
}

fn s(n: usize) -> Slot {
    Slot::numeric(n as _)
}

fn mk_perm(n: usize, f: impl Fn(usize) -> usize) -> Perm {
    (0..n).map(|x| (s(x), s(f(x)))).collect()
}
