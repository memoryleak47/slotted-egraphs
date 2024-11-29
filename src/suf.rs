use crate::*;

pub struct SlottedUF<C>(Vec<SUFClass<C>>);

struct SUFClass<C> {
    leader: AppliedId,
    slots: HashSet<Slot>,
    group: Group<SlotMap>,

    c: C,
}

impl<C> SlottedUF<C> {
    pub fn new() -> Self {
        SlottedUF(Vec::new())
    }

    pub fn add(&mut self, slots: HashSet<Slot>, c: C) -> AppliedId {
        let SlottedUF(v) = self;
        let i = v.len();
        let identity = SlotMap::identity(&slots);
        let group = Group::new(&identity, Default::default());
        let out = identity * Id(i);
        let leader = out.clone();
        v.push(SUFClass { leader, slots, group, c });

        out
    }

    fn shrink(&mut self, Id(i): Id, s: HashSet<Slot>) {
        let SlottedUF(v) = self;
        let class = &mut v[i];

        let red = &class.slots - &s;

        for x in red {
            if !class.slots.contains(&x) { continue; }
            for y in class.group.orbit(x) {
                class.slots.remove(&y);
            }
        }

        let identity = SlotMap::identity(&class.slots);
        let restrict = |x: SlotMap| {
            x.iter()
             .filter(|(x, _)| class.slots.contains(&x))
             .collect()
        };
        let generators = class.group.generators()
                                    .into_iter()
                                    .map(restrict)
                                    .collect();
        class.group = Group::new(&identity, generators);
        class.leader = identity * Id(i);
    }

/*
    fn union(&mut self, mut x: AppliedId, mut y: AppliedId) {
        let SlottedUF(v) = self;
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
        let SlottedUF(v) = self;
        loop {
            let y = vec[x.id].leader.apply_slotmap(x.m)
            if x == y { return x; }
            x = y;
        }
    }

    fn is_equal(SlottedUF(v): &Self, x: AppliedId, y: AppliedId) -> bool {
        let SlottedUf(v) = self;
        let x = self.find(x);
        let y = self.find(y);
        if x.id != y.id { return false; }
        vec[x.id].group.contains(x.m * y.m^-1)
    }
*/
}
