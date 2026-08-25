//! This module implements a relatively simple structurization algorithm, with a few goals.
//! Besides simplicty, an important goal for the algorithm is it's resistance to irreducible
//! control flow: it is supposed to reduce the control flow(to strurcutred form) as much as
//! possible, falling back to switch-emulation ONLY in places where that is absoultely necessary.
//!
//! We do this, by recursively structurizing regions: a structured if may be
//! a "node" of an irreducible CFG, and also contain another, disjoint irrudicible CFG as
//! one of it's arms.
//!
#[cfg(test)]
use std::sync::atomic::AtomicUsize;
use std::{collections::HashMap, num::NonZeroU8};

use crate::{
    BasicBlock, Body, CFGElem, Constant, I1_TY, InstrList, Instruction, Label, Locals, Operand,
    SSAVal, Switch, Termiantor, Type,
};

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
#[derive(Clone)]
enum StructureRegion {
    OpSeq(InstrList),
    Unstructured {
        entry: Label,
        unstructured: HashMap<Label, (Self, Termiantor)>,
        exit: Label,
    },
    Seq(Vec<Self>),
}
fn label_map(labels: &HashMap<Label, impl Sized>, exit: Label) -> HashMap<Label, Constant> {
    let mut map: HashMap<Label, Constant> = labels
        .iter()
        .map(|(l, _)| l)
        .enumerate()
        .map(|(idx, l)| (*l, Constant::Int(idx as i128)))
        .collect();
    map.insert(exit, Constant::Int(map.len() as _));
    map
}
impl StructureRegion {
    fn to_body(self, locals: &mut Locals, sass: &mut Vec<Type>, ret_ty: &Type) -> Body {
        match self {
            StructureRegion::Seq(mut seq) => {
                assert!(
                    !seq.is_empty(),
                    "structurizer has an invalid input: 0-length control flow sequence"
                );
                let mut body = seq.remove(0).to_body(locals, sass, ret_ty);
                for s in seq {
                    body.elems.extend(s.to_body(locals, sass, ret_ty).elems);
                }
                body
            }
            StructureRegion::OpSeq(instrs) => {
                let cfg = CFGElem::Instructions { instrs };
                Body { elems: vec![cfg] }
            }
            StructureRegion::Unstructured {
                entry,
                unstructured,
                exit,
            } => {
                // special case - one unstructured - no fallback needed.
                if unstructured.len() == 1 {
                    let (s, t) = unstructured[&entry].clone();
                    let mut s = s.to_body(locals, sass, ret_ty);
                    match t {
                        Termiantor::VoidRet => {
                            s.elems.push(CFGElem::VoidRet);
                            return s;
                        }
                        Termiantor::Ret(operand) => {
                            s.elems.push(CFGElem::Return {
                                ty: ret_ty.clone(),
                                operand,
                            });
                            return s;
                        }
                        Termiantor::Br(label) => {
                            if label == exit {
                                return s;
                            } else {
                                assert_eq!(
                                    label, entry,
                                    "Invalid unstructured region, contains jump to missing label {label}"
                                );
                                let l = CFGElem::DoWhile {
                                    cond: Operand::Constant(Constant::True),
                                    label,
                                    body: s,
                                };
                                return Body { elems: vec![l] };
                            }
                        }
                        Termiantor::BrCond { .. } => (),
                    }
                }
                // Unstructured CFG fallback.
                // first - how many blocks are there(ergo - size of our dispatch type?)
                let bits = usize::BITS - unstructured.len().leading_zeros();
                // We round up to bytes
                let bytes = bits.div_ceil(8);
                // Then try to pick a power-of-2 bitesize - 2^ceil(log2(x))
                let bytes = bytes.next_power_of_two();
                // Then round back to bits
                let bits = bytes * 8;
                let dispatch_ty = Type::ix(NonZeroU8::new(bits as _).unwrap());
                // dispatch var
                let local = locals.add_local(dispatch_ty.clone());
                let mut cases = vec![];
                let label_map = label_map(&unstructured, exit);
                // An important check - sometimes, the control flow diverges entirely(no jump to exit).
                let no_exit = !unstructured
                    .iter()
                    .map(|(_, (_, t))| t.sucessors())
                    .flatten()
                    .any(|s| s == exit);
                for (label, (region, term)) in unstructured.into_iter() {
                    let mut region = region.to_body(locals, sass, ret_ty);
                    // store the dispatcher
                    let term = match term {
                        Termiantor::VoidRet => CFGElem::VoidRet,
                        Termiantor::Ret(operand) => CFGElem::Return {
                            ty: ret_ty.clone(),
                            operand,
                        },
                        Termiantor::Br(label) => {
                            let val = label_map[&label].clone();
                            let instrs = vec![Instruction::StoreLocal {
                                local: local.clone(),
                                ty: dispatch_ty.clone(),
                                val: Operand::Constant(val),
                            }];
                            CFGElem::Instructions {
                                instrs: InstrList { instrs },
                            }
                        }
                        Termiantor::BrCond { cond, then, els } => {
                            let dst = SSAVal(sass.len() as u32);
                            sass.push(dispatch_ty.clone());
                            let instrs = vec![
                                Instruction::Select {
                                    dst,
                                    cond,
                                    cond_ty: I1_TY.clone(),
                                    ty: dispatch_ty.clone(),
                                    then: Operand::Constant(label_map[&then].clone()),
                                    els: Operand::Constant(label_map[&els].clone()),
                                },
                                Instruction::StoreLocal {
                                    local: local.clone(),
                                    ty: dispatch_ty.clone(),
                                    val: Operand::SSA(dst),
                                },
                            ];
                            CFGElem::Instructions {
                                instrs: InstrList { instrs },
                            }
                        }
                    };
                    region.elems.push(term);
                    let cst = label_map[&label].clone();
                    cases.push((cst, label, region));
                }
                // dispatch instr - for the switch
                let val = crate::SSAVal(sass.len() as u32);
                sass.push(dispatch_ty.clone());
                let load_val = Instruction::LoadLocal {
                    dst: val,
                    local,
                    ty: dispatch_ty.clone(),
                };
                // we could use one of the labels as the default - would be safe and alll... but it is safer to trap on it.
                let switch = Switch {
                    default_label: entry,
                    default: Body {
                        elems: vec![CFGElem::Trap],
                    },
                    ty: dispatch_ty.clone(),
                    cases,
                    val: Operand::SSA(val),
                };
                // besdies dispatching, we also need to check if the loop is done loopin
                let val = crate::SSAVal(sass.len() as u32);
                sass.push(dispatch_ty.clone());
                let load_val2 = Instruction::LoadLocal {
                    dst: val,
                    local,
                    ty: dispatch_ty.clone(),
                };
                let cond = crate::SSAVal(sass.len() as u32);
                sass.push(I1_TY.clone());
                let check = Instruction::ICmp {
                    dst: cond,
                    ty: dispatch_ty.clone(),
                    lhs: Operand::SSA(val),
                    rhs: Operand::Constant(label_map[&exit].clone()),
                    cmp: crate::ICmp::Ne,
                };
                let body = Body {
                    elems: vec![
                        CFGElem::Instructions {
                            instrs: InstrList {
                                instrs: vec![load_val],
                            },
                        },
                        CFGElem::Switch(switch),
                        CFGElem::Instructions {
                            instrs: InstrList {
                                instrs: vec![load_val2, check],
                            },
                        },
                    ],
                };
                let dloop = CFGElem::DoWhile {
                    cond: Operand::SSA(cond),
                    label: entry,
                    body,
                };
                let set_entry = Instruction::StoreLocal {
                    local,
                    ty: dispatch_ty,
                    val: Operand::Constant(label_map[&entry].clone()),
                };
                let mut elems = vec![
                    CFGElem::Instructions {
                        instrs: InstrList {
                            instrs: vec![set_entry],
                        },
                    },
                    dloop,
                ];
                // for some statisitcs about the ammout of time a fallback was needed
                #[cfg(test)]
                {
                    FALLBACK_COUNTER.with(|v| v.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
                }
                // fallback - what happens if nobody jumps to exit?
                if no_exit {
                    elems.push(CFGElem::Trap);
                }
                Body { elems }
            }
        }
    }
    fn structurize(&mut self, locals: &mut Locals, sass: &mut Vec<Type>, ret_ty: &Type) {
        let (entry, unstructured, exit) = match self {
            StructureRegion::OpSeq(_) => return (),
            StructureRegion::Seq(seq) => {
                return seq
                    .iter_mut()
                    .for_each(|s| s.structurize(locals, sass, ret_ty));
            }
            StructureRegion::Unstructured {
                entry,
                unstructured,
                exit,
            } => (entry, unstructured, exit),
        };

        // Maybe convert the Unstructured into a nested tree of unstrurctured-s?
        /*
        Petgraph pre and post dominators?
        */
    }
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
#[cfg(test)]
fn structurize_random_cfg(u: &mut arbitrary::Unstructured) -> arbitrary::Result<()> {
    let count = u.arbitrary::<u8>()? as usize + 2;
    let mut cases = vec![];
    u.arbitrary_loop(Some(2), Some(count as u32), |u| {
        let res = match u.int_in_range(0..=100)? {
            0..=50 => vec![u.arbitrary::<u8>()?, u.arbitrary::<u8>()?],
            51..=75 => vec![u.arbitrary::<u8>()?],
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
        &Type::ix(NonZeroU8::new(8).unwrap()),
    );
    let _ = body;
    Ok(())
}
#[test]
fn random_cfg() {
    FALLBACK_COUNTER.with(|c| c.store(0, std::sync::atomic::Ordering::Relaxed));
    let iters = 1024;
    crate::unstructured(structurize_random_cfg, iters, 4);
    let mut counter = 0;
    FALLBACK_COUNTER.with(|c| counter = c.load(std::sync::atomic::Ordering::Relaxed));
    eprintln!(
        "fallback needed {}% of the time. {counter}",
        (counter as f64 / iters as f64) * 100.0
    );
    //panic!()
}
#[cfg(test)]
thread_local! {
    static FALLBACK_COUNTER:AtomicUsize = AtomicUsize::new(0);
}
