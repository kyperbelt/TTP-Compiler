use std::env::Args;

use crate::{compiler, vm};

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
    Tree,               // show the hierchy
    Strict,              // strict mode to not allow registers are labels and also becomes case sensitive
    Analyze,
}



impl CommandType{
    pub fn get_type(command: &str,require_prefix: bool)->Option<CommandType>{
        match command{
            "-v" | "--version"  =>{Some(CommandType::Version)},
            "-h" | "--help"     =>{Some(CommandType::Help)},
            "-c" | "--compile"  =>{Some(CommandType::Compile)},
            "-o" | "--output"   =>{Some(CommandType::Output)},
            "-b" | "--binary"   =>{Some(CommandType::Binary)},
            "-d" | "--dump"     =>{Some(CommandType::Dump)},
            "-t" | "--tree"     =>{Some(CommandType::Tree)},
            "-s" | "--strict"   =>{Some(CommandType::Strict)}
            "-a" | "--analyze"   =>{Some(CommandType::Analyze)}
            _=>{
                if !require_prefix{
                    CommandType::get_type_without_prefix(command)
                }else{
                    None
                }
            }
        }
    }

    /// get
    pub fn get_priority(&self)->usize{
        match self{
            CommandType::Compile => {50},
            CommandType::Dump   |
            CommandType::Tree   |
            CommandType::Strict => {40}
            _=>{100}
        }
    }

    pub fn get_arg_count(&self)->isize{
        match self{
            CommandType::Version    |
            CommandType::Binary     |
            CommandType::Dump       |
            CommandType::Strict     |
            CommandType::Analyze    |
            CommandType::Tree =>{0},
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
            CommandType::Tree   | CommandType::Dump   |
            CommandType::Analyze|
            CommandType::Strict => {Some(&[CommandType::Compile])},
            _=>{None}
        }
    }

    /// returns command type without the prefix
    fn get_type_without_prefix(command : &str)->Option<CommandType>{
        match command{
            "v" | "version"  =>{Some(CommandType::Version)},
            "h" | "help"     => {Some(CommandType::Help)},
            "c" | "compile"  =>{Some(CommandType::Compile)},
            "o" | "output"   =>{Some(CommandType::Output)},
            "b" | "binary"   =>{Some(CommandType::Binary)},
            "d" | "dump"     =>{Some(CommandType::Dump)},
            "t" | "tree"     =>{Some(CommandType::Tree)}
            "s" | "strict"     =>{Some(CommandType::Strict)}
            "a" | "analyze"     =>{Some(CommandType::Analyze)}
            _=>{None}
        }
    }

    /// print all command help strings
    fn print_all_help(){
        println!("{}\n",CommandType::Help.get_help_string());
        println!("{}\n",CommandType::Version.get_help_string());
        println!("{}\n",CommandType::Compile.get_help_string());
        println!("{}\n",CommandType::Binary.get_help_string());
        println!("{}\n",CommandType::Output.get_help_string());
        println!("{}\n",CommandType::Dump.get_help_string());
        println!("{}\n",CommandType::Tree.get_help_string());
        println!("{}\n",CommandType::Strict.get_help_string());
        println!("{}\n",CommandType::Analyze.get_help_string());
    }

