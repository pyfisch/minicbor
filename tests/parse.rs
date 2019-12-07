use minicbor::{Parser, Result, Token};

#[test]
fn test_parse() {
    let parser = Parser::new(b"\x83\x01\x02\x03");
    let tokens: Result<Vec<Token>> = parser.collect();
    println!("tokens {:?}", tokens);
    panic!()
}
