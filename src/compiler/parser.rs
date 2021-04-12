use crate::compiler;
use crate::compiler::lexer::*;
use std::fmt;


// *******************************
// Defs
// *******************************
#[derive(PartialEq,Debug,Clone,Copy)]
pub enum StatementType{
    Operation,              // One of the several operations in the Op enum
    Label                   // A label counts as a statement since it can be followed by an expresison
}

#[derive(Debug,Clone,Copy,PartialEq)]
pub enum ExpressionType{
    Dot,                    // ( . ) current PC value
    Value,                  // numerical value
    LabelPtr,               // Label pointer - we evaluate after
    Register,               // Register Pointer or Register (doesnt matter at this point)
    Equation                // an Equation(+ or -) with two preceeding Label or Ptr Expressions
}

pub struct Statement{
    pub value : String,
    pub statement_type : StatementType,
    pub expressions : Vec<Expression>,
    line : u32,
    col: u32,
    pub byte_addr : u8
}

pub struct RootNode{
    pub statements:Vec<Statement>,
}

#[derive(PartialEq)]
pub struct Expression{
    pub expression_type : ExpressionType,
    pub expressions : Vec <Expression>,
    pub value : String,
    line : u32,
    col : u32
}

pub struct Parser<'a>{
    root : RootNode,
    expression_mode: bool,
    tokens :  &'a Vec<Token>,
    current_index : usize,
}

pub trait DebugInfo{
    fn line(&self)->u32;
    fn col(&self)->u32;
    fn raw(&self)->&str;
    fn dump(&self,level : usize)->String;
}

// *******************************
// Implementations
// *******************************

impl<'a> Parser<'a>{

    pub fn create(tokens:&'a Vec<Token>)->Self{
        Parser{root:RootNode{statements:Vec::new()},expression_mode:false,tokens,current_index:0}
    }

    pub fn reset(&mut self){
        self.current_index = 0;
    }

    pub fn generate(&mut self)->Result<&RootNode,String>{
        let parser = self;

        parser.reset();

        parser.root = RootNode{statements:Vec::new()};

        let mut next_option : Option<&Token> = Parser::next(&parser.tokens,&mut parser.current_index);
        let mut byte_counter : u8 = 0;
        while next_option != None && next_option.unwrap().token_type != TokenType::Eof{
            // either operation or label
            let token : &Token = next_option.unwrap();
            let mut statement = Statement::new();
            let mut op_type = compiler::Ops::get_op(statement.value.as_str());

            if token.token_type == TokenType::Label{
                statement = Statement{

                    value:String::from(token.value.as_str()),
                    statement_type:StatementType::Label,expressions:Vec::new(),
                    line : token.line,
                    col : token.column,
                    byte_addr: byte_counter
                };
            }else if token.token_type == TokenType::Op{
                statement = Statement{
                    value:String::from(token.value.as_str()),
                    statement_type:StatementType::Operation,expressions:Vec::new(),
                    line : token.line,
                    col : token.column,
                    byte_addr: byte_counter
                };

                // add appropriate number of bytes to the byte_counter
                if let Some(op) = compiler::Ops::get_op(statement.value.as_str()){
                    op_type = Some(op);
                    byte_counter+=op.get_byte_count();
                }else{
                    // FIXME: Throw some error because of invalid operator
                }
            }else{

                // error occursed == was expecting statement label or operation
                return Err(format!("Was expecting label or operation on line:{} col:{} | got {:?} instead.", token.line,token.column,token.token_type));
            }


            while !Parser::next_token_is(parser.current_index,&parser.tokens,&[TokenType::Op,TokenType::Label,TokenType::Eof]){
                if let Some(exp) = Parser::parse_expression(&mut parser.current_index,parser.tokens){

                    statement.expressions.push(exp);
                }else{
                    // something has gone wrong we were expecting an expression of the following:
                    //  - Identifier
                    //  - Register / Register Pointer
                    //  - Number
                    //  - Dot
                    //  TODO: figure out if this should cause an exit
                   return Err(format!("Unable to parse expression"));
                }
                parser.expect_token(TokenType::Comma);//eats commas for breakfast
            }

            // if the statement is a label and there was an expression after it
            if statement.statement_type == StatementType::Label {

                if statement.expressions.len() > 0{

                    if statement.expressions.len() > 1{
                        // we only expect 1 expression for a byte
                        // if we have more throw an error
                        // FIXME
                    }

                    //FIXME TODO:: instead of inserting a byte here we just replace all the values from a label directly into its reference

                    // byte_counter+=1; // label has an expression so it consomes one byte
                    // let mut byte_statement = Statement::new();
                    // byte_statement.line = statement.line;
                    // byte_statement.col = statement.col;
                    // byte_statement.value = String::from("byte");
                    // byte_statement.expressions.push(statement.expressions.pop().unwrap());
                    &parser.root.statements.push(statement);
                    // &parser.root.statements.push(byte_statement);
                }else{
                    &parser.root.statements.push(statement);
                   // byte_counter-=1; o
                }


            }else{

                if op_type!=None && statement.expressions.len() != op_type.unwrap().get_op_param_count() {
                    // expected parameters is not the same as the supplied expressions

                    return Err(format!("Invalid number of parameters supplied for Operation[{:?}] on line:{}.\nExpected {} but instead got {}",
                                           op_type.unwrap(),statement.line,op_type.unwrap().get_op_param_count(),
                                            statement.expressions.len()));

                }else if op_type == None{
                    return Err(format!("Invalid op nmemonic [{}] for op_code: {:?} statement_type:{:?}",statement.value,op_type,statement.statement_type));
                }

                &parser.root.statements.push(statement);


            }


            next_option = Parser::next(&parser.tokens,&mut parser.current_index);
        }

        Ok(&parser.root)
    }

