use crate::*;

pub struct SlottedUF<C>(Vec<SUFClass<C>>);

pub struct SUFClass<C> {
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
        let i = Id(v.len());
        let identity = SlotMap::identity(&slots);
        let group = Group::new(&identity, Default::default());
        let out = identity * i;
        let leader = out.clone();
        v.push(SUFClass { leader, slots, group, c });

        out
    }

    fn shrink(&mut self, i: Id, s: HashSet<Slot>) {
        let class = &mut self[i];

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
        class.leader = identity * i;
    }

    // TODO add path compression later.
    fn find(&mut self, mut x: AppliedId) -> AppliedId {
        loop {
            let y = x.m() * self[x.id()].leader.clone();
            if x == y { return x; }
            x = y;
        }
    }

    fn is_equal(&mut self, x: &AppliedId, y: &AppliedId) -> bool {
        if x.id() != y.id() { return false; }
        let perm = x.m() * y.m().inverse();
        self[x.id()].group.contains(&perm)
    }

/*
    fn union(&mut self, mut a1: AppliedId, mut a2: AppliedId) {
        loop {
            a1 = self.find(a1);
            a2 = self.find(a2);
            shrink(x.id, (x.m.inverse() * y.m * y.id).slots());
            shrink(y.id, (y.m.inverse() * x.m * x.id).slots());
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
*/
}

impl<C> Index<Id> for SlottedUF<C> {
    type Output = SUFClass<C>;

    fn index(&self, Id(i): Id) -> &SUFClass<C> {
        let SlottedUF(v) = self;
        &v[i]
    }
}

impl<C> IndexMut<Id> for SlottedUF<C> {
    fn index_mut(&mut self, Id(i): Id) -> &mut SUFClass<C> {
        let SlottedUF(v) = self;
        &mut v[i]
    }
}
