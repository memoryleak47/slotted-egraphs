use crate::*;

pub struct SlottedUF<C>(Vec<SUFClass<C>>);

struct SUFClass<C> {
    leader: AppliedId,
    s: HashSet<Slot>,
    g: Group<SlotMap>,

    c: C,
}

impl<C> SlottedUF<C> {
    pub fn new() -> Self {
        SlottedUF(Vec::new())
    }

    pub fn add(SlottedUF(v): &mut Self, s: HashSet<Slot>, c: C) -> AppliedId {
        let i = v.len();
        let identity = SlotMap::identity(&s);
        let g = Group::new(&identity, Default::default());
        let out = Applied(identity, Id(i));
        let leader = out.clone();
        v.push(SUFClass { leader, s, g, c });

        out
    }

/*
    fn shrink(&mut self, i: Id, s: Set<Slot>) {
        let red = vec[i].slots \ s;
        let red = Cup {vec[i].group.orbit(x) for x in red};
        let s = vec[i].slots \ red;

        vec[i].leader = identity(s) * i;
        vec[i].slots = s;
        vec[i].group = vec[i].group.iter_generators().map(|x| x.restrict(s)).collect();
    }

    fn union(SlottedUF(v): &mut Self, mut x: AppliedId, mut y: AppliedId) {
        loop {
            x = find(x);
            y = find(y);
            shrink(x.id, slots(x.m^-1 * y.m * y.id));
            shrink(y.id, slots(y.m^-1 * x.m * x.id));
            if nothing shrunk { break }
        }

        if x.id == y.id {
            vec[x.id].group.add(x.m * y.m^-1);
        } else {
            # move y into x
            m = x.m^-1 * y.m
            vec[y].leader = m * x.id
            vec[x].group.extend(vec[y].group.iter_generators().map(|x| m*x*m^-1))
            vec[y].group = none;
        }
    }

    // we omit path compression for now.
    fn find(SlottedUF(v): &Self, mut x: AppliedId) -> AppliedId {
        loop {
            let y = vec[x.id].leader.apply_slotmap(x.m)
            if x == y { return x; }
            x = y;
        }
    }

    fn is_equal(SlottedUF(v): &Self, x: AppliedId, y: AppliedId) -> bool {
        let x = self.find(x);
        let y = self.find(y);
        if x.id != y.id { return false; }
        vec[x.id].group.contains(x.m * y.m^-1)
    }
*/
}
