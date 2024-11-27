use crate::*;

pub struct AppliedId(SlotMap, Id);

pub struct SlotMap(Vec<(Slot, Slot)>);

pub struct Slot(usize);
pub struct Id(usize);
