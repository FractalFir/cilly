use std::{collections::HashMap, num::NonZeroU8};

use crate::{
    Body, CFGElem, Constant, I1_TY, InstrList, Instruction, Label, Locals, Operand, SSAVal, Switch,
    Termiantor, Type, structurize::StructureRegion,
};
impl StructureRegion {
    pub(super) fn to_body(self, locals: &mut Locals, sass: &mut Vec<Type>, ret_ty: &Type) -> Body {
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
            StructureRegion::If { cond, then, label } => {
                let body = then.to_body(locals, sass, ret_ty);
                let cfg = CFGElem::If { cond, label, body };
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
                        Termiantor::Trap => {
                            s.elems.push(CFGElem::Trap);
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
                        Termiantor::Switch { .. } => (),
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
                        Termiantor::Trap => CFGElem::Trap,
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
                        Termiantor::Switch {
                            default,
                            ty,
                            cases,
                            val,
                        } => {
                            let (mut instrs, val) = switch_cases_select(
                                default,
                                cases,
                                sass,
                                &label_map,
                                val,
                                &ty,
                                &dispatch_ty,
                            );
                            instrs.push(Instruction::StoreLocal {
                                local: local.clone(),
                                ty: dispatch_ty.clone(),
                                val: val,
                            });
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
                    super::FALLBACK_COUNTER
                        .with(|v| v.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
                }
                // fallback - what happens if nobody jumps to exit?
                if no_exit {
                    elems.push(CFGElem::Trap);
                }
                Body { elems }
            }
        }
    }
}
fn switch_gen_select(
    val: Operand,
    val_ty: &Type,
    case_val: Constant,
    prev: Operand,
    case_res: Constant,
    sass: &mut Vec<Type>,
    dispatch_ty: &Type,
) -> ([Instruction; 2], Operand) {
    let dst = SSAVal(sass.len() as _);
    sass.push(I1_TY.clone());
    let is_case = Instruction::ICmp {
        dst,
        ty: val_ty.clone(),
        lhs: val,
        rhs: Operand::Constant(case_val),
        cmp: crate::ICmp::Eq,
    };
    let is_case_dst = dst;
    let dst = SSAVal(sass.len() as _);
    sass.push(dispatch_ty.clone());
    let select = Instruction::Select {
        dst,
        cond: Operand::SSA(is_case_dst),
        ty: dispatch_ty.clone(),
        then: Operand::Constant(case_res),
        els: prev,
        cond_ty: I1_TY.clone(),
    };
    ([is_case, select], Operand::SSA(dst))
}
fn switch_cases_select(
    default: Label,
    mut cases: Vec<(Constant, Label)>,
    sass: &mut Vec<Type>,
    label_map: &HashMap<Label, Constant>,
    val: Operand,
    val_ty: &Type,
    dispatch_ty: &Type,
) -> (Vec<Instruction>, Operand) {
    cases.sort_by_key(|(c, _)| c.as_i128());
    let mut instrs = vec![];
    let mut prev = Operand::Constant(label_map[&default].clone());
    for (case_val, label) in cases {
        let case_res = label_map[&label].clone();
        let (ins, curr) = switch_gen_select(
            val.clone(),
            val_ty,
            case_val,
            prev,
            case_res,
            sass,
            dispatch_ty,
        );
        instrs.extend(ins);
        prev = curr;
    }
    (instrs, prev)
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
