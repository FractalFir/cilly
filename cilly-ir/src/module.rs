use crate::{func::Fnc, global::Global};
#[derive(Default)]
pub struct Module {
    globals: Vec<Global>,
    functions: Vec<Fnc>,
}
impl Module {}
