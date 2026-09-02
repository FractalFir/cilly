use qparse::Parseable;

#[test]
fn parse_add_emu() {
    //; CILLY_FALLBACK iadd $WIDE $NARROW
    let emu = include_str!("fallback/emu_iadd.cir")
        .replace("$NARROW", "64")
        .replace("$WIDE", "128")
        .replace("$IS_LE", "true")
        .replace("$IS_BE", "false");

    let (reminder, emu) = crate::Fnc::parse(&emu).unwrap();
    todo!("emu:{emu}");
}
