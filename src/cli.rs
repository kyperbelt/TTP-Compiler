use std::env::Args;

use crate::compiler;

use std::io::prelude::*;
use std::fs::OpenOptions;
use compiler::Program;
use compiler::lexer;
use compiler::parser;
use std::path;
use std::fs;


#[derive(PartialEq,Debug,Clone,Copy)]
pub enum CommandType{
    Help,               // show help for all commands or a specific command
    Version,            // show the current version of the ttpc
    Compile,            // compile a ttpasm file
    Output,             // set the output file
    Binary,             // set if binary output (off by default)
    Dump,               // dump the tokens
    Tree                // show the hierchy
    // Analyze TODO: create a vm to run analysis on
}



impl CommandType{
    pub fn get_type(command: &str)->Option<CommandType>{
        match command{
            "-v" | "--version"  =>{Some(CommandType::Version)},
            "-h" | "--help"     => {Some(CommandType::Help)},
            "-c" | "--compile"  =>{Some(CommandType::Compile)},
            "-o" | "--output"   =>{Some(CommandType::Output)},
            "-b" | "--binary"   =>{Some(CommandType::Binary)}
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
            CommandType::Output | CommandType::Binary |
            CommandType::Tree   => {Some(&[CommandType::Compile])},
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

    // if next_command.as_ref().unwrap().command_type.get_dependencies() != None{
    //     return Err(format!("{:?} command requires additional dependencies.",
    //                        next_command.as_ref().unwrap().command_type));
    // }

    let mut lexer = lexer::Lexer::create();
    let mut parser : Option<parser::Parser> = None;
    let mut program : Option<Program> = None;
    let mut output : Option<path::PathBuf> = None;
    let mut binary : bool = false;

    while next_command != None{

        let command = next_command.unwrap();

        // check dependencies
        if let Some(dependencies) = command.command_type.get_dependencies(){
            for dep in dependencies{
                if let None = find_command(*dep,commands){
                    return Err(format!("Dependency missing for [{:?}] command. required:{:?}.",command.command_type,dep));
                }
            }
        }

        match command.command_type{
            CommandType::Compile =>{
                let in_path = path::PathBuf::from(command.arg.as_ref().unwrap());

                if in_path.is_file(){

                   let mut kf = swap_e(fs::File::open(in_path.as_path()))?;

                    // do file stuff here
                    let mut source = String::new();
                    if let Err(_) = kf.read_to_string(&mut source){
                        return Err(format!("Unable to read file."));
                    }

                    let tokens = lexer.tokenize(source.as_str())?;
                    let mut inner_parser = parser::Parser::create(&tokens);

                    // println!("Tokens:");
                    // for token in &tokens{
                    //     println!("{}",token);
                    // }

                    let root = inner_parser.generate()?;


                    let inner_program = compiler::Compiler::compile(root)?;

                    // println!("{}",inner_program.dump());

                    parser = Some(inner_parser);
                    program = Some(inner_program);

                    let stem = in_path.file_stem().unwrap();
                    let mut out_path = path::PathBuf::from(in_path.as_os_str());
                    out_path.set_file_name(stem);

                    output = Some(out_path)


                }else{
                    return Err(format!("{} is not a valid file.",command.arg.as_ref().unwrap()))
                }

            },
            CommandType::Output =>{
                // set the output file
                if let Some(path) = &command.arg{

                    let new_out = path::PathBuf::from(path);
                    if new_out.is_dir(){
                        return Err(format!("[{}] output file path must contain a filename", path));
                    }

                    output = Some(new_out);

                }
            },
            CommandType::Version =>{
                println!("[ttpc] by Jonathan Camarena 2021 - version {}", compiler::VERSION);
            },
            CommandType::Help =>{
                // display help for commands

            },
            CommandType::Binary=>{
                // output as binary without logisim header
                binary = true;
            },
            CommandType::Dump =>{
                // dump the tokens to console TODO: allow file output
            },
            CommandType::Tree =>{
                // show the hierchy
            }
        }
        next_command = iter.next();
    }

    if let Some(p) = program{
        let mut options = OpenOptions::new();
        let mut file = swap_e(options.write(true).create(true).append(false).open(output.as_ref().unwrap()))?;

        let mut out = String::new();

        if !binary{
            out.push_str("v2.0 raw\n"); // makes this compatible with Logisim v 2.7.1
        }
        out.push_str(p.dump().as_str());

        if let Err(_) = file.write(out.as_bytes()){
            return Err(format!("unable to write to file!"))
        }

    }

    println!("compiled to file: {:?}" , output.unwrap().as_os_str());

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

/// swap an error Result from file io into one that just returns a string
pub fn swap_e<T>(result: Result<T,std::io::Error>)->Result<T,String>{
    match result{
        Ok(ok)=>Ok(ok),
        Err(e)=>Err(format!("{}",e))
    }
}