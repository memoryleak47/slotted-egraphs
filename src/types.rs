use crate::*;

#[derive(Clone)]
pub struct AppliedId(SlotMap, Id);

#[derive(Clone)]
pub struct Id(usize);
