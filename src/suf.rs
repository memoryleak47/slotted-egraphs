use crate::*;

pub struct SUF<C>(Vec<SUFClass<C>>);

pub struct SUFClass<C> {
    leader: AppliedId,
    slots: HashSet<Slot>,
    group: Group<SlotMap>,

    c: C,
}

impl<C> SUF<C> {
    pub fn new() -> Self {
        SUF(Vec::new())
    }

    pub fn add(&mut self, slots: HashSet<Slot>, c: C) -> AppliedId {
        let SUF(v) = self;
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
        let x = self.find(x.clone());
        let y = self.find(y.clone());
        if x.id() != y.id() { return false; }
        let perm = x.m() * y.m().inverse();
        self[x.id()].group.contains(&perm)
    }

    fn union(&mut self, mut x: AppliedId, mut y: AppliedId) {
        loop {
            x = self.find(x);
            y = self.find(y);
            let n1 = self[x.id()].slots.len() + self[y.id()].slots.len();
            self.shrink(x.id(), &x.m().inverse() * y.m() * &self[y.id()].slots);
            self.shrink(y.id(), &y.m().inverse() * x.m() * &self[x.id()].slots);
            let n2 = self[x.id()].slots.len() + self[y.id()].slots.len();
            if n1 == n2 { break; }
        }

        if x.id() == y.id() {
            self[x.id()].group.add(x.m() * y.m().inverse());
        } else {
            // move y into x
            let m: SlotMap = &x.m().inverse() * y.m().clone();
            self[y.id()].leader = m.clone() * x.id();
            let generators = self[y.id()].group
                                         .generators()
                                         .into_iter()
                                         .map(|x| &m * &x * m.inverse())
                                         .collect();
            self[x.id()].group.add_set(generators);
            // self[y.id()].group is now meaningless.
        }
    }
}

impl<C> Index<Id> for SUF<C> {
    type Output = SUFClass<C>;

    fn index(&self, Id(i): Id) -> &SUFClass<C> {
        let SUF(v) = self;
        &v[i]
    }
}

impl<C> IndexMut<Id> for SUF<C> {
    fn index_mut(&mut self, Id(i): Id) -> &mut SUFClass<C> {
        let SUF(v) = self;
        &mut v[i]
    }
}
