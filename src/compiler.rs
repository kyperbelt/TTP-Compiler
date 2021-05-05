pub mod lexer;
pub mod parser;

use parser::*;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

// operations where x and y are considered registers
#[derive(Debug,PartialEq,Clone,Copy)]
pub enum Ops{

    //
    NoOp,
    Halt,

    //jump operations
    Jumpi,
    JumpLessi,
    JumpOverflowi,
    JumpSigni,
    JumpCarryi,
    JumpZeroi,
    CopyReg,
    JumpLess,
    JumpOverflow,
    JumpSign,
    Jump,
    JumpCarry,
    JumpZero,

    Loadi,
    Load,
    Store,

    //arithmetic
    Add,
    Sub,
    Compare,

    // logical
    RightShift,
    Not,
    And,
    Or,      // or x,y

    Byte,
    //
    Increment, // inc x - increment x register by 1 (no flag change)
    Decrement  // dec x - decrement x register by 1 (no flag change)

}

impl Ops{
    pub fn get_byte_count(&self)->u8{
        match self {
            Ops::Jumpi |
            Ops::JumpLessi |
            Ops::JumpOverflowi |
            Ops::JumpSigni |
            Ops::JumpCarryi |
            Ops::JumpZeroi |
            Ops::Loadi =>{2},
            _=>{1}
        }
    }

    pub fn get_op_param_count(&self)->usize{
        match self {
            Ops::Byte |
            Ops::JumpLessi|
            Ops::JumpCarryi |
            Ops::JumpOverflowi |
            Ops::JumpSigni |
            Ops::JumpZeroi |
            Ops::Jumpi |
            Ops::Increment |
            Ops::Decrement |
            Ops::JumpLess |
            Ops::JumpOverflow |
            Ops::JumpSign |
            Ops::Not |
            Ops::Jump |
            Ops::JumpCarry |
            Ops::JumpZero =>{1},
            Ops::NoOp |
            Ops::Halt =>{0},
            _=>{2}
        }
    }

    pub fn get_op(s : &str)->Option<Ops>{
        match s{
            "nop"=>{Some(Ops::NoOp)},
            "halt"=>{Some(Ops::Halt)},
            "jmpi"=>{Some(Ops::Jumpi)},
            "jli"=>{Some(Ops::JumpLessi)},
            "joi"=>{Some(Ops::JumpOverflowi)},
            "jsi"=>{Some(Ops::JumpSigni)},
            "jci"=>{Some(Ops::JumpCarryi)},
            "jzi"=>{Some(Ops::JumpZeroi)},
            "cpr"=>{Some(Ops::CopyReg)},
            "jl"=>{Some(Ops::JumpLess)},
            "jo"=>{Some(Ops::JumpOverflow)},
            "js"=>{Some(Ops::JumpSign)},
            "ldi"=>{Some(Ops::Loadi)},
            "ld"=>{Some(Ops::Load)},
            "add"=>{Some(Ops::Add)},
            "sub"=>{Some(Ops::Sub)},
            "rsh"=>{Some(Ops::RightShift)},
            "not"=>{Some(Ops::Not)},
            "jmp"=>{Some(Ops::Jump)},
            "jc"=>{Some(Ops::JumpCarry)},
            "jz"=>{Some(Ops::JumpZero)},
            "and"=>{Some(Ops::And)},
            "or"=>{Some(Ops::Or)},
            "cmp"=>{Some(Ops::Compare)},
            "st"=>{Some(Ops::Store)},
            "inc"=>{Some(Ops::Increment)},
            "dec"=>{Some(Ops::Decrement)},
            "byte"=>{Some(Ops::Byte)},
            _=>{
                None
            }
        }
    }
}

#[derive(PartialEq,Debug,Clone,Copy)]
// Register Implementation
pub enum Register{
    A,
    B,
    C,
    D
}

impl Register{

    ///
    pub fn from_char(c : char)->Option<Register>{
        match c {
            'a'|'A' => Some(Register::A),
            'b'|'B' => Some(Register::B),
            'c'|'C' => Some(Register::C),
            'd'|'D' => Some(Register::D),
            _=> None
        }
    }

    pub fn bits(&self)->u8{
        match self{
            Register::A=> 0,
            Register::B=> 1,
            Register::C=> 2,
            Register::D=> 3

        }
    }

    pub fn from_bits(bits : u8)->Register{
        match bits % 4 {
            0 => Register::A,
            1 => Register::B,
            2 => Register::C,
            3 => Register::D,
            _=> Register::D // should never hit this
        }
    }
}

pub struct Instruction{
    pub operation : Ops,    // operation
    pub data : u8
}
impl Instruction{
    fn create(operation: Ops,data : u8)->Instruction{
       Instruction{operation,data}
    }
}

