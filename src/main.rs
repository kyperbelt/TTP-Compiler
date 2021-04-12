mod compiler;
mod cli;

use compiler::parser::Parser;
use compiler::lexer::Lexer;
use compiler::Compiler;
use cli::*;

use std::env;


fn main() -> Result<(),String>{

    let commands = cli::parse_commands(&mut env::args())?;

    cli::handle_commands(&commands)?;


    // let test_data =
    //             " nop // this is a comment
    //               ldi a, . 1 +
    //               jmpi .
    //               byte .
    //              ";

    // let mut lexer = Lexer::create();
    // let tokens = lexer.tokenize(test_data);

    // println!("Tokens{}:\n",tokens.len());
    // for token in &tokens{
    //     println!("\t{}",token);
    // }

    // let mut parser = Parser::create(&tokens);

    // let root = parser.generate();

    // println!("{}",root);
    // let result = Compiler::compile(root);
    // println!();
    // if let Ok(program) = result{
    //     println!("{}",program.dump())
    // }else{
    //     println!("{}",result.err().unwrap());
    // }
    Ok(())
}


#[test]
fn test_register_from_char(){
    let result1 = compiler::Register::from_char('a').unwrap();
    let result2 = compiler::Register::from_char('B').unwrap();
    let result3 = compiler::Register::from_char('c').unwrap();
    let result4 = compiler::Register::from_char('D').unwrap();
    assert_eq!(result1,compiler::Register::A);
    assert_eq!(result2,compiler::Register::B);
    assert_eq!(result3,compiler::Register::C);
    assert_eq!(result4,compiler::Register::D);
    assert_eq!(None,compiler::Register::from_char('l'));


}