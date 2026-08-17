use std::num::NonZeroU32;

use crate::{GlobalIdent, Linkage, PlaceHolder};

#[qparse_macros::qparse("")]
pub(crate) enum GlobalKind {
    #[qparse("global")]
    Global,
    #[qparse("constant")]
    Constant,
}
#[qparse_macros::qparse("{name} = {linkage} {kind} {initializer}, align {align}")]
pub(crate) struct Global {
    name: GlobalIdent,
    linkage: Linkage,
    kind: GlobalKind,
    initializer: ConstInit,
    align: NonZeroU32,
}
pub(crate) struct ConstInit{
    bytes:Vec<u8>,
    // Constraint: this is always in order of offsets.
    refs:Vec<(u32, GlobalIdent)>,
}
impl ConstInit {
    pub(crate) fn new(bytes: Vec<u8>, mut refs: Vec<(u32, GlobalIdent)>) -> Self {
        refs.sort_by_key(|(offset,_)|*offset);
        Self { bytes, refs }
    }
    pub(crate) fn at_offset(&self, offset:u32)->Option<&(u32, GlobalIdent)>{
        let idx = self.refs.binary_search_by_key(&offset, |(offset,_)|*offset).ok()?;
        self.refs.get(idx)
    }
}
pub fn const_init_str(bytes:&[u8])->String{
    use std::fmt::Write;
    let mut s = String::with_capacity(bytes.len() + 3);
    s.push_str("c\"");
    for b in bytes{
        let c = *b as char ;
        if c.is_ascii_alphanumeric() || matches!(c,' ' | ':' | ';' ){
            s.push(c);
        } else{
            write!(s, "\\{b:x}").unwrap();
        }
    }
    s.push('"');
    s
}
impl std::fmt::Display for ConstInit{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.refs.is_empty(){
            write!(f, "[{len}xi8] {init}",len = self.bytes.len(),init = const_init_str(&self.bytes))?;
            return Ok(());
        }
        let mut byterun:Vec<u8> = vec![];
        for (offset,byte) in self.bytes.iter().enumerate(){
            if let Some(rf) = self.at_offset(offset as u32){
                todo!("reference {}",rf.1)
            } 
        }
        todo!()
    }
}
impl qparse::Parseable<qparse::Display> for ConstInit {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        todo!()
    }
}

#[test]
fn text_global() {
    panic!(
        "{}",
        &Global {
            name: GlobalIdent::new("hello_func🦆").unwrap(),
            linkage: Linkage::External,
            kind: GlobalKind::Constant,
            initializer: ConstInit::new(b"FELLING CUTE :3 \xF0\x9F\xA6\x86".to_vec(), vec![]),
            align: NonZeroU32::new(128).unwrap()
        }.to_string()
    )
}
