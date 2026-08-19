use arbitrary::Arbitrary;

#[qparse_macros::qparse("")]
#[derive(Clone, Debug, Arbitrary)]
pub enum CType {
    #[qparse("unsigned char")]
    UChar,
}