struct LabelInfo<'a>{
    label: String,
    addr : u8,
    expression : Option<&'a Expression>
}

pub struct Program{
    pub instructions : Vec<Instruction>
}

impl Program{

    pub fn dump(&self)->String{
        let mut out = String::new();

        let mut count = 0;
        for instruction in &self.instructions{
            out.push_str(format!("{:02X} ",instruction.data).as_str());
            count+=1;
            if count % 3 == 0 {
                out.push('\n')
            }
        }
        out
    }
}

pub struct Compiler{}
impl Compiler{

    pub fn compile(strict: bool,root: &RootNode)->Result<Program, String>{
        let mut labels : Vec<LabelInfo> = Vec::new();
        let mut program = Program{instructions:Vec::new()};

        // gather all labels on first pass of parse tree
        Compiler::gather_labels(strict,&root.statements,&mut labels)?;

        // go through all statements and convert them to instructions in second pass
        for statement in &root.statements{
            // compile the satement but return early if an error occur
            Compiler::compile_statement(strict,&statement,&mut program, &mut labels)?;
        }
        Ok(program)
    }

    /// compile a statement into an instruction if it is an operation
    /// otherwise it is a label and we submit to the labels list
    /// @return an error if unable to compile statement
    fn compile_statement(strict : bool,statement : &parser::Statement,program : &mut Program, labels : &mut Vec<LabelInfo>)->Result<(),String>{

        if statement.statement_type == parser::StatementType::Operation{
            //TODO: create a getBitPatten function for operations to lower code reuse
            let op = Ops::get_op(statement.raw()).unwrap();
            match op{
                Ops::NoOp =>{
                    program.instructions.push(Instruction::create(op,0));
                },
                Ops::Halt =>{
                    program.instructions.push(Instruction::create(op,1));

                },
                Ops::Byte =>{
                    program.instructions.push(Instruction::create(op,Compiler::evaluate_expression(strict,statement.byte_addr,&statement.expressions[0],labels)?));
                },
                Ops::Jumpi =>{
                    program.instructions.push(Instruction::create(op,0b0100_0000));
                    program.instructions.push(Instruction::create(Ops::Byte,Compiler::evaluate_expression(strict,statement.byte_addr,&statement.expressions[0],labels)?));
                },
                Ops::JumpLessi =>{
                    program.instructions.push(Instruction::create(op,0b0100_0001));
                    program.instructions.push(Instruction::create(Ops::Byte,Compiler::evaluate_expression(strict,statement.byte_addr,&statement.expressions[0],labels)?));
                },
                Ops::JumpOverflowi =>{
                    program.instructions.push(Instruction::create(op,0b0100_0010));
                    program.instructions.push(Instruction::create(Ops::Byte,Compiler::evaluate_expression(strict,statement.byte_addr,&statement.expressions[0],labels)?));
                },
                Ops::JumpSigni =>{
                    program.instructions.push(Instruction::create(op,0b0100_0011));
                    program.instructions.push(Instruction::create(Ops::Byte,Compiler::evaluate_expression(strict,statement.byte_addr,&statement.expressions[0],labels)?));
                },
                Ops::JumpCarryi =>{
                    program.instructions.push(Instruction::create(op,0b0100_0100));
                    program.instructions.push(Instruction::create(Ops::Byte,Compiler::evaluate_expression(strict,statement.byte_addr,&statement.expressions[0],labels)?));

                },
                Ops::JumpZeroi =>{

                    program.instructions.push(Instruction::create(op,0b0100_0101));
                    program.instructions.push(Instruction::create(Ops::Byte,Compiler::evaluate_expression(strict,statement.byte_addr,&statement.expressions[0],labels)?));

                  
                },
                Ops::CopyReg =>{
                    let arg1 = Compiler::evaluate_expression(strict,statement.byte_addr,&statement.expressions[0],labels)? << 2;
                    let arg2 = Compiler::evaluate_expression(strict,statement.byte_addr,&statement.expressions[1],labels)?;

                    let op_code = 0b0101_0000 | arg1 | arg2;
                    program.instructions.push(Instruction::create(op,op_code));


                },
                Ops::JumpLess =>{
                    let arg1 = Compiler::evaluate_expression(strict,statement.byte_addr,&statement.expressions[0],labels)?;
                    let op_code = 0b0110_0000 | arg1;

                    program.instructions.push(Instruction::create(op,op_code));
                },
                Ops::JumpOverflow =>{

                    let arg1 = Compiler::evaluate_expression(strict,statement.byte_addr,&statement.expressions[0],labels)?;
                    let op_code = 0b0110_0100 | arg1;

                    program.instructions.push(Instruction::create(op,op_code));
                },
                Ops::JumpSign =>{

                    let arg1 = Compiler::evaluate_expression(strict,statement.byte_addr,&statement.expressions[0],labels)?;
                    let op_code = 0b0110_1000 | arg1;

                    program.instructions.push(Instruction::create(op,op_code));
                },
                Ops::Loadi =>{

                    let arg1 = Compiler::evaluate_expression(strict,statement.byte_addr,&statement.expressions[0],labels)?;
                    let op_code = 0b0110_1100 | arg1;

                    program.instructions.push(Instruction::create(op,op_code));
                    program.instructions.push(Instruction::create(Ops::Byte,Compiler::evaluate_expression(strict,statement.byte_addr,&statement.expressions[1],labels)?));

                },
                Ops::Load =>{

                    let arg1 = Compiler::evaluate_expression(strict,statement.byte_addr,&statement.expressions[0],labels)? << 2;
                    let arg2 = Compiler::evaluate_expression(strict,statement.byte_addr,&statement.expressions[1],labels)?;

                    let op_code = 0b0111_0000 | arg1 | arg2;
                    program.instructions.push(Instruction::create(op,op_code));
                },
                Ops::Add =>{

                    let arg1 = Compiler::evaluate_expression(strict,statement.byte_addr,&statement.expressions[0],labels)? << 2;
                    let arg2 = Compiler::evaluate_expression(strict,statement.byte_addr,&statement.expressions[1],labels)?;

                    let op_code = 0b1000_0000 | arg1 | arg2;
                    program.instructions.push(Instruction::create(op,op_code));
                },
                Ops::Sub =>{

                    let arg1 = Compiler::evaluate_expression(strict,statement.byte_addr,&statement.expressions[0],labels)? << 2;
                    let arg2 = Compiler::evaluate_expression(strict,statement.byte_addr,&statement.expressions[1],labels)?;

                    let op_code = 0b1001_0000 | arg1 | arg2;
                    program.instructions.push(Instruction::create(op,op_code));
                },
                Ops::RightShift =>{

                    let arg1 = Compiler::evaluate_expression(strict,statement.byte_addr,&statement.expressions[0],labels)? << 2;
                    let arg2 = Compiler::evaluate_expression(strict,statement.byte_addr,&statement.expressions[1],labels)?;

                    let op_code = 0b1010_0000 | arg1 | arg2;
                    program.instructions.push(Instruction::create(op,op_code));
                },
                Ops::Not =>{

                    let arg1 = Compiler::evaluate_expression(strict,statement.byte_addr,&statement.expressions[0],labels)? << 2;
                    let op_code = 0b1011_0000 | arg1;

                    program.instructions.push(Instruction::create(op,op_code));
                },
                Ops::Jump =>{

                    let arg1 = Compiler::evaluate_expression(strict,statement.byte_addr,&statement.expressions[0],labels)? << 2;
                    let op_code = 0b1011_0001 | arg1;

                    program.instructions.push(Instruction::create(op,op_code));
                },
                Ops::JumpCarry =>{

                    let arg1 = Compiler::evaluate_expression(strict,statement.byte_addr,&statement.expressions[0],labels)? << 2;
                    let op_code = 0b1011_0010 | arg1;

                    program.instructions.push(Instruction::create(op,op_code));
                },
                Ops::JumpZero =>{

                    let arg1 = Compiler::evaluate_expression(strict,statement.byte_addr,&statement.expressions[0],labels)? << 2;
                    let op_code = 0b1011_0011 | arg1;

                    program.instructions.push(Instruction::create(op,op_code));
                },
                Ops::And =>{

                    let arg1 = Compiler::evaluate_expression(strict,statement.byte_addr,&statement.expressions[0],labels)? << 2;
                    let arg2 = Compiler::evaluate_expression(strict,statement.byte_addr,&statement.expressions[1],labels)?;

                    let op_code = 0b1100_0000 | arg1 | arg2;
                    program.instructions.push(Instruction::create(op,op_code));
                },
                Ops::Or =>{

                    let arg1 = Compiler::evaluate_expression(strict,statement.byte_addr,&statement.expressions[0],labels)? << 2;
                    let arg2 = Compiler::evaluate_expression(strict,statement.byte_addr,&statement.expressions[1],labels)?;

                    let op_code = 0b1101_0000 | arg1 | arg2;
                    program.instructions.push(Instruction::create(op,op_code));
                },
                Ops::Compare =>{

                    let arg1 = Compiler::evaluate_expression(strict,statement.byte_addr,&statement.expressions[0],labels)? << 2;
                    let arg2 = Compiler::evaluate_expression(strict,statement.byte_addr,&statement.expressions[1],labels)?;

                    let op_code = 0b1110_0000 | arg1 | arg2;
                    program.instructions.push(Instruction::create(op,op_code));
                },
                Ops::Store =>{

                    let arg1 = Compiler::evaluate_expression(strict,statement.byte_addr,&statement.expressions[0],labels)?;
                    let arg2 = Compiler::evaluate_expression(strict,statement.byte_addr,&statement.expressions[1],labels)? << 2;

                    let op_code = 0b1111_0000 | arg1 | arg2;
                    program.instructions.push(Instruction::create(op,op_code));
                },
                Ops::Increment =>{
                    let arg1 = Compiler::evaluate_expression(strict,statement.byte_addr,&statement.expressions[0],labels)?;
                    let arg2 = arg1 << 2;
                    let op_code = 0b1101_0000 | arg1 | arg2;

                    program.instructions.push(Instruction::create(op,op_code));
                },
                Ops::Decrement =>{


                    let arg1 = Compiler::evaluate_expression(strict,statement.byte_addr,&statement.expressions[0],labels)?;
                    let arg2 = arg1 << 2;
                    let op_code = 0b1110_0000 | arg1 | arg2;

                    program.instructions.push(Instruction::create(op,op_code));

                }
            }
        }

        Ok(())
    }

