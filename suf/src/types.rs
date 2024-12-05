use crate::*;

pub struct Id(usize);
pub struct AppliedId(SlotMap, Id);
pub struct Equation(Id, Id, SlotMap);

