use crate::*;
use std::fmt::*;
use std::cell::RefCell;

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
/// Slots represent Variable names.
///
/// Internally, they are just a number.
pub struct Slot(u64);

// %4 = 0 -> numeric
// %4 = 1 -> fresh
// %4 = 2 -> named
// %4 = 3 -> <unused>
struct SlotTable {
    fresh_idx: u64,
    named_vec: Vec<String>,
    named_map: HashMap<String, u64>,
}

thread_local! {
    static SLOT_TABLE: RefCell<SlotTable> = RefCell::new(SlotTable {
        fresh_idx: 1,
        named_vec: Vec::default(),
        named_map: HashMap::default(),
    });
}

impl Slot {
    /// Generates a fresh slot.
    ///
    /// Any slot returned from this function has never been constructed before.
    pub fn fresh() -> Self {
        SLOT_TABLE.with_borrow_mut(|tab| {
            let old_val = tab.fresh_idx;
            tab.fresh_idx += 4;
            Slot(old_val)
        })
    }

    /// Generates a numeric slot like `$42`
    pub fn numeric(u: u32) -> Slot {
        Slot(u as u64 * 4)
    }

    /// Generates a named slot like `$xyz`
    pub fn named(s: &str) -> Slot {
        if let Ok(x) = s.parse::<u64>() {
            return Slot(x*4); // numeric
        }

        SLOT_TABLE.with_borrow_mut(|tab| {
            if s.starts_with("f") {
                if let Ok(x) = s[1..].parse::<u64>() {
                    let out = x*4+1;
                    if tab.fresh_idx <= out {
                        tab.fresh_idx = out+4;
                    }
                    return Slot(out); // fresh
                }
            }

            if let Some(x) = tab.named_map.get(s) {
                return Slot(*x); // cached named
            }

            let i = tab.named_vec.len() as u64;
            tab.named_vec.push(s.to_string());
            Slot(4*i+2) // new named
        })
    }
}

impl Display for Slot {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let u = self.0;
        match u%4 {
            // numeric:
            0 => write!(f, "${}", u/4),

            // fresh:
            1 => write!(f, "$f{}", (u-1)/4),

            // named:
            2 => {
                let idx = ((u-2)/4) as usize;
                SLOT_TABLE.with_borrow(|tab| {
                    write!(f, "${}", tab.named_vec[idx])
                })
            }

            // unused:
            3 => unreachable!(),

            _ => unreachable!(),
        }
    }
}

impl Debug for Slot {
    // debug falls back to display.
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self)
    }
}
