use crate::*;

pub type Id = usize;

pub struct Suf {
    vec: Vec<Class>,
}

struct Class {
    leader: (SlotMap, Id),
    group: Group,
}

impl Suf {
    pub fn new() -> Self {
        Self { vec: Vec::new() }
    }

    pub fn add(&mut self, slots: HashSet<Slot>) -> Id {
        let i = self.vec.len();
        let leader = (SlotMap::identity(&slots), i);
        let group = Group::new(slots, Default::default());
        self.vec.push(Class {
            leader,
            group,
        });
        i
    }

    fn find_id(&mut self, id: Id) -> (SlotMap, Id) {
        let (m2, id2) = self.vec[id].leader.clone();
        if id == id2 {
            return (m2, id2);
        } else {
            let (m3, id3) = self.find_id(id2);
            let out = (&m2 * &m3, id3);
            self.vec[id].leader = out.clone();
            return out;
        }
    }

    fn find(&mut self, m: &SlotMap, id: Id) -> (SlotMap, Id) {
        let (m2, id2) = self.find_id(id);
        (m * &m2, id2)
    }

    fn is_equal(&mut self, x: Id, y: Id, x_to_y: SlotMap) -> bool {
        let (x_to_orig_y, x) = self.find(&x_to_y, x);
        let (y_to_orig_y, y) = self.find_id(y);
        if x != y { return false; }
        self.vec[x].group.contains(&(&x_to_orig_y.inverse() * &y_to_orig_y))
    }

    // call `shrink(i, s)` when the leader `i` is equated to something with slotset `s`.
    // returns whether something actually happened.
    fn shrink(&mut self, i: Id, s: &HashSet<Slot>) -> bool {
        let c = &mut self.vec[i];
        let red = &c.group.omega - &s;
        if red.is_empty() { return false; }
        let red = red.iter()
                     .map(|x| c.group.orbit(*x))
                     .flatten()
                     .collect();
        let s = &c.group.omega - &red;

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
        c.group = Group::new(s, generators);

        true
    }

    fn union(&mut self, x: Id, y: Id, x_to_y: &SlotMap) {
        let mut x = x;
        let mut y = y;
        let mut x_to_y = x_to_y.clone();
        loop {
            let (mx2, x2) = self.find(&x_to_y, x);
            let (my2, y2) = self.find_id(y);
            let mx2_inv = mx2.inverse();
            let my2_inv = my2.inverse();

            // mx2 * x2 == my2 * y2
            // -> x2 == mx2^-1 * my2 * y2
            // -> y2 == my2^-1 * mx2 * x2

            let x_set = self.vec[y2].group.omega
                    .iter()
                    .filter_map(|a| my2.get(*a).and_then(|a| mx2_inv.get(a)))
                    .collect();
            let x_delta = self.shrink(x2, &x_set);

            let y_set = self.vec[x2].group.omega
                    .iter()
                    .filter_map(|a| mx2.get(*a).and_then(|a| my2_inv.get(a)))
                    .collect();
            let y_delta = self.shrink(y2, &y_set);

            if !x_delta && !y_delta { break }

            (x, y, x_to_y) = (x2, y2, &my2_inv * &mx2);
        }

        if x == y {
            self.vec[x].group.add(x_to_y);
        } else {
            // merge x into y
            let translate = |m: &SlotMap| -> SlotMap {
                m.iter()
                 .map(|(a, b)| (x_to_y[a], x_to_y[b]))
                 .collect()
            };
            let generators = self.vec[x].group.generators()
                 .iter()
                 .map(translate)
                 .collect();
            self.vec[y].group.add_set(generators);
            self.vec[x].leader = (x_to_y, y);
        }
    }
}
