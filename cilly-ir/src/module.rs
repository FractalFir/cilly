use std::{
    collections::HashMap,
    io::{Read, Write},
    num::NonZeroU32,
    sync::atomic::{AtomicUsize, Ordering},
};

use tempfile::NamedTempFile;

use crate::{
    AttrAndTy, ConstInit, FunctionBuilder, GlobalDeclLinkage, GlobalIdent, InputArgs, Legalzer,
    Linkage, Locals, Section, SourceLocation, TyAndAttr, func::Fnc, global::Global,
};
#[derive(Default, Clone)]
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
    pub fn decl_global(
        &mut self,
        name: GlobalIdent,
        is_weak: bool,
        is_const: bool,
        align: NonZeroU32,
    ) -> GlobalRef {
        let idx = self.globals.len();
        self.globals.push(Global::Decl {
            name,
            linkage: if is_weak {
                crate::GlobalDeclLinkage::ExternWeak
            } else {
                crate::GlobalDeclLinkage::External
            },
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
        let global = self.get_global_mut(global)?;
        let tls = thread_local;
        match global {
            Global::Def { thread_local, .. } | Global::Decl { thread_local, .. } => {
                *thread_local = tls
            }
        };
        Ok(())
    }
    pub fn set_global_link_section(
        &mut self,
        global: GlobalRef,
        link_section: &str,
    ) -> Result<(), ModuleBuilderError> {
        let global = self.get_global_mut(global)?;
        let section = Section(Some(link_section.to_string()));
        match global {
            Global::Def { link_section, .. } | Global::Decl { link_section, .. } => {
                *link_section = section
            }
        };
        Ok(())
    }
    pub fn global_def(
        &mut self,
        global: GlobalRef,
        linkage: Linkage,
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
                        SymbolRef::GlobalRef(global_ref) => {
                            self.globals[global_ref.0].name().clone()
                        }
                        SymbolRef::FuncRef(func_ref) => self.functions[func_ref.0].name().clone(),
                    },
                )
            })
            .collect();
        let initializer = ConstInit::new(bytes, refs, ptr_size)?;
        let global = self.get_global_mut(global)?;
        *global = match global.clone() {
            Global::Decl {
                name,
                kind,
                align,
                thread_local,
                link_section,
                ..
            } => Global::Def {
                name,
                linkage,
                kind,
                initializer,
                align,
                thread_local,
                link_section,
            },
            Global::Def {
                name,
                kind,
                align,
                thread_local,
                link_section,
                ..
            } => Global::Def {
                name,
                linkage,
                kind,
                initializer,
                align,
                thread_local,
                link_section,
            },
        };
        Ok(())
    }
    pub fn declare(
        &mut self,
        name: GlobalIdent,
        linkage: Linkage,
        output: TyAndAttr,
        inputs: Vec<TyAndAttr>,
        va_args: bool,
    ) -> Result<FuncRef, ModuleBuilderError> {
        let fnc = self.functions.len();
        self.functions.push(Fnc::Decl {
            linkage,
            output: AttrAndTy {
                ty: output.ty,
                attr: output.attr,
            },
            name,
            inputs: InputArgs {
                args: inputs,
                va_args,
            },
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
    fn split(
        &mut self,
        mut fn_filter: impl FnMut(&Fnc) -> bool,
        mut gl_filter: impl FnMut(&Global) -> bool,
    ) -> Self {
        let mut functions = vec![];
        for f in self.functions.iter_mut() {
            if fn_filter(f) {
                functions.push((*f).clone());
                let Fnc::Def {
                    src_loc,
                    linkage,
                    output,
                    name,
                    inputs,
                    ..
                } = f
                else {
                    continue;
                };
                *f = Fnc::Decl {
                    src_loc: src_loc.clone(),
                    linkage: if matches!(linkage, Linkage::ExternWeak) {
                        *linkage
                    } else {
                        Linkage::External
                    },
                    output: output.clone(),
                    name: name.clone(),
                    inputs: inputs.clone(),
                };
            } else {
                let Fnc::Def {
                    src_loc,
                    linkage,
                    output,
                    name,
                    inputs,
                    ..
                } = f
                else {
                    functions.push(f.clone());
                    continue;
                };
                functions.push(Fnc::Decl {
                    src_loc: src_loc.clone(),
                    linkage: if matches!(linkage, Linkage::ExternWeak) {
                        *linkage
                    } else {
                        Linkage::External
                    },
                    output: output.clone(),
                    name: name.clone(),
                    inputs: inputs.clone(),
                });
            }
        }
        let mut globals = vec![];
        for global in self.globals.iter_mut() {
            if gl_filter(global) {
                globals.push((*global).clone());
                let Global::Def {
                    name,

                    kind,

                    align,
                    thread_local,
                    link_section,
                    linkage,
                    ..
                } = global.clone()
                else {
                    continue;
                };
                *global = Global::Decl {
                    name,
                    kind,
                    align,
                    thread_local,
                    link_section,
                    linkage: if matches!(linkage, Linkage::ExternWeak) {
                        GlobalDeclLinkage::ExternWeak
                    } else {
                        GlobalDeclLinkage::External
                    },
                };
            } else {
                let global = (*global).clone();
                let Global::Def {
                    name,

                    kind,

                    align,
                    thread_local,
                    link_section,
                    linkage,
                    ..
                } = global
                else {
                    globals.push(global);
                    continue;
                };
                globals.push(Global::Decl {
                    name,
                    kind,
                    align,
                    thread_local,
                    link_section,
                    linkage: if matches!(linkage, Linkage::ExternWeak) {
                        GlobalDeclLinkage::ExternWeak
                    } else {
                        GlobalDeclLinkage::External
                    },
                });
            }
        }
        Self { globals, functions }
    }
    pub fn half_split(mut self) -> (Self, Self) {
        let cnt = AtomicUsize::new(0);
        let res = self.split(
            |f| {
                if f.is_def() {
                    cnt.fetch_add(1, Ordering::Release) % 2 == 0
                } else {
                    false
                }
            },
            |g| {
                if g.is_def() {
                    cnt.fetch_add(1, Ordering::Release) % 2 == 0
                } else {
                    false
                }
            },
        );
        (self, res)
    }
    /// Compiles a module with `llc`
    pub fn llc(&self) -> Result<Vec<u8>, String> {
        let mut command = std::process::Command::new("/usr/lib/llvm-20/bin/llc");
        let mut input = NamedTempFile::with_suffix(".cir").unwrap();
        input.write_all(self.to_string().as_bytes()).unwrap();
        let output = NamedTempFile::with_suffix(".o").unwrap();
        command.arg(input.path());
        command.arg("-filetype=obj");
        command.arg("-o");
        command.arg(output.path());
        let out = command.output().unwrap();
        if out.status.success() {
            let mut out = vec![];
            output.as_file().read_to_end(&mut out).unwrap();
            Ok(out)
        } else {
            let stdout = String::from_utf8(out.stdout).unwrap();
            let stderr = String::from_utf8(out.stderr).unwrap();
            Err(format!("stdout:{stdout}\nstderr:{stderr}"))
        }
    }
    /// Compiles a module with `llc`
    pub fn llc_partial(&self) -> (Vec<Vec<u8>>, Vec<String>) {
        let mut tasklist = vec![(*self).clone()];
        let mut objects = vec![];
        let mut errors = vec![];
        while let Some(task) = tasklist.pop() {
            // Skip empty modules
            if task.symdefcount() == 0 {
                continue;
            }
            match task.llc() {
                Ok(ok) => objects.push(ok),
                Err(err) => {
                    let (lhs, rhs) = task.half_split();
                    if lhs.symdefcount() == 0 || rhs.symdefcount() == 0 {
                        errors.push(err);
                    } else {
                        tasklist.push(lhs);
                        tasklist.push(rhs);
                    }
                }
            }
        }
        (objects, errors)
    }
    /// Global / function count
    pub fn symdefcount(&self) -> usize {
        self.functions.iter().filter(|f| f.is_def()).count()
            + self.globals.iter().filter(|g| g.is_def()).count()
    }
    /// Replaces instructions with calls to a target-specific legalizer.
    pub(crate) fn legalize(
        &mut self,
        mut legalizer: impl FnMut(&mut Fnc, &mut HashMap<GlobalIdent, Fnc>),
    ) {
        let mut idx = 0;
        while idx < self.functions.len() {
            let mut fallbacks = HashMap::new();
            legalizer(&mut self.functions[idx], &mut fallbacks);
            for (name, fallback) in fallbacks {
                self.insert_fn(name, fallback);
            }
            idx += 1;
        }
    }

    fn insert_fn(&mut self, name: GlobalIdent, fnc: Fnc) {
        if !self.functions.iter().any(|f| *f.name() == name) {
            self.functions.push(fnc)
        }
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
    FallbackNotFound { name: String },
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
