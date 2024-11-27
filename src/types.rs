use crate::*;

#[derive(Clone)]
pub struct AppliedId(SlotMap, Id);

#[derive(Clone)]
pub struct SlotMap(Vec<(Slot, Slot)>);

#[derive(Clone)]
pub struct Slot(usize);

#[derive(Clone)]
pub struct Id(usize);
