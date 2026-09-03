use traversable::{Traversable, TraversableMut};

#[qparse_macros::qparse("")]
#[derive(Clone, Copy, Debug, Traversable, TraversableMut)]
pub enum Linkage {
    #[qparse("internal ")]
    Internal,
    #[qparse("private ")]
    Private,
    #[qparse("weak ")]
    Weak,
    #[qparse("linkonce ")]
    LinkOnce,
    #[qparse("common ")]
    Common,
    #[qparse("appending ")]
    Appending,
    #[qparse("extern_weak ")]
    ExternWeak,
    // Omited linkage in LLVM is external. This is stupid as shit,
    // but we need to explictly omit the linkage, otherwise extern
    // globals get sad. Why, LLVM, whyyyyyy... T T
    #[qparse("")]
    External,
}
