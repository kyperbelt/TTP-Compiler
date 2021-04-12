use std::env::Args;

use crate::compiler;

use std::io::prelude::*;
use compiler::Program;
use compiler::lexer;
use compiler::parser;
use std::path;
use std::fs;


#[derive(PartialEq,Debug,Clone,Copy)]
pub enum CommandType{
    Help,
    Version,
    Compile,
    Output,
}



impl CommandType{
    pub fn get_type(command: &str)->Option<CommandType>{
        match command{
            "-v" | "--version"  =>{Some(CommandType::Version)},
            "-h" | "--help"     => {Some(CommandType::Help)},
            "-c" | "--compile"  =>{Some(CommandType::Compile)},
            "-o" | "--output"   =>{Some(CommandType::Output)},
            _=>{None}
        }
    }

    pub fn get_priority(&self)->usize{
        match self{
            CommandType::Compile => {10},
            _=>{100}
        }
    }

    pub fn get_arg_count(&self)->isize{
        match self{
            CommandType::Version =>{0},
            CommandType::Help =>{-1} //variable size
            _=>{1}
        }
    }

    /// get the dependencies for each command
    /// for example -output depends on the compile command
    /// but compile does not depend on output
    pub fn get_dependencies(&self)->Option<&[CommandType]>{
        match self{
            CommandType::Output => {Some(&[CommandType::Compile])},
            _=>{None}
        }
    }

}

#[derive(Debug,PartialEq)]
pub struct Command{
    command_type : CommandType,
    arg : Option<String>
}


pub fn find_command(command_type : CommandType,commands: &[Command])->Option<&Command>{

    for command in commands{

        if command.command_type == command_type{
            return Some(&command)
        }
    }

    None
}


pub fn handle_commands(commands : &[Command])->Result<(),String>{

    let mut iter = commands.iter();

    let mut next_command = iter.next();

    if next_command.as_ref().unwrap().command_type.get_dependencies() != None{
        return Err(format!("{:?} command requires additional dependencies.",
                           next_command.as_ref().unwrap().command_type));
    }

    let mut lexer = lexer::Lexer::create();
    let mut parser : Option<parser::Parser> = None;
    let mut program : Option<Program> = None;
    let mut output : Option<path::PathBuf> = None;

    while next_command != None{
        let command = next_command.unwrap();
        match command.command_type{
            CommandType::Compile =>{
                let in_path = path::PathBuf::from(command.arg.as_ref().unwrap());
                if let Ok(file) = fs::File::open(in_path.as_path()) {
                    let mut kf = file;
                    // do file stuff here
                    let mut source = String::new();
                    if let Err(_) = kf.read_to_string(&mut source){
                        return Err(format!("Unable to read file."));
                    }

                    let tokens = lexer.tokenize(source.as_str())?;
                    let mut inner_parser = parser::Parser::create(&tokens);

                    println!("Tokens:");
                    for token in &tokens{
                        println!("{}",token);
                    }

                    let mut root = inner_parser.generate()?;


                    let inner_program = compiler::Compiler::compile(root)?;

                    println!("{}",inner_program.dump());

                    parser = Some(inner_parser);
                    program = Some(inner_program);




                }else{
                    return Err(format!("Unable to resolve file: {} .",in_path.as_path().to_str().unwrap()));
                }

            },
            CommandType::Output =>{

            },
            CommandType::Version =>{
                println!("[ttpc] by Jonathan Camarena 2021 - version {}", compiler::VERSION);
            },
            CommandType::Help =>{


            }
        }
        next_command = iter.next();
    }
    Ok(())
}

pub fn parse_commands(commands :&mut Args)->Result<Vec<Command>,String>{

    let mut ret_commands: Vec<Command> = Vec::new();

    let mut current = commands.next();

    if current != None && CommandType::get_type(current.as_ref().unwrap())==None{
        // this means we probably have an invalid command
        // or we have the program call as the 0th argument

        current = commands.next();
    }

    while current!=None{
        let command_str : String = current.unwrap();
        let mut next = commands.next();
        if let Some(command_type) = CommandType::get_type(&command_str){
            let arg_count = command_type.get_arg_count();
            // println!("command found : {:?} -- arguments needed {}",command_type,arg_count);

            if arg_count == 0 {
                ret_commands.push(Command{command_type,arg:None});
            }else if arg_count == 1{
                // expect an argument if next is not an argument we throw an error
                if next == None || CommandType::get_type(next.as_ref().unwrap())!=None {
                    return Err(format!("Expected an argument for {:?} command",command_type));
                }else{
                    ret_commands.push(Command{command_type,arg:Some(next.unwrap())});
                    next = commands.next();
                }

            }else if arg_count == -1 {
                // we dont expect an argument but if we get one its ok
                if next != None && CommandType::get_type(next.as_ref().unwrap()) == None{
                    ret_commands.push(Command{command_type,arg:Some(next.unwrap())});
                    next = commands.next();
                }else{
                    ret_commands.push(Command{command_type,arg:None});
                }

            }
        }else{

            //no command found
            return Err(format!("[{}] is not a valid command. use --help | -h for a list of valid commands.",command_str));
        }

        current = next;
    }
    // println!("size of ret commands : {}", ret_commands.len());
    // sort commands by priority to make sure
    // that commands that need to be executed first do so
    ret_commands.sort_by_key(|k| k.command_type.get_priority());


    if ret_commands.len()  == 0{
        // no commands
        Err(format!("No commands. use --help | -h for a list of valid commands."))
    }else{
        Ok(ret_commands)

    }

}
