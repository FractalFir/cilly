use std::num::NonZeroU8;

use crate::{AttrList, GlobalIdent, Linkage, Module, TyAndAttr, Type};

#[test]
fn declare_identity_module(){
    let mut module = Module::default();
    let i8 = TyAndAttr { attr: AttrList::default(), ty: Type::Int { bitwidth: NonZeroU8::new(8).unwrap() } };
    let ident = module.declare(GlobalIdent::new("identity").unwrap(), Linkage::External,i8 .clone(), vec![i8]).unwrap();
    assert_eq!(module.to_string().trim(), "declare external i8 @identity (i8 %v0)");
    module.set_fn_src_loc(ident, "identity.rs", 67, 42).unwrap();
    assert_eq!(module.to_string().trim(), "; source @identity.rs:42:67\ndeclare external i8 @identity (i8 %v0)");
}