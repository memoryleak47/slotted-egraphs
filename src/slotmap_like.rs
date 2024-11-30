use crate::*;

pub trait SlotMapLike {
    fn map(&self, x: Slot) -> Option<Slot>;
    fn iter(&self) -> impl Iterator<Item=(Slot, Slot)>;
    fn into(self) -> SlotMap where Self: Sized { self.iter().collect() }
}

impl SlotMapLike for SlotMap {
    fn map(&self, x: Slot) -> Option<Slot> { self.get(x) }
    fn iter(&self) -> impl Iterator<Item=(Slot, Slot)> { self.iter() }
    fn into(self) -> SlotMap { self }
}
pub struct Compose<L: SlotMapLike, R: SlotMapLike>(pub L, pub R);
impl<L: SlotMapLike, R: SlotMapLike> SlotMapLike for Compose<L, R> {
    fn map(&self, x: Slot) -> Option<Slot> {
        self.0.map(self.1.map(x)?)
    }
    fn iter(&self) -> impl Iterator<Item=(Slot, Slot)> {
        self.1.iter().filter_map(|(x, y)| self.0.map(y).map(|z| (x, z)))
    }
}

impl<'a, T: SlotMapLike> SlotMapLike for &'a T {
    fn map(&self, x: Slot) -> Option<Slot> { (*self).map(x) }
    fn iter(&self) -> impl Iterator<Item=(Slot, Slot)> { (*self).iter() }
}