    /// get an expression from the next set of tokens
    /// NOTE: could be that there is not expression to return
    fn parse_expression(current_index : &mut usize, tokens: &[Token])->Option<Expression>{


        let mut expression_stack : Vec<Expression> = Vec::new();

        let mut token;
        loop{

            token = Parser::next(tokens,current_index).unwrap();
            let mut exp_type = ExpressionType::Value;

            match token.token_type{
                TokenType::Dot=>{exp_type = ExpressionType::Dot},
                TokenType::Identifier=>{exp_type = ExpressionType::LabelPtr},
                TokenType::Reg | TokenType::PtrReg =>{exp_type = ExpressionType::Register},
                TokenType::Plus | TokenType::Minus =>{exp_type = ExpressionType::Equation},
                _=>{}
            }

            expression_stack.push(Expression{
                expression_type : exp_type,
                value : String::from(token.value.as_str()),
                expressions : Vec::new(),
                line : token.line,
                col : token.column
            });
            // do while clause
            if Parser::next_token_is(*current_index, tokens, &[TokenType::Op,TokenType::Comma,TokenType::Label,TokenType::Eof])
            {break;}
        }

        // make sure that last expression is aithmetic expression if there is more than one and
        // if there is more than one then there must be at least 3
        if expression_stack.len() > 1 && expression_stack.len() % 2 != 0 {

            if let ExpressionType::Equation = expression_stack.last()?.expression_type {

                let mut retexp = expression_stack.pop()?;
                let mut path : Vec<usize> = Vec::new();
                let mut last_type = retexp.expression_type;
                while expression_stack.len() > 0 {

                    let exp = expression_stack.pop()?;
                    let mut nlx = &mut retexp; //next leader expressions are by default on the root expression
                    for i in &path{
                        nlx = &mut nlx.expressions[*i];
                    }
                    // println!("current_leader:{:?}[{}] current_token_value:{}",nlx.expression_type,nlx.raw() ,exp.raw());
                    let exp_type = exp.expression_type;

                    if exp.expression_type == ExpressionType::Equation{
                        if last_type == ExpressionType::Equation{
                            // coult not have two equation expressions in a row
                            // FIXME throw some error
                            compiler::error(format!("Unexpected Arithmetic Symbol at line:({}) col:({})",exp.line(),exp.col()));
                        }else{
                            let mut nlx = &mut retexp; //next leader expressions are by default on the root expression
                            for i in &path{
                                nlx = &mut nlx.expressions[*i];
                            }
                            let new_leader_index = nlx.expressions.len();
                            nlx.expressions.push(exp);
                            path.push(new_leader_index);
                        }

                    }else{

                        nlx.expressions.push(exp)

                    }



                    last_type = exp_type;
                }


                Some(retexp)
            }else{

                // FIXME
                // return none for now but should probably error out
                compiler::error(format!("Invalid Expression on line:({})",expression_stack.last()?.line));
                None

            }


        }else if expression_stack.len() == 1{

            Some(expression_stack.pop()?)

        }else{

            compiler::error(format!("Unbalanced Arithmetic on line:({})",expression_stack.last()?.line));
            None
        }

    }


