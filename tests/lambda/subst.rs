use crate::*;

// returns an eclass containing b[x := t]
// out has slots (slots(b) - {x}) | slots(t).
// I presume that slots(t) is allowed to contain x.
pub fn subst(b: AppliedId, x: Slot, t: AppliedId, eg: &mut EGraph<Lambda>) -> AppliedId {
    let b = eg.find_applied_id(&b);
    let t = eg.find_applied_id(&t);

    // Is it necessary to do the unions in the end? Can it not be done "directly"?
    let mut union_cmds = vec![];
    let out = subst_impl(b, x, t, eg, &mut union_cmds, &mut Default::default());

    for (x, y) in union_cmds {
        eg.union(&x, &y);
    }

    let out = eg.find_applied_id(&out);
    out
}

// TODO do I need this? If yes, why?
fn subst_impl(b: AppliedId, x: Slot, t: AppliedId, eg: &mut EGraph<Lambda>, union_cmds: &mut Vec<(AppliedId, AppliedId)>, map: &mut Map) -> AppliedId {
    let large = &(&b.slots() | &t.slots()) | &singleton_set(x);

    // m :: Fresh -> Large
    let m = SlotMap::bijection_from_fresh_to(&large);

    // m_inv :: Large -> Fresh
    let m_inv = m.inverse();

    // make everything "fresh".
    let b = b.apply_slotmap(&m_inv);
    let x = m_inv[x];
    let t = t.apply_slotmap(&m_inv);

    let out = subst_impl2(b, x, t, eg, union_cmds, map);

    // convert back from Fresh to Large.
    out.apply_slotmap(&m)
}

fn subst_impl2(b: AppliedId, x: Slot, t: AppliedId, eg: &mut EGraph<Lambda>, union_cmds: &mut Vec<(AppliedId, AppliedId)>, map: &mut Map) -> AppliedId {
    if !b.slots().contains(&x) { // trivial-substitution-check.
        return b;
    }

    let new_b = match map_lookup(&b, x, &t, eg, map) {
        Ok(map_b) => return map_b,
        Err(new_b) => new_b,
    };

    for enode in eg.enodes_applied(&b) {
        let app_id = enode_subst(enode, &b, x, &t, eg, union_cmds, map);
        union_cmds.push((new_b.clone(), app_id));
    }

    new_b
}

// `enode` is an enode from the eclass `b`.
//
// enode.slots() is superset of slots(b).
// b.slots() - {x} might intersect t.slots(), this represents the bt_relation.
//
// x in b.slots(), and x is never part of the bt_relation.
//
// we return an eclass containing `enode[x := t]`
//
// The resulting AppliedId has slots "(slots(enode) - {x}) | slots(t)"
fn enode_subst(enode: Lambda, _b: &AppliedId, x: Slot, t: &AppliedId, eg: &mut EGraph<Lambda>, union_cmds: &mut Vec<(AppliedId, AppliedId)>, map: &mut Map) -> AppliedId {
    let out = match enode.clone() {
        Lambda::Var(x2) => {
            // We know that b.slots().contains(x) as if would otherwise have been filtered out in the trivial-substitution-check.
            // Thus enode.slots().contains(x), as its a superset of b.slots().
            // And as enode.slots() == {x2}, we know x == x2.
            assert_eq!(x, x2);

            t.clone()
        },

        Lambda::App(l, r) => {
            let mut call = |a: AppliedId| -> AppliedId {
                // X := (slots(b) - {x}) | slots(t)
                // a.m :: slots(a.id) -> X
                subst_impl(a.clone(), x, t.clone(), eg, union_cmds, map)
            };
            let l = call(l);
            let r = call(r);

            eg.add(Lambda::App(l, r))
        },

        Lambda::Lam(x2, b2) => {
            assert!(x2 != x);

            let b2 = subst_impl(b2.clone(), x, t.clone(), eg, union_cmds, map);
            eg.add(Lambda::Lam(x2, b2))
        },

        Lambda::Lambda(..) => panic!("This should never encounter a let!"),
    };

    let correct = &(&enode.slots() - &singleton_set(x)) | &t.slots();
    assert_eq!(out.slots(), correct);

    out
}

/////////////// Map impl ///////////////

#[derive(PartialEq, Eq, Hash, Debug, Clone, PartialOrd, Ord)]
struct Key {
    b: Id,
    x: Slot,

    // (w1, w2) in bt_relation, iff w1 in slots(b) && w2 in slots(t), and
    // if w1 and w2 correspond to the same slot in this subst instance.
    // Note that b and t are Ids in this context.
    bt_relation: SlotMap,
}

#[derive(Clone)]
struct Value {
    // the returned Id
    out_id: Id,

    // b: Id
    // maps (slots(b) - {x}) -> slots(out_id)
    b_map: SlotMap,

    // t: Id
    // maps slots(t) -> slots(out_id)
    t_map: SlotMap,

    // b_map and t_map need to be consistent with the bt_relation:
    // if (w1, w2) in bt_relation, then b_map[w1] == t_map[w2].
}

type Map = HashMap<Key, Value>;

// Ok(app_id) means that it was already in the map, and nothing needs to be done.
// Err(app_id) means that it was not yet in the map, but a new entry was added for it. Go and union stuff to it!
// Either way slots(app_id) == (slots(b) - {x}) | slots(t).
fn map_lookup(b: &AppliedId, x: Slot, t: &AppliedId, eg: &mut EGraph<Lambda>, map: &mut Map) -> Result<AppliedId, AppliedId> {
    assert!(b.slots().contains(&x));

    // b.m :: slots(b.id) -> X
    // t.m :: slots(t.id) -> X

    // bt_relation :: slots(b.id) -> slots(t.id)
    let bt_relation = b.m.compose_partial(&t.m.inverse());

    // x :: X
    // real_x :: slots(b.id)
    let real_x = b.m.inverse()[x];

    let key = Key {
        b: b.id,
        x: real_x,
        bt_relation: bt_relation.clone(),
    };

    let new_class = !map.contains_key(&key);

    // add to map, if necessary
    if new_class {
        // max_slots = X
        let max_slots = &(&b.slots() - &singleton_set(x)) | &t.slots();
        let fresh = SlotMap::bijection_from_fresh_to(&max_slots);
        let slots = fresh.keys();

        let new_b = eg.alloc_empty_eclass(&slots);

        // fresh :: slots(new_b) -> X
        // fresh_inv :: X -> slots(new_b)
        let fresh_inv = fresh.inverse();

        // t_map :: slots(t.id) -> slots(new_b)
        let t_map = t.m.compose_partial(&fresh_inv);

        // b_map :: slots(b.id) -> slots(new_b)
        let b_map = b.m.compose_partial(&fresh_inv);

        let v = Value {
            out_id: new_b,
            t_map,
            b_map,
        };

        map.insert(key.clone(), v);
    }

    let v = map.get(&key).unwrap();
    // v.t_map :: slots(t.id) -> slots(v.out_id)
    // v.b_map :: slots(b.id) -> slots(v.out_id)

    // s_b :: slots(v.out_id) -> slots(b)
    let s_b: SlotMap = v.b_map.inverse().compose_partial(&b.m);

    // s_t :: slots(v.out_id) -> slots(t)
    let s_t: SlotMap = v.t_map.inverse().compose_partial(&t.m);

    // s_res :: slots(v.out_id) -> X
    let s_res = s_b.union(&s_t);

    let app_id = AppliedId::new(v.out_id, s_res);

    match new_class {
        true => Err(app_id),
        false => Ok(app_id),
    }
}
