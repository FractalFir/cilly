mod func;
pub use func::*;
mod global;
pub use global::*;
mod module;
pub use module::*;
mod linkage;
pub use linkage::*;
mod global_ident;
pub use global_ident::*;
#[qparse_macros::qparse("PlaceHolder")]
#[derive(Default)]
pub(crate) struct PlaceHolder;

pub(crate) type TyAndAttr = PlaceHolder;

