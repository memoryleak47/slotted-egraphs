use crate::*;

#[derive(Clone, Copy)]
pub struct Id(pub usize);
pub struct AppliedId(pub SlotMap, pub Id);
pub struct Equation(pub Id, pub Id, pub SlotMap);

