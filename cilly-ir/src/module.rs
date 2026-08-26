use std::num::NonZeroU32;

use crate::{
    AttrAndTy, ConstInit, FunctionBuilder, GlobalIdent, InputArgs, Linkage, Locals, Section,
    SourceLocation, TyAndAttr, func::Fnc, global::Global,
};
#[derive(Default)]
pub struct Module {
    globals: Vec<Global>,
    pub(crate) functions: Vec<Fnc>,
}
impl std::fmt::Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for global in &self.globals {
            writeln!(f, "{global}")?;
        }
        for func in &self.functions {
            writeln!(f, "{func}")?;
        }
        Ok(())
    }
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
            thread_local: false,
            link_section: Section::empty(),
        });
        GlobalRef(idx)
    }
    fn get_global_mut(&mut self, global: GlobalRef) -> Result<&mut Global, ModuleBuilderError> {
        self.globals
            .get_mut(global.0)
            .ok_or(ModuleBuilderError::InvalidGlobalRef)
    }
    pub fn set_global_tls(
        &mut self,
        global: GlobalRef,
        thread_local: bool,
    ) -> Result<(), ModuleBuilderError> {
        self.get_global_mut(global)?.thread_local = thread_local;
        Ok(())
    }
    pub fn set_global_link_section(
        &mut self,
        global: GlobalRef,
        link_section: &str,
    ) -> Result<(), ModuleBuilderError> {
        self.get_global_mut(global)?.link_section = Section(Some(link_section.to_string()));
        Ok(())
    }
    pub fn set_global_init(
        &mut self,
        global: GlobalRef,
        bytes: Vec<u8>,
        refs: Vec<(u32, SymbolRef)>,
        ptr_size: u32,
    ) -> Result<(), ModuleBuilderError> {
        let refs = refs
            .into_iter()
            .map(|(idx, r)| {
                (
                    idx,
                    match r {
                        SymbolRef::GlobalRef(global_ref) => self.globals[global_ref.0].name.clone(),
                        SymbolRef::FuncRef(func_ref) => self.functions[func_ref.0].name().clone(),
                    },
                )
            })
            .collect();
        self.get_global_mut(global)?.initializer = ConstInit::new(bytes, refs, ptr_size)?;
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
    pub fn fn_builder(&mut self, id: FuncRef) -> Result<FunctionBuilder, ModuleBuilderError> {
        let fnc = self
            .functions
            .get(id.0 as usize)
            .ok_or(ModuleBuilderError::InvalidFuncRef { fnc: id })?;
        let Fnc::Decl { inputs, .. } = &fnc else {
            return Err(ModuleBuilderError::FuncFinished);
        };
        let mut builder = FunctionBuilder {
            id,
            fnc: fnc.clone(),
            locals: Locals::empty(),
            bbs: vec![],
            ssas: vec![],
            pos: None,
        };
        for arg in &inputs.args {
            builder.alloc_ssa_id(arg.ty.clone());
        }
        Ok(builder)
    }
    pub fn set_fn_src_loc(
        &mut self,
        fnc: FuncRef,
        file: impl Into<String>,
        col: u32,
        line: u32,
    ) -> Result<(), ModuleBuilderError> {
        let fnc = self
            .functions
            .get_mut(fnc.0)
            .ok_or(ModuleBuilderError::InvalidFuncRef { fnc })?;
        let file = GlobalIdent::new(file).ok_or(ModuleBuilderError::InvalidSourceLoc)?;
        let src = SourceLocation {
            opt: Some(crate::SourceLocationInner { file, line, col }),
        };
        match fnc {
            Fnc::Def { src_loc, .. } | Fnc::Decl { src_loc, .. } => *src_loc = src,
        }
        Ok(())
    }
}
#[derive(Debug)]
pub enum ModuleBuilderError {
    InvalidGlobalRef,
    GlobalInitNotEnoughBytes,
    GlobalInitAddrByteNonzero,
    InvalidFuncRef { fnc: FuncRef },
    InvalidSourceLoc,
    FuncFinished,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct GlobalRef(usize);
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FuncRef(pub(crate) usize);
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SymbolRef {
    GlobalRef(GlobalRef),
    FuncRef(FuncRef),
}