    /// get a formated help string for the CommandType
    fn get_help_string(&self)->String{
        match self{
            CommandType::Help    =>{format!("{:<25} {}\n{:<25}{}","[-h | --help] <command>", "Output help information for specified command",""," or all if none specified.")},
            CommandType::Version =>{format!("{:<25} {}","[-v | --version]", "Output current version information.")},
            CommandType::Compile =>{format!("{:<25} {}\n{:<25}{}","[-c | --compile] <file>", "Compile the specified file. If no -o specified it",""," will output to same directory with same file-name.")},
            CommandType::Output  =>{format!("{:<25} {}","[-o | --output] <file>", "Set the output file of the Compiled program.")},
            CommandType::Binary  =>{format!("{:<25} {}\n{:<25}{}","[-b | --binary]", "(Unsupported) Output the file as a binary instead",""," of a logisim compatible file.")},
            CommandType::Dump    =>{format!("{:<25} {}","[-d | --dump]", "Output all tokens from the Compile target.")},
            CommandType::Tree    =>{format!("{:<25} {}","[-t | --tree]", "Output a statement heirchy of the Compile target.")},
            CommandType::Strict  =>{format!("{:<25} {}\n{:<25}{}","[-s | --strict]", "Strict flag | no register identifiers as labels and",""," everything is case sensitive.")},
            CommandType::Analyze =>{format!("{:<25} {}\n{:<25}{}","[-a | --analyze]", "Run trace analysis on the compiled program.","","--")},
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
    let mut dump_tokens : bool = false;
    let mut show_tree : bool = false;
    let mut strict : bool =  false;
    let mut analyze : bool = false;

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

                    let tokens = lexer.tokenize(strict,source.as_str())?;

                    let mut inner_parser = parser::Parser::create(tokens);


                    let root = inner_parser.generate();
                    if let Err(some) = root{
                        if dump_tokens {
                            println!("Tokens:\n");
                            for token in inner_parser.get_tokens(){
                                println!("{}",token);
                            }
                        }
                        return Err(some);
                    }


                    let inner_program = compiler::Compiler::compile(strict,root.unwrap());
                    if let Err(some) = inner_program{
                        if dump_tokens {
                            println!("Tokens:\n");
                            for token in inner_parser.get_tokens(){
                                println!("{}",token);
                            }
                        }
                        return Err(some);
                    }

                    // println!("{}",inner_program.dump());

                    parser = Some(inner_parser);
                    program = Some(inner_program.unwrap());

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

                println!("format: (ttpc) [COMMAND] <Argument>");
                // display help for commands
                if let Some(arg) = &command.arg{

                    if let Some(t) = CommandType::get_type(arg,false){
                        println!("{}",t.get_help_string());
                    }else{
                        CommandType::print_all_help();

                        println!("\n{} was not a recognized command!",arg);
                    }

                }else{
                    CommandType::print_all_help();
                }

            },
            CommandType::Binary=>{
                // output as binary without logisim header
                binary = true;
            },
            CommandType::Dump =>{
                // dump the tokens to console
                dump_tokens = true;
            },
            CommandType::Tree =>{
                // show the hierchy
                show_tree = true;
            },
            CommandType::Strict =>{
                // apply strict rules
                strict = true;
            },
            CommandType::Analyze =>{
                analyze = true;
            }
        }
        next_command = iter.next();
    }

    if let Some(p) = program{
        let mut options = OpenOptions::new();
        let mut file = swap_e(options.write(true).create(true).truncate(true).append(false).open(output.as_ref().unwrap()))?;

        if dump_tokens{
            println!("Tokens:\n");
            for token in parser.as_ref().unwrap().get_tokens(){
                println!("{}",token);
            }
        }

        if show_tree{
            println!("Parse Tree:\n{}",parser.as_ref().unwrap().root);
        }


        let mut out = String::new();

        if !binary{
            out.push_str("v2.0 raw\n"); // makes this compatible with Logisim v 2.7.1
        }else{
            // do we need to throw an error? probably not but meh
            // TODO: add
            return Err(format!("binary output is currently not supported."));
        }
        out.push_str(p.dump().as_str());

        if let Err(_) = file.write(out.as_bytes()){
            return Err(format!("unable to write to file!"))
        }

        if analyze{

            let vm = vm::VirtualMachine::create();
            vm.load(&p)?;
            vm.run();

            println!("\nRegisters[A:{:0>3},B:{:0>3},C:{:0>3},D:{:0>3}] \nFlags[C:{}, L:{}, Z:{}, O:{}, S:{}]",
                     vm.get_register_data(compiler::Register::A),
                     vm.get_register_data(compiler::Register::B),
                     vm.get_register_data(compiler::Register::C),
                     vm.get_register_data(compiler::Register::D),
                     vm.flags.carry.get(),
                     vm.flags.less_than.get(),
                     vm.flags.zero.get(),
                     vm.flags.overflow.get(),
                     vm.flags.sign.get(),
            );


        }else{

            println!("compiled to file: {:?}" , output.unwrap().as_os_str());
        }

    }


    Ok(())
}

pub fn parse_commands(commands :&mut Args)->Result<Vec<Command>,String>{

    let mut ret_commands: Vec<Command> = Vec::new();

    let mut current = commands.next();

    if current != None && CommandType::get_type(current.as_ref().unwrap(),true)==None{
        // this means we probably have an invalid command
        // or we have the program call as the 0th argument

        current = commands.next();
    }

    while current!=None{
        let command_str : String = current.unwrap();
        let mut next = commands.next();
        if let Some(command_type) = CommandType::get_type(&command_str,true){
            let arg_count = command_type.get_arg_count();
            // println!("command found : {:?} -- arguments needed {}",command_type,arg_count);

            if arg_count == 0 {
                ret_commands.push(Command{command_type,arg:None});
            }else if arg_count == 1{
                // expect an argument if next is not an argument we throw an error
                if next == None || CommandType::get_type(next.as_ref().unwrap(),true)!=None {
                    return Err(format!("Expected an argument for {:?} command",command_type));
                }else{
                    ret_commands.push(Command{command_type,arg:Some(next.unwrap())});
                    next = commands.next();
                }

            }else if arg_count == -1 {
                // we dont expect an argument but if we get one its ok
                if next != None && CommandType::get_type(next.as_ref().unwrap(),true) == None{
                    ret_commands.push(Command{command_type,arg:Some(next.unwrap())});
                    next = commands.next();
                }else{
                    ret_commands.push(Command{command_type,arg:None});
                }

            }
        }else{

            //no command found
            println!("format: (ttpc) [COMMAND] <Argument>");
            CommandType::print_all_help();
            println!();
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
