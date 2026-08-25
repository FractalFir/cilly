use arbitrary::Arbitrary;

#[qparse_macros::qparse("")]
#[derive(Clone, Debug, Arbitrary, PartialEq)]
pub enum CType {
    #[qparse("unsigned char")]
    UChar,
}
