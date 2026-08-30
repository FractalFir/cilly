//! This module implements a relatively simple structurization algorithm, with a few goals.
//! Besides simplicty, an important goal for the algorithm is it's resistance to irreducible
//! control flow: it is supposed to reduce the control flow(to strurcutred form) as much as
//! possible, falling back to switch-emulation ONLY in places where that is absoultely necessary.
//!
//! We do this, by recursively structurizing regions: a structured if may be
//! a "node" of an irreducible CFG, and also contain another, disjoint irrudicible CFG as
//! one of it's arms.
//!
use std::collections::{HashMap, HashSet};
#[cfg(test)]
use std::sync::atomic::AtomicUsize;

use petgraph::{
    Direction,
    algo::dominators::Dominators,
    graph::{DiGraph, NodeIndex},
};

use crate::{
    BasicBlock, Body, I1_TY, InstrList, Instruction, Label, Locals,
    Operand::{self, SSA},
    SSAVal, Termiantor, Type,
};

mod fallback;
pub(crate) fn to_body(
    bbs: Vec<BasicBlock>,
    locals: &mut Locals,
    sass: &mut Vec<Type>,
    ret_ty: &Type,
) -> Body {
    // 1st stage of the algorithm: convert a block list to a StructureRegion.
    let entry = Label { id: 0 };
    // "Virtual" exit node. This ID is reserved, and impossible to reach under normal cicrumstances.
    // It is needed to represent the idea that returns / infinite loops all have a "postdominator".
    let exit = Label { id: u32::MAX };
    let mut unstructured = bbs
        .into_iter()
        .enumerate()
        .map(|(id, bb)| {
            (
                Label { id: id as u32 },
                (StructureRegion::OpSeq(bb.instrs), bb.term.unwrap()),
            )
        })
        .collect();
    // First, we do a DCE pass to remove dead blocks.
    dce(entry, &mut unstructured);
    // then, we find linear regions and flatten em
    linearize(entry, &mut unstructured);
    // Simple cleanup for degenerate, but valid cases
    simplyfy_terms(&mut unstructured);
    let mut res = StructureRegion::Unstructured {
        entry,
        unstructured,
        exit,
    };
    // Then, we run the region-based structurizer on the StructureRegion.
    res.structurize(locals, sass, ret_ty);
    // And turn it to a body, inserting fallbacks if need be.
    res.to_body(locals, sass, ret_ty)
}
#[derive(Clone, Debug)]
enum StructureRegion {
    OpSeq(InstrList),
    Unstructured {
        entry: Label,
        unstructured: HashMap<Label, (Self, Termiantor)>,
        exit: Label,
    },
    Seq(Vec<Self>),
    If {
        cond: crate::Operand,
        then: Box<StructureRegion>,
        label: Label,
    },
}
impl StructureRegion {
    fn structurize(&mut self, locals: &mut Locals, ssas: &mut Vec<Type>, ret_ty: &Type) {
        let (entry, unstructured, exit) = match self {
            StructureRegion::OpSeq(_) => return (),
            StructureRegion::Seq(seq) => {
                return seq
                    .iter_mut()
                    .for_each(|s| s.structurize(locals, ssas, ret_ty));
            }
            StructureRegion::Unstructured {
                entry,
                unstructured,
                exit,
            } => (entry, unstructured, exit),
            StructureRegion::If { then, .. } => return then.structurize(locals, ssas, ret_ty),
        };

        'outer: loop {
            let graph = unstructured_to_digraph(*entry, &*unstructured, *exit);
            /*
            Petgraph pre and post dominators?
            */
            for (l, (bb, term)) in unstructured.clone() {
                let Termiantor::BrCond { cond, then, els } = term else {
                    continue;
                };
                // If - all cf that passes trough then must go back to els
                if graph.post_dominates(graph.ids[&els], graph.ids[&then]) {
                    if try_reduce_if(&graph, l, then, els, &cond, unstructured, false, ssas) {
                        continue 'outer;
                    }
                }
                if graph.post_dominates(graph.ids[&then], graph.ids[&els]) {
                    if try_reduce_if(&graph, l, els, then, &cond, unstructured, true, ssas) {
                        continue 'outer;
                    }
                }
            }
            break;
        }
    }
}
fn try_reduce_if(
    graph: &Graph,
    l: Label,
    then: Label,
    els: Label,
    cond: &Operand,
    unstructured: &mut HashMap<Label, (StructureRegion, Termiantor)>,
    invert_cond: bool,
    ssa: &mut Vec<Type>,
) -> bool {
    // if (condtion) goto then;
    // else goto els;
    // then:
    // do_sth();
    // els:
    // === simple ===
    // if (condtion)do_sth();
    let then_set = graph.nodes_between_nodes(graph.ids[&then], graph.ids[&els]);
    if !then_set
        .iter()
        .all(|then| graph.dominates(graph.ids[&l], *then))
    {
        // Not all blocks in the region dominated by the "if head" - not an if :<
        return false;
    }
    // Branch dominates itself - cheap loop check
    if then_set.contains(&graph.ids[&l]) {
        return false;
    }
    // Proper loop check
    let closed = then_set.iter().all(|&n| {
        graph
            .graph
            .neighbors_directed(n, Direction::Incoming)
            .all(|p| p == graph.ids[&l] || then_set.contains(&p))
    });
    if !closed {
        return false;
    }
    let mut inner: HashMap<_, _> = Default::default();
    for t in then_set {
        inner.insert(
            graph.graph[t],
            unstructured.remove(&graph.graph[t]).unwrap(),
        );
    }
    let inner = StructureRegion::Unstructured {
        entry: then,
        unstructured: inner,
        exit: els,
    };
    let new = if invert_cond {
        let ncond = SSAVal(ssa.len() as _);
        let flip = Instruction::Binop {
            dst: ncond,
            ty: I1_TY.clone(),
            lhs: cond.clone(),
            rhs: Operand::Constant(crate::Constant::True),
            op: crate::Binop::Xor,
        };
        let inner = StructureRegion::If {
            cond: Operand::SSA(ncond),
            then: Box::new(inner),
            label: l,
        };
        let (old_str, _) = unstructured.remove(&l).unwrap();
        StructureRegion::Seq(vec![
            old_str,
            StructureRegion::OpSeq(InstrList { instrs: vec![flip] }),
            inner,
        ])
    } else {
        let inner = StructureRegion::If {
            cond: cond.clone(),
            then: Box::new(inner),
            label: l,
        };
        let (old_str, _) = unstructured.remove(&l).unwrap();
        StructureRegion::Seq(vec![old_str, inner])
    };

    unstructured.insert(l, (new, Termiantor::Br(els)));
    true
}
struct Graph {
    entry: Label,
    graph: DiGraph<Label, ()>,
    ids: HashMap<Label, NodeIndex>,
    doms: Dominators<NodeIndex>,
    post_doms: Dominators<NodeIndex>,
}
impl Graph {
    fn dominates(&self, a: NodeIndex, b: NodeIndex) -> bool {
        self.doms
            .dominators(b)
            .into_iter()
            .flatten()
            .any(|d| d == a)
    }
    fn post_dominates(&self, a: NodeIndex, b: NodeIndex) -> bool {
        self.post_doms
            .dominators(b)
            .into_iter()
            .flatten()
            .any(|d| d == a)
    }
    fn bad(&self, start: NodeIndex, end: NodeIndex) -> HashSet<NodeIndex> {
        let doms: HashSet<NodeIndex> = self.doms.dominators(end).into_iter().flatten().collect();
        let post_doms: HashSet<NodeIndex> = self
            .post_doms
            .dominators(start)
            .into_iter()
            .flatten()
            .collect();
        eprintln!("{doms:?} {post_doms:?}");
        doms.intersection(&post_doms).copied().collect()
    }
    fn nodes_between_nodes(&self, start: NodeIndex, end: NodeIndex) -> HashSet<NodeIndex> {
        let mut seen = HashSet::new();
        let mut stack = vec![start];
        while let Some(n) = stack.pop() {
            if n == end || !seen.insert(n) {
                continue;
            }
            stack.extend(self.graph.neighbors_directed(n, Direction::Outgoing));
        }
        seen
    }
}
fn unstructured_to_digraph(
    entry: Label,
    unstructured: &HashMap<Label, (StructureRegion, Termiantor)>,
    exit: Label,
) -> Graph {
    let mut graph = DiGraph::new();
    let mut ids: HashMap<Label, NodeIndex> = HashMap::with_capacity(unstructured.len() + 1);
    for label in unstructured.keys() {
        ids.insert(*label, graph.add_node(*label));
    }
    let exit_idx = *ids.entry(exit).or_insert_with(|| graph.add_node(exit));
    for (label, (_, term)) in unstructured {
        let from = ids[label];
        let succ = term.sucessors();
        // divergent control flow connects to exit!
        if succ.is_empty() {
            graph.add_edge(from, exit_idx, ());
        }
        for succ in succ {
            graph.add_edge(from, ids[&succ], ());
        }
    }
    // Handling of infinite loops - they diverge too(in a wierd way).
    // how do we detect that? Well, in our graph, nodes with no connection to the
    // exit must naturally be infinite loops.
    let gclone = graph.clone();
    let rev = petgraph::visit::Reversed(&gclone);
    let mut reaches_exit = std::collections::HashSet::new();
    let mut dfs = petgraph::visit::Dfs::new(&rev, ids[&exit]);
    while let Some(n) = dfs.next(&rev) {
        reaches_exit.insert(n);
    }
    for node in graph.node_indices().filter(|n| !reaches_exit.contains(n)) {
        graph.add_edge(node, exit_idx, ());
    }
    let doms = petgraph::algo::dominators::simple_fast(&graph, ids[&entry]);
    let mut rgraph = graph.clone();
    rgraph.reverse();
    let post_doms = petgraph::algo::dominators::simple_fast(&rgraph, ids[&exit]);
    Graph {
        graph,
        ids,
        entry,
        doms,
        post_doms,
    }
}
#[cfg(test)]
fn structurize_random_cfg(u: &mut arbitrary::Unstructured) -> arbitrary::Result<()> {
    use crate::{
        BasicBlock, Constant, I1_TY, InstrList, Label, Locals, Operand, SSAVal, Termiantor, Type,
    };
    let count = u.arbitrary::<u8>()?.saturating_add(2);
    let mut cases = vec![];
    u.arbitrary_loop(Some(2), Some(count as u32), |u| {
        let res = match u.int_in_range(0..=100)? {
            0..=50 => vec![
                u.int_in_range(1..=count - 1)?,
                u.int_in_range(1..=count - 1)?,
            ],
            51..=75 => vec![u.int_in_range(1..=count - 1)?],
            76..=100 => vec![],
            _ => todo!(),
        };
        cases.push(res);
        Ok(std::ops::ControlFlow::Continue(()))
    })?;
    // and then we normalize them
    let clen = cases.len();
    cases
        .iter_mut()
        .map(|v| v.iter_mut())
        .flatten()
        .for_each(|c| *c = (*c % (clen as u8)).max(1));
    // Next, we make sure the func does not terminate toooo early.
    let mut curr = 0;
    for _ in 0..clen {
        let curr_cases = &mut cases[curr];
        match &curr_cases[..] {
            [] => {
                let next = (u.arbitrary::<u8>()? % (clen as u8)).max(1);
                curr_cases.push(next);
                curr = next as usize;
            }
            [next] => curr = *next as usize,
            _ => break,
        }
    }
    let mut args = vec![];
    let bbs = cases
        .iter()
        .enumerate()
        .map(|(idx, branches)| {
            let term = match &branches[..] {
                [] => Termiantor::Ret(Operand::Constant(Constant::Int(idx as i128))),
                [case] => Termiantor::Br(Label { id: *case as u32 }),
                [then, els] => {
                    let cond = Operand::SSA(SSAVal(args.len() as _));
                    args.push(I1_TY.clone());
                    Termiantor::BrCond {
                        cond: cond,
                        then: Label { id: *then as _ },
                        els: Label { id: *els as _ },
                    }
                }
                _ => unreachable!(),
            };
            let instrs = vec![];
            BasicBlock {
                instrs: InstrList { instrs },
                term: Some(term),
            }
        })
        .collect::<Vec<_>>();
    let body = to_body(
        bbs,
        &mut Locals::empty(),
        &mut args,
        &Type::ix(std::num::NonZeroU8::new(8).unwrap()),
    );
    eprintln!("===============");
    eprintln!("{body}");
    let _ = body;
    Ok(())
}
fn dce(entry: Label, unstructured: &mut HashMap<Label, (StructureRegion, Termiantor)>) {
    let mut reachable = std::collections::HashSet::new();
    let mut stack = vec![entry];
    while let Some(label) = stack.pop() {
        if !reachable.insert(label) {
            continue;
        }
        let Some((_, term)) = unstructured.get(&label) else {
            continue;
        };
        stack.extend(term.sucessors());
    }
    unstructured.retain(|label, _| reachable.contains(label));
}
fn simplyfy_terms(unstructured: &mut HashMap<Label, (StructureRegion, Termiantor)>) {
    for (_, (_, t)) in unstructured {
        let Termiantor::BrCond { then, els, .. } = t else {
            continue;
        };
        if then == els {
            *t = Termiantor::Br(*then);
        }
    }
}
fn linearize(entry: Label, unstructured: &mut HashMap<Label, (StructureRegion, Termiantor)>) {
    // the idea behind the linearize pass is this: control flow can contain "linear" regions,
    // where a block has only one, unconditonal, predecesor. In such case, we can copy the body
    // of that block, append it to the previous block, and terminate it with the terminator of the successor.
    let mut pred_count: HashMap<Label, usize> = Default::default();
    // for each bb, add it's sucss to the counter.
    unstructured
        .iter()
        .map(|(_, (_, t))| t.sucessors())
        .flatten()
        .for_each(|s| {
            *pred_count.entry(s).or_default() += 1;
            assert_ne!(s, entry, "no block may jump to entry!");
        });
    // Go trough bbs, finding blocks to linearize.
    // We need a copy of `unstructured`s labels here,
    // to be able to iter and mutate it at the same time.
    let labels: Vec<_> = unstructured.keys().cloned().collect();

    loop {
        let mut lop = false;
        for l in &labels {
            let Some((_, Termiantor::Br(target))) = unstructured.get(l) else {
                continue;
            };
            if pred_count[target] != 1 {
                continue;
            }
            // infinite loop(which should be dead and impossiblle...), no linearize.
            if target == l {
                continue;
            }
            let target = *target;
            let (s_tgt, t_tgt) = unstructured.remove(&target).unwrap();
            let src = unstructured.get_mut(l).unwrap();
            let old = src.0.clone();
            src.0 = StructureRegion::Seq(vec![old, s_tgt]);
            src.1 = t_tgt;
            lop = true;
        }
        if !lop {
            break;
        }
    }
}

#[test]
fn random_cfg() {
    FALLBACK_COUNTER.with(|c| c.store(0, std::sync::atomic::Ordering::Relaxed));
    let iters = 1024;
    crate::unstructured(structurize_random_cfg, iters, 20);
    let mut counter = 0;
    FALLBACK_COUNTER.with(|c| counter = c.load(std::sync::atomic::Ordering::Relaxed));
    eprintln!(
        "fallback needed {}% of the time. {counter}",
        (counter as f64 / iters as f64) * 100.0
    );
}
#[cfg(test)]
thread_local! {
    static FALLBACK_COUNTER:AtomicUsize = AtomicUsize::new(0);
}
