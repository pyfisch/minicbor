use minicbor::{
    Encoding::*,
    Parser, Result,
    Token::{self, *},
};

#[test]
fn test_parse() {
    let parser = Parser::from_slice(b"\x83\x01\x02\x03");
    let output: Result<Vec<Token>> = parser.collect();
    let tokens = output.expect("valid");
    assert_eq!(
        tokens,
        vec![
            StartArray(Some((3, SameByte))),
            Unsigned(1, SameByte),
            Unsigned(2, SameByte),
            Unsigned(3, SameByte)
        ]
    );
}
