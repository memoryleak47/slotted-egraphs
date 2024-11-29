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
        let out = AppliedId { t: Id(i), m: identity };
        let leader = out.clone();
        v.push(SUFClass { leader, s, g, c });

        out
    }

    pub fn union(&mut self, x: &AppliedId, y: &AppliedId) {
        todo!()
    }

    pub fn is_equal(&self, x: &AppliedId, y: &AppliedId) {
        todo!()
    }
}