    fn gather_labels<'a>(strict : bool,statements: &'a [Statement], labels : &mut Vec<LabelInfo<'a>>)->Result<(),String>{
        for statement in statements{
            if statement.statement_type == StatementType::Label{
                let mut expression : Option<&'a Expression> = None;
                if statement.expressions.len() > 0 {
                    expression = Some(&statement.expressions[0]);
                }

                // check if we already have the label in the labels list
                // to prevent duplicate labels.
                let check = Compiler::get_label(strict,statement.value.as_str(),labels);

                if let Some(_) = check{
                    return Err(format!("duplicate label [{}] at line:{} col:{}\nTry running in strict mode if you are trying to use case sensitive labels."
                                       ,statement.value.as_str(),statement.line(),statement.col()))
                }
                labels.push(LabelInfo{label: String::from(statement.value.as_str()), addr: statement.byte_addr,expression});

            }
        }

        Ok(())
    }

    /// evaluate an expression and reduce it to a single 8bit value
    fn evaluate_expression(strict : bool,byte_addr: u8,expression : &parser::Expression,labels : &[LabelInfo])->Result<u8,String>{
        match expression.expression_type{
            ExpressionType::Equation => {
                let mut result = std::num::Wrapping(0u8);
                if expression.value == "+" {
                    // add
                    for sub_exp in &expression.expressions{
                        result+=std::num::Wrapping::<u8>(Compiler::evaluate_expression(strict,byte_addr,sub_exp,labels)?);
                    }

                }else{
                    // subtract
                    for sub_exp in &expression.expressions{
                        result-=std::num::Wrapping::<u8>(Compiler::evaluate_expression(strict,byte_addr,sub_exp,labels)?);
                    }

                }

                Ok(result.0)
            },
            ExpressionType::Dot=>{
                Ok(byte_addr)
            },
            ExpressionType::Register=>{

                if let Some(register) = Register::from_char(expression.value.chars().nth(0).unwrap()){
                    Ok(register.bits())
                }else{
                    // not going to happen probably
                    //TODO: more descriptive error
                    Err(format!("Invalid register \"{}\" at line:{} col:{}.",expression.value, expression.line(), expression.col()))
                }

            }
            ExpressionType::LabelPtr=>{
                let mut query = String::new();
                query.push_str(expression.value.as_str());
                query.push(':');
                if let Some(info) = Compiler::get_label(strict,query.as_str(), labels){

                    if let Some(label_exp) = info.expression{

                        Ok(Compiler::evaluate_expression(strict,byte_addr,label_exp,labels)?)
                    }else{

                        Ok(info.addr)
                    }
                }else{
                    Err(format!("Label:[{}] referenced at line:{} col:{} not found.",expression.value,expression.line(),expression.col()))
                }
            },
            ExpressionType::Value=>{
                if let Ok(val) = expression.value.parse::<u8>(){
                    Ok(val)
                }else{
                    let message = format!("Unable to convert \"{}\" to 8bit integer value on line:{} col:{}",expression.value, expression.line(),expression.col());
                    Err(message)

                }
            }
        }
    }


    fn get_label<'a,'b>(strict : bool,label : &str, labels :&'a [LabelInfo])->Option<&'b LabelInfo<'a>>{
        for info in labels{
            if strict &&  info.label.as_str() == label {

                return Some(&info);
            }else if !strict && info.label.to_lowercase() == label.to_lowercase() {
                return Some(&info)
            }
        }

        None
    }
}



pub fn error(message:String){
    eprintln!("Error: \n{}",message);
}

// pub fn warning(message:String){
//     println!("Warning: \n{}",message);
// }
