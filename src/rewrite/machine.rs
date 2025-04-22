use crate::*;
use std::result;

type Result = result::Result<(), ()>;

type Var = String;

#[derive(Default)]
struct Machine {
    reg: Vec<AppliedId>,
    // a buffer to re-use for lookups
    lookup: Vec<AppliedId>,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Reg(u32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program<L> {
    instructions: Vec<Instruction<L>>,
    subst: Subst,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Instruction<L> {
    Bind { node: L, i: Reg, out: Reg },
    Compare { i: Reg, j: Reg },
    Lookup { term: Vec<ENodeOrReg<L>>, i: Reg },
    Scan { out: Reg },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ENodeOrReg<L> {
    ENode(L),
    Reg(Reg),
}

impl Machine {
    #[inline(always)]
    fn reg(&self, reg: Reg) -> AppliedId {
        self.reg[reg.0 as usize].clone()
    }

    fn run<L, N>(
        &mut self,
        egraph: &EGraph<L, N>,
        instructions: &[Instruction<L>],
        subst: &Subst,
        yield_fn: &mut impl FnMut(&Self, &Subst) -> Result,
    ) -> Result
    where
        L: Language,
        N: Analysis<L>,
    {
        let mut instructions = instructions.iter();
        while let Some(instruction) = instructions.next() {
            match instruction {
                Instruction::Bind { i, out, node } => {
                    let remaining_instructions = instructions.as_slice();
                    // dbg!(self.reg(*i));
                    // panic!()
                    // let eclass = egraph[self.reg(*i)];
                    // let eclass = egraph.classes.get(&self.reg(*i).id);
                    // let mut out = Vec::new();
                    for n in egraph.enodes_applied(&self.reg(*i)) {
                        if std::mem::discriminant(node) != std::mem::discriminant(&n) {
                            continue;
                        };
                        // egraph.get_group_compatible_variants(&n).iter().for_each(|id|);

                        // dbg!(n.applied_id_occurrences());
                        // self.reg.extend(n.applied_id_occurrences());
                        // n.applied_id_occurrences()
                        //     .iter()
                        //     .for_each(|aid| self.reg.push(aid.clone()));
                        self.reg.truncate(out.0 as usize);
                        for aid in n.applied_id_occurrences() {
                            self.reg.push(aid.clone());
                        }
                        self.run(egraph, remaining_instructions, subst, yield_fn)?;

                        // ematch_node(&st, eg, &n, children, &mut out, &nn);
                    }
                    return Ok(());

                    // return eclass.for_each_matching_node(node, |matched| {
                    //     self.reg.truncate(out.0 as usize);
                    //     matched.for_each(|id| self.reg.push(id));
                    //     self.run(egraph, remaining_instructions, subst, yield_fn)
                    // });
                }
                Instruction::Scan { out } => {
                    panic!("only necessary for multipatterns which we don't have?");
                    // let remaining_instructions = instructions.as_slice();
                    // for class in egraph.classes() {
                    //     self.reg.truncate(out.0 as usize);
                    //     self.reg.push(class.id);
                    //     self.run(egraph, remaining_instructions, subst, yield_fn)?
                    // }
                    // return Ok(());
                }
                Instruction::Compare { i, j } => {
                    panic!();
                    // if egraph.find(self.reg(*i)) != egraph.find(self.reg(*j)) {
                    //     return Ok(());
                    // }
                }
                // verify that a specific pattern described by term exists in the e-graph and is equivalent to the e-class represented by register i.
                Instruction::Lookup { term, i } => {
                    self.lookup.clear();
                    for node in term {
                        match node {
                            ENodeOrReg::ENode(node) => {
                                // let look = |i| self.lookup[usize::from(i)];
                                panic!()
                                // match egraph.lookup(node.clone().map_children(look)) {
                                //     Some(id) => self.lookup.push(id),
                                //     None => return Ok(()),
                                // }
                            }
                            ENodeOrReg::Reg(r) => {
                                // self.lookup.push(egraph.find(self.reg(*r)));
                                panic!()
                            }
                        }
                    }

                    // let id = egraph.find(self.reg(*i));
                    // if self.lookup.last().copied() != Some(id) {
                    //     return Ok(());
                    // }
                    panic!()
                }
            }
        }

        yield_fn(self, subst)
    }
}

struct Compiler<L> {
    v2r: HashMap<Var, Reg>,
    free_vars: Vec<HashSet<Var>>,
    subtree_size: Vec<usize>,
    todo_nodes: HashMap<(AppliedId, Reg), L>,
    instructions: Vec<Instruction<L>>,
    next_reg: Reg,
}

impl<L: Language> Compiler<L> {
    fn new() -> Self {
        Self {
            free_vars: Default::default(),
            subtree_size: Default::default(),
            v2r: Default::default(),
            todo_nodes: Default::default(),
            instructions: Default::default(),
            next_reg: Reg(0),
        }
    }

    // id either refers to the root of the pattern, or all children.
    fn add_todo(&mut self, pattern: &PatternAstFlat<L>, id: AppliedId, reg: Reg) {
        match &pattern[id.clone()] {
            ENodeOrVar::Var(v) => {
                if let Some(&j) = self.v2r.get(v) {
                    self.instructions.push(Instruction::Compare { i: reg, j })
                } else {
                    self.v2r.insert(v.clone(), reg);
                }
            }
            ENodeOrVar::ENode(pat) => {
                self.todo_nodes.insert((id, reg), pat.clone());
            }
        }
    }

    fn load_pattern(&mut self, pattern: &PatternAstFlat<L>) {
        let len = pattern.len();
        self.free_vars = Vec::with_capacity(len);
        self.subtree_size = Vec::with_capacity(len);

        for node in &pattern.nodes {
            let mut free = HashSet::default();
            let mut size = 0;
            match node {
                ENodeOrVar::ENode(n) => {
                    size = 1;
                    for child in n.applied_id_occurrences() {
                        let id = child.id;
                        free.extend(self.free_vars[id.0].clone());
                        size += self.subtree_size[id.0];
                    }
                }
                ENodeOrVar::Var(v) => {
                    free.insert(v.clone());
                }
            }
            self.free_vars.push(free);
            self.subtree_size.push(size);
        }
    }

    fn next(&mut self) -> Option<((AppliedId, Reg), L)> {
        // we take the max todo according to this key
        // - prefer grounded
        // - prefer more free variables
        // - prefer smaller term
        let key = |(id, _): &&(AppliedId, Reg)| {
            let i = id.id.0;
            let n_bound = self.free_vars[i]
                .iter()
                .filter(|v| self.v2r.contains_key(*v))
                .count();
            let n_free = self.free_vars[i].len() - n_bound;
            let size = self.subtree_size[i] as isize;
            (n_free == 0, n_free, -size)
        };

        self.todo_nodes
            .keys()
            .max_by_key(key)
            .cloned()
            .map(|k| (k.clone(), self.todo_nodes.remove(&k).unwrap()))
    }

    /// check to see if this e-node corresponds to a term that is grounded by
    /// the variables bound at this point
    fn is_ground_now(&self, id: AppliedId) -> bool {
        self.free_vars[id.id.0]
            .iter()
            .all(|v| self.v2r.contains_key(v))
    }

    fn compile(&mut self, patternbinder: Option<Var>, pattern: &PatternAstFlat<L>) {
        self.load_pattern(pattern);
        let root = pattern.root();

        let mut next_out = self.next_reg;

        // Check if patternbinder already bound in v2r
        // Behavior common to creating a new pattern
        let add_new_pattern = |comp: &mut Compiler<L>| {
            if !comp.instructions.is_empty() {
                // After first pattern needs scan
                comp.instructions
                    .push(Instruction::Scan { out: comp.next_reg });
            }
            comp.add_todo(pattern, root.clone(), comp.next_reg);
        };

        if let Some(v) = patternbinder {
            if let Some(&i) = self.v2r.get(&v) {
                // patternbinder already bound
                self.add_todo(pattern, root, i);
            } else {
                // patternbinder is new variable
                next_out.0 += 1;
                add_new_pattern(self);
                self.v2r.insert(v, self.next_reg); //add to known variables.
            }
        } else {
            // No pattern binder
            next_out.0 += 1;
            add_new_pattern(self);
        }

        while let Some(((id, reg), node)) = self.next() {
            if self.is_ground_now(id.clone()) && !node.is_leaf() {
                let extracted = pattern.extract(id);
                self.instructions.push(Instruction::Lookup {
                    i: reg,

                    term: extracted
                        .into_iter()
                        .map(|n| match n {
                            ENodeOrVar::ENode(n) => ENodeOrReg::ENode(n.clone()),
                            ENodeOrVar::Var(v) => ENodeOrReg::Reg(self.v2r[&v]),
                        })
                        .collect(),
                });
            } else {
                let out = next_out;
                next_out.0 += node.applied_id_occurrences().len() as u32;

                // zero out the children so Bind can use it to sort
                let mut op = node.clone(); //.map_children(|_| AppliedId::from(0));
                op.applied_id_occurrences_mut()
                    .iter_mut()
                    .for_each(|aid| aid.id = Id(0));
                self.instructions.push(Instruction::Bind {
                    i: reg,
                    node: op,
                    out,
                });

                for (i, &child) in node.applied_id_occurrences().iter().enumerate() {
                    self.add_todo(pattern, child.clone(), Reg(out.0 + i as u32));
                }
            }
        }
        self.next_reg = next_out;
    }

    fn extract(self) -> Program<L> {
        let mut subst = Subst::default();
        for (v, r) in self.v2r {
            subst.insert(
                v,
                AppliedId {
                    id: Id(r.0 as usize),
                    m: SlotMap::new(),
                },
            );
        }
        Program {
            instructions: self.instructions,
            subst,
        }
    }
}

impl<L: Language> Program<L> {
    pub fn compile_from_pat(pattern: &PatternAstFlat<L>) -> Self {
        let mut compiler = Compiler::new();
        compiler.compile(None, pattern);
        let program = compiler.extract();
        // log::debug!("Compiled {:?} to {:?}", pattern.as_ref(), program);
        program
    }

    // pub(crate) fn compile_from_multi_pat(patterns: &[(Var, PatternAstFlat<L>)]) -> Self {
    //     let mut compiler = Compiler::new();
    //     for (var, pattern) in patterns {
    //         compiler.compile(Some(*var), pattern);
    //     }
    //     compiler.extract()
    // }

    pub fn run_with_limit<A>(
        &self,
        egraph: &EGraph<L, A>,
        eclass: AppliedId,
        mut limit: usize,
    ) -> Vec<Subst>
    where
        A: Analysis<L>,
    {
        // assert!(egraph.clean, "Tried to search a dirty e-graph!");

        if limit == 0 {
            return vec![];
        }

        let mut machine = Machine::default();
        assert_eq!(machine.reg.len(), 0);
        machine.reg.push(eclass);

        let mut matches = Vec::new();
        machine
            .run(
                egraph,
                &self.instructions,
                &self.subst,
                &mut |machine, subst| {
                    // if !egraph.analysis.allow_ematching_cycles() {
                    //     if let Some((first, rest)) = machine.reg.split_first() {
                    //         if rest.contains(first) {
                    //             return Ok(());
                    //         }
                    //     }
                    // }
                    dbg!(subst);
                    // panic!();

                    // let subst_vec = subst
                    //     .vec
                    //     .iter()
                    //     // HACK we are reusing AppliedIds here, this is bad
                    //     .map(|(v, reg_id)| (*v, machine.reg(Reg(usize::from(*reg_id) as u32))))
                    //     .collect();
                    // TODO
                    let subst = subst
                        .iter()
                        .map(|(s, aid)| (s.clone(), machine.reg(Reg(aid.id.0 as u32))))
                        .collect();

                    dbg!(subst);
                    panic!();

                    // matches.push(Subst { vec: subst_vec });
                    matches.push(subst);

                    limit -= 1;
                    if limit != 0 {
                        Ok(())
                    } else {
                        Err(())
                    }
                },
            )
            .unwrap_or_default();

        // log::trace!("Ran program, found {:?}", matches);
        matches
    }
}

pub fn machine_ematch_all<L: Language, N: Analysis<L>>(
    eg: &EGraph<L, N>,
    pattern: &PatternAst<L>,
) -> Vec<Subst> {
    let pattern_flat = pattern_ast_to_flat(pattern);
    let program = Program::compile_from_pat(&pattern_flat);
    let mut out = Vec::new();
    for i in eg.ids() {
        let i = eg.mk_sem_identity_applied_id(i);
        out.extend(
            program.run_with_limit(eg, i, 1000).into_iter(), // .map(final_subst),
        );
    }
    out
}
