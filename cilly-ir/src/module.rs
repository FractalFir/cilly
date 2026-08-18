use std::num::NonZeroU32;

use crate::{
    AttrAndTy, ConstInit, GlobalIdent, InputArgs, Linkage, TyAndAttr, func::Fnc, global::Global,
};
#[derive(Default)]
pub struct Module {
    globals: Vec<Global>,
    pub(crate) functions: Vec<Fnc>,
}
impl Module {
    pub fn add_global(
        &mut self,
        name: GlobalIdent,
        linkage: Linkage,
        is_const: bool,
        align: NonZeroU32,
    ) -> GlobalRef {
        let idx = self.globals.len();
        self.globals.push(Global {
            name,
            linkage,
            initializer: ConstInit::new(vec![], vec![], 0).unwrap(),
            kind: if is_const {
                crate::GlobalKind::Constant
            } else {
                crate::GlobalKind::Global
            },
            align,
        });
        GlobalRef(idx)
    }
    pub fn set_global_init(
        &mut self,
        global: GlobalRef,
        bytes: Vec<u8>,
        refs: Vec<(u32, GlobalRef)>,
        ptr_size: u32,
    ) -> Result<(), ModuleBuilderError> {
        let refs = refs
            .into_iter()
            .map(|(idx, r)| (idx, self.globals[r.0].name.clone()))
            .collect();
        let globals = self
            .globals
            .get_mut(global.0)
            .ok_or(ModuleBuilderError::InvalidGlobalRef)?;
        globals.initializer = ConstInit::new(bytes, refs, ptr_size)?;
        Ok(())
    }
    pub fn declare(
        &mut self,
        name: GlobalIdent,
        linkage: Linkage,
        output: TyAndAttr,
        inputs: Vec<TyAndAttr>,
    ) -> Result<FuncRef, ModuleBuilderError> {
        let fnc = self.functions.len();
        self.functions.push(Fnc::Decl {
            linkage,
            output: AttrAndTy {
                ty: output.ty,
                attr: output.attr,
            },
            name,
            inputs: InputArgs { args: inputs },
            src_loc: Default::default(),
        });
        Ok(FuncRef(fnc))
    }
}
#[derive(Debug)]
pub enum ModuleBuilderError {
    InvalidGlobalRef,
    GlobalInitNotEnoughBytes,
    GlobalInitAddrByteNonzero,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct GlobalRef(usize);
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FuncRef(pub(crate) usize);
