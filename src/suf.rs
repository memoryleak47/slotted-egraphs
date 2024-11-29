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
    fn find(&mut self, Applied(mut m1, mut x1): AppliedId) -> AppliedId {
        loop {
            let Applied(m2, x2) = &m1 * self[x1].leader.clone();
            if x1 == x2 && m1 == m2 { return Applied(m1, x1); }
            Applied(m1, x1) = Applied(m2, x2);
        }
    }

    fn is_equal(&mut self, a1: &AppliedId, a2: &AppliedId) -> bool {
        let Applied(m1, x1) = self.find(a1.clone());
        let Applied(m2, x2) = self.find(a2.clone());
        if x1 != x2 { return false; }
        let perm = m1 * m2.inverse();
        self[x1].group.contains(&perm)
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