    /// expects to find the specified token
    /// if the token is not found returns None
    /// if it is found then it 'consumes' it and
    /// returns a reference to the token
    ///
    /// It advances the current index if found
    fn expect_token(&mut self, token_type : TokenType)->Option<&Token>{

        if self.current_index < self.tokens.len() &&
            self.tokens[self.current_index].token_type == token_type
        {
            let t : &Token = &self.tokens[self.current_index];
            self.current_index+=1;
            Some(t)
        }else{
            None
        }

    }

    /// we return a ref to the next token
    /// assuming at least there is an EOF token
    fn next(tokens: &'a [Token],current_index : &mut usize)->Option<&'a Token>{
        if *current_index >= tokens.len(){
            None
        }else{
            let token: &'a Token = &tokens[*current_index];
            *current_index+=1;
            Some(token)
        }

    }

    /// check if the next few tokens are the same as the valid types
    /// this does not consume the token
    fn next_tokens_are(current_index: usize, tokens : &[Token], valid_types: &[TokenType])->bool{

        let mut temp_index = current_index;

        for t in valid_types{
            if &tokens[temp_index].token_type != t{
                return false
            }
            temp_index+=1;
        }

        true

    }

    /// check if the next token is of the valid types
    /// this does not consume the token
    fn next_token_is(current_index : usize, tokens : &[Token],valid_types : &[TokenType])->bool{

        let token = &tokens[current_index];

        for t in valid_types {
            if t == &token.token_type {
                return true
            }
        }

        false

    }
}

impl DebugInfo for Expression{
    fn line(&self)->u32{self.line}
    fn col(&self)->u32{self.col}
    fn raw(&self)->&str{self.value.as_str()}
    fn dump(&self,level : usize)->String{
        let mut string = format!("{:->count$}{:<10}:\"{}\"","",format!("({:?})",self.expression_type),self.raw(),count=level*4);
        for exp in &self.expressions{
            string.push('\n');
            string.push_str(exp.dump(level+1).as_str());
        }

        string
    }
}


impl Statement{
    fn new()->Statement{
        Statement{byte_addr:0, col : 0, line : 0,expressions : Vec::new(), statement_type: StatementType::Operation, value: String::new()}
    }
}
impl DebugInfo for Statement{
    fn line(&self)->u32{self.line}
    fn col(&self)->u32{self.col}
    fn raw(&self)->&str{self.value.as_str()}
    fn dump(&self,level : usize)->String{
        let mut string = format!("({:?}):\"{}\"",self.statement_type,self.raw());

        for exp in &self.expressions{
            string.push('\n');
            string.push_str(exp.dump(level+1).as_str());
        }

        string
    }


}

impl fmt::Display for RootNode{

    fn fmt(&self, f: &mut fmt::Formatter)->fmt::Result{
        let mut s  = String::new();
        for statement in &self.statements{
            s.push('\n');
            s.push_str(statement.dump(0).as_str());
        }

        write!(f,"{}",s)
    }
}
