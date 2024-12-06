use crate::*;

pub type Id = usize;

pub struct Suf {
    vec: Vec<Class>,
}

struct Class {
    leader: (SlotMap, Id),
    slots: HashSet<Slot>,
    group: Group,
}

impl Suf {
    pub fn new() -> Self {
        Self { vec: Vec::new() }
    }

    pub fn add(&mut self, slots: HashSet<Slot>) -> Id {
        let i = self.vec.len();
        let leader = (SlotMap::identity(&slots), i);
        let group = Group::new(slots.clone(), Default::default());
        self.vec.push(Class {
            leader,
            slots,
            group,
        });
        i
    }

    // TODO path compression
    fn find(&mut self, mut m: SlotMap, mut id: Id) -> (SlotMap, Id) {
        loop {
            let (m2, id2) = &self.vec[id].leader;
            // m * m2 * id2 == m * id
            let m2 = &m * &m2;
            if (&m, &id) == (&m2, &id2) { return (m, id); }
            (m, id) = (m2, *id2);
        }
    }

    fn is_equal(&mut self, x: Id, y: Id, x_to_y: SlotMap) -> bool {
        let (x_to_orig_y, x) = self.find(x_to_y, x);
        let (y_to_orig_y, y) = self.find(SlotMap::identity(&self.vec[y].slots), y);
        if x != y { return false; }
        self.vec[x].group.contains(&(&x_to_orig_y.inverse() * &y_to_orig_y))
    }

    // call `shink(i, s)` when `i` is equated to something with slotset `s`.
    fn shrink(&mut self, i: Id, s: &HashSet<Slot>) {
        let c = &mut self.vec[i];
        let red = &c.slots - &s;
        let red = red.iter()
                     .map(|x| c.group.orbit(*x))
                     .flatten()
                     .collect();
        let s = &c.slots - &red;
        
        c.leader = (SlotMap::identity(&s), i);
        let restrict = |m: SlotMap| -> SlotMap {
            m.iter()
             .filter(|(x, _)|
                s.contains(&x)
             )
             .collect()
        };
        let generators = c.group.generators()
                         .into_iter()
                         .map(restrict)
                         .collect();
        c.group = Group::new(s.clone(), generators);
        c.slots = s;
    }

/*
    fn union(&mut self, mut x: AppliedId, mut y: AppliedId) {
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
