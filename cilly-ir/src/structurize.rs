//! This module implements a relatively simple structurization algorithm, with a few goals.
//! Besides simplicty, an important goal for the algorithm is it's resistance to irreducible
//! control flow: it is supposed to reduce the control flow(to strurcutred form) as much as
//! possible, falling back to switch-emulation ONLY in places where that is absoultely necessary.
//!
//! We do this, by recursively structurizing regions: a structured if may be
//! a "node" of an irreducible CFG, and also contain another, disjoint irrudicible CFG as
//! one of it's arms.
//!
use std::{collections::HashMap, num::NonZeroU8};

use crate::{
    BasicBlock, Body, CFGElem, Constant, InstrList, Instruction, Label, Locals, Operand, Switch,
    Termiantor, Type,
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
    let unstructured = bbs
        .into_iter()
        .enumerate()
        .map(|(id, bb)| {
            (
                Label { id: id as u32 },
                (StructureRegion::OpSeq(bb.instrs), bb.term.unwrap()),
            )
        })
        .collect();
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

enum StructureRegion {
    OpSeq(InstrList),
    Unstructured {
        entry: Label,
        unstructured: HashMap<Label, (Self, Termiantor)>,
        exit: Label,
    },
}
fn label_map(labels:&HashMap<Label, impl Sized>,exit:Label)->HashMap<Label, Constant>{
    let mut map:HashMap<Label, Constant> = labels.iter().map(|(l,_)|l).enumerate().map(|(idx,l)|(*l, Constant::Int(idx as i128))).collect();
    map.insert(exit, Constant::Int(map.len() as _));
    map
}
impl StructureRegion {
    fn to_body(self, locals: &mut Locals, sass: &mut Vec<Type>, ret_ty: &Type) -> Body {
        match self {
            StructureRegion::OpSeq(instrs) => {
                let cfg = CFGElem::Instructions { instrs };
                Body { elems: vec![cfg] }
            }
            StructureRegion::Unstructured {
                entry,
                unstructured,
                exit,
            } => {
                // Unstructured CFG fallback.
                // first - how many blocks are there(ergo - size of our dispatch type?)
                let bits = usize::BITS - unstructured.len().leading_zeros();
                // We round up to bytes
                let bytes = bits.div_ceil(8);
                // Then try to pick a power-of-2 bitesize - 2^ceil(log2(x))
                let bytes = bytes.next_power_of_two();
                // Then round back to bits
                let bits = bytes * 8;
                let dispatch_ty = Type::Int {
                    bitwidth: NonZeroU8::new(bits as _).unwrap(),
                };
                // dispatch var
                let local = locals.add_local(dispatch_ty.clone());
                let mut cases = vec![];
                let label_map = label_map(&unstructured,exit);
                // An important check - sometimes, the control flow diverges entirely(no jump to exit).
                let no_exit = !unstructured.iter().map(|(_,(_,t))|t.sucessors()).flatten().any(|s|s == exit);
                for  (label, (region, term)) in unstructured.into_iter(){
                    let mut region = region.to_body(locals, sass, ret_ty);
                    // store
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
                                val:Operand::Constant(val),
                            }];
                            CFGElem::Instructions {
                                instrs: InstrList { instrs },
                            }
                        }
                        Termiantor::BrCond { cond, then, els } => todo!(),
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
                sass.push(Type::Int {
                    bitwidth: NonZeroU8::new(1).unwrap(),
                });
                let check = Instruction::ICmp {
                    dst: cond,
                    ty: dispatch_ty.clone(),
                    lhs: Operand::SSA(val),
                    rhs: Operand::Constant(label_map[&exit].clone()),
                    cmp: crate::ICmp::Ne,
                };
                let body = Body{ elems: vec![
                    CFGElem::Instructions { instrs: InstrList { instrs: vec![load_val] } },
                    CFGElem::Switch(switch),
                    CFGElem::Instructions { instrs: InstrList { instrs: vec![load_val2, check] } },
                ] };
                let dloop = CFGElem::DoWhile { cond:Operand::SSA(cond), label: entry, body};
                let set_entry = Instruction::StoreLocal { local, ty: dispatch_ty, val: Operand::Constant(label_map[&entry].clone()) };
                let mut elems =  vec![
                    CFGElem::Instructions { instrs: InstrList { instrs: vec![set_entry] } },
                    dloop
                ];
                if no_exit{
                    elems.push(CFGElem::Trap);
                }
                // fallback - what happens if nobody jumps to exit?
                Body { elems }
            }
        }
    }
    fn structurize(&mut self, locals: &mut Locals, sass: &mut Vec<Type>, ret_ty: &Type) {
        let (entry, unstructured, exit) = match self {
            StructureRegion::OpSeq(instr_list) => return (),
            StructureRegion::Unstructured {
                entry,
                unstructured,
                exit,
            } => (entry, unstructured, exit),
        };
        /*
        Petgraph pre and post dominators?
        */
    }
}
