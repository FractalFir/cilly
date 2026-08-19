use std::num::NonZeroU8;

use crate::{AttrList, GlobalIdent, Linkage, Module, TyAndAttr, Type};

#[test]
fn declare_identity_module() {
    let mut module = Module::default();
    let i8 = TyAndAttr {
        attr: AttrList::default(),
        ty: Type::Int {
            bitwidth: NonZeroU8::new(8).unwrap(),
        },
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
        *builder.get_type(&arg).unwrap(),
        Type::Int {
            bitwidth: NonZeroU8::new(8).unwrap()
        }
    );
    let entry = builder.new_block();
    builder.position_at_end(entry).unwrap();
    builder.build_ret(Some(arg)).unwrap();
    builder.finish(&mut module);
}
