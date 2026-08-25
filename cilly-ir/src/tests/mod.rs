use std::num::NonZeroU8;

use crate::{AttrList, GlobalIdent, Linkage, Module, TyAndAttr, Type};

#[test]
fn declare_identity_module() {
    let mut module = Module::default();
    let i8 = TyAndAttr {
        attr: AttrList::default(),
        ty: Type::ix(NonZeroU8::new(8).unwrap()),
    };
    let ident = module
        .declare(
            GlobalIdent::new("identity").unwrap(),
            Linkage::External,
            i8.clone(),
            vec![i8],
        )
        .unwrap();
    assert_eq!(
        module.to_string().trim(),
        "declare external i8 @identity (i8 %v0)"
    );
    module.set_fn_src_loc(ident, "identity.rs", 67, 42).unwrap();
    assert_eq!(
        module.to_string().trim(),
        "; source @identity.rs:42:67\ndeclare external i8 @identity (i8 %v0)"
    );
    let mut builder = module.fn_builder(ident).unwrap();
    let arg = builder.get_param(0).unwrap();
    assert_eq!(
        *builder.get_type(&arg, &Type::Void).unwrap(),
        Type::ix(NonZeroU8::new(8).unwrap())
    );
    let entry = builder.new_block();
    builder.position_at_end(entry).unwrap();
    builder.build_ret(Some(arg)).unwrap();
    builder.finish(&mut module);
    assert_eq!(
        module.to_string(),
        "; source @identity.rs:42:67\ndefine external i8 @identity (i8 %v0){\n\nret i8 %v0\n}\n"
    );
}
#[test]
fn declare_add_module() {
    let mut module = Module::default();
    let i8 = TyAndAttr {
        attr: AttrList::default(),
        ty: Type::ix(NonZeroU8::new(8).unwrap()),
    };
    let add = module
        .declare(
            GlobalIdent::new("add").unwrap(),
            Linkage::External,
            i8.clone(),
            vec![i8.clone(), i8.clone()],
        )
        .unwrap();
    let mut builder = module.fn_builder(add).unwrap();
    let entry = builder.new_block();
    builder.position_at_end(entry).unwrap();
    let lhs = builder.get_param(0).unwrap();
    let rhs = builder.get_param(1).unwrap();
    let res = builder.build_add(i8.ty, lhs, rhs).unwrap();
    builder.build_ret(Some(res)).unwrap();
    builder.finish(&mut module);
    assert_eq!(
        module.to_string(),
        "define external i8 @add (i8 %v0,i8 %v1){\n%v2 = add i8 %v0, %v1\n\nret i8 %v2\n}\n"
    );
}
#[test]
fn declare_select() {
    let mut module = Module::default();
    let i1 = TyAndAttr {
        attr: AttrList::default(),
        ty: crate::I1_TY.clone(),
    };
    let i8 = TyAndAttr {
        attr: AttrList::default(),
        ty: Type::ix(NonZeroU8::new(8).unwrap()),
    };
    let sel = module
        .declare(
            GlobalIdent::new("sel").unwrap(),
            Linkage::External,
            i8.clone(),
            vec![i1.clone(), i8.clone(), i8.clone()],
        )
        .unwrap();
    let mut builder = module.fn_builder(sel).unwrap();
    let entry = builder.new_block();
    let ret_lhs = builder.new_block();
    let ret_rhs = builder.new_block();
    builder.position_at_end(entry).unwrap();
    let cond = builder.get_param(0).unwrap();
    builder.build_condbr(cond, ret_lhs, ret_rhs).unwrap();
    builder.position_at_end(ret_lhs).unwrap();
    let lhs = builder.get_param(1).unwrap();
    builder.build_ret(Some(lhs)).unwrap();
    builder.position_at_end(ret_rhs).unwrap();
    let rhs = builder.get_param(2).unwrap();
    builder.build_ret(Some(rhs)).unwrap();
    builder.finish(&mut module);
    println!("{module}");
    assert_eq!(module.to_string(), "");
}
