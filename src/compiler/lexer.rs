
use std::fmt;

#[derive(Clone,PartialEq)]
pub struct Token{
    pub token_type : TokenType,
    pub line : u32,
    pub column : u32,
    pub value : String,
    pub state : LexerState
}

impl Token{
    fn create(token_type:TokenType, line : u32, column : u32, value : String, state : LexerState)->Self{
        Token{token_type,line,column,value,state}
    }
}

impl fmt::Display for Token{

    fn fmt(&self, f:&mut fmt::Formatter)->fmt::Result{
        // ( [STATE] | TokenType - line:col "value" )
        write!(f,"([{:>8} | {:<8}] - {}:{} \"{}\")",format!("{:?}",self.state),format!("{:?}",self.token_type),self.line,self.column,self.value)
    }
}



#[derive(Debug,Clone,Copy,PartialEq)]
pub enum TokenType{
    Op,         // operation nmenomic (includes byte)
    Reg,        // register identifier
    PtrReg,     // reference register identifier - (x) pointer to value at ram location
    Label,      // label  L2:
    Identifier, // Label Identifier
    Number,     // Any Number
    Plus,       // Arithmetic +
    Minus,      // Arithmetic -
    Comma,      // Comma ,
    Dot,        // Dot (Period) .
    Eof         // End of File
}

#[derive(Debug,Clone,Copy,PartialEq)]
pub enum LexerState{
    Base,           // base state when entering new line
    Operand,        // searching for operands registers,labels or expressions
}

impl fmt::Display for LexerState{
    fn fmt(&self, f:&mut fmt::Formatter)->fmt::Result{

        write!(f,"{:?}",self)
    }
}

pub struct Lexer{
    current_state :  LexerState,

}

impl Lexer{

    pub fn create()->Self{
        Lexer{current_state:LexerState::Base}
    }

    pub fn tokenize(&mut self, input : &str)->Result<Vec<Token>,String>{
        let mut line_number = 0;
        let mut tokens = Vec::new();
        let input_string = String::from(input);

        for line in input_string.split('\n'){
            line_number+=1;
            let line_tokens = self.scan_line(&String::from(line),line_number)?;
            tokens.extend(line_tokens);
        }
        tokens.push(Token::create(TokenType::Eof,line_number+1,1,String::from("EOF"),self.current_state));

        Ok(tokens)
    }

    fn scan_line(&mut self, line : &String, line_number : u32)->Result<Vec<Token>,String>{
        // reset state to Base for each line since multiline
        // operations are not possible
        self.current_state = LexerState::Base;
        let mut line_tokens = Vec::new();
        let mut col_number = 0;

        let mut line_chars = line.chars();
        let mut current = line_chars.next();


        while current != None {
            let mut next_token_set = false;
            col_number+=1;
            let current_char = current.unwrap();
            match self.current_state {
                LexerState::Base =>{

                    if current_char.is_alphabetic() {
                        let mut identifier = String::from(current_char);
                        let col_start = col_number;
                        col_number+=1;
                        let mut next_char = line_chars.next();
                        while next_char != None && (next_char.unwrap().is_alphanumeric() || next_char.unwrap() == '_' || next_char.unwrap() == ':' ){

                            identifier.push(next_char.unwrap());

                            next_char = line_chars.next();
                            col_number+=1;
                        }
                        if identifier.chars().nth(identifier.len() - 1).unwrap() == ':'{ //label

                            line_tokens.push(Token::create(TokenType::Label,line_number,col_start,identifier,self.current_state));

                        }else{ //op

                            // FIXME: code should never be reached because once we find the first mnemonic we switch to Operand state
                            //        which allows more identifiers
                            if line_tokens.len() >= 1 {
                                return Err(format!("Too many op mnemonics in line:{}",line_number));
                            }
                            line_tokens.push(Token::create(TokenType::Op,line_number,col_start,identifier,self.current_state));
                        }

                        self.current_state = LexerState::Operand;

                    }else if current_char == '/'{

                        // check if valid comment
                        if let Some('/') = line_chars.next(){
                            break; // valid comment so we skip the rest of the line
                        }else{
                            return Err(format!("Expected '/' on line:{} col:{}",line_number,col_number+1));
                        }
                    }else if current_char.is_numeric(){

                        // shouldnt be a numnber?
                        return Err(format!("[Op codes or Labels cannot start with a number] at line:{} col:{}",line_number,col_number))
                    }else{
                        // skip rest including line endings
                    }

                },
                LexerState::Operand =>{
                    match current_char {
                        ','=>{line_tokens.push(Token::create(TokenType::Comma,line_number,col_number,String::from(current_char),self.current_state));},
                        '(' =>{ //register pointer begin
                            if let Some(c) = line_chars.next(){
                                if let 'a' | 'b' | 'c' | 'd' = c {

                                    if let Some(')') = line_chars.next(){
                                        // we found a valid register in the form (x)
                                        let value = String::from(c);
                                        line_tokens.push(Token::create(TokenType::PtrReg,line_number,col_number,value,self.current_state));
                                        col_number+=2; //we advance the col counter to conpensate
                                        // self.current_state = LexerState::Base;
                                    }else{
                                        return Err(format!("Expected ')' after pointer register identifier line:{} - col:{}",line_number,col_number));
                                    }


                                }else{
                                    //expected valid register col+1

                                    // expected character for register col+1 NOTE: this could mean that a space was detected which may not be fatal
                                    return Err(format!("Invalid Register Identifier on line:{} - col:{}",line_number,col_number+1));
                                }
                            }else{

                                return Err(format!("Expected Register Identifier on line:{} - col:{}",line_number,col_number+1));
                            }
                        },
                        '/'=>{   // comment start
                            // check if valid comment
                            if let Some('/') = line_chars.next(){
                                break; // valid comment so we skip the rest of the line
                            }else{
                                //panic for now but probably do something else later
                                return Err(format!("Error: Expected '/' on line:{} col:{}",line_number,col_number+1));
                            }
                        },
                        '+'=>{line_tokens.push(Token::create(TokenType::Plus,line_number,col_number,String::from(current_char),self.current_state));},
                        '-'=>{line_tokens.push(Token::create(TokenType::Minus,line_number,col_number,String::from(current_char),self.current_state));},
                        '.'=>{line_tokens.push(Token::create(TokenType::Dot,line_number,col_number,String::from(current_char),self.current_state));},
                        ' '=>{}, // skip
                        _=>{
                            if (current_char.is_alphabetic() || current_char == '_') && current_char!= ' '{ // could be register

                                let mut identifier = String::from(current_char);
                                let start_col = col_number;
                                let mut next = line_chars.next();

                                col_number+=1;

                                if next == None || next == Some(',') || next == Some(' ') || next == Some('\n') || next == Some('/') || next == Some('\t') || next == Some('\r'){

                                    // println!("next after register[{}] at(line:{} - col:{}) is {:?}",current_char,line_number,col_number,next);
                                    // NOTE: This compiler will not allow the use of register names as labels
                                    if let 'a' | 'b' | 'c' | 'd' = current_char {
                                        line_tokens.push(Token::create(TokenType::Reg,line_number,start_col,identifier,self.current_state));

                                    }else{
                                      //single letter idenfifier
                                      line_tokens.push(Token::create(TokenType::Identifier,line_number,start_col,identifier,self.current_state));
                                    }

                                    // push next token
                                    next_token_set = true;
                                    current = next;
                                }else{
                                    //label identifier
                                    while next != None && (next.unwrap().is_alphanumeric() || next.unwrap() == '_'){
                                        let c = next.unwrap();
                                        identifier.push(c);
                                        next = line_chars.next();
                                        col_number+=1;
                                    }
                                    // push next
                                    next_token_set = true;
                                    current = next;
                                    line_tokens.push(Token::create(TokenType::Identifier,line_number,start_col,identifier,self.current_state));

                                }
                            }else if current_char.is_numeric(){
                                let mut number = String::from(current_char);
                                let mut next = line_chars.next();
                                let start_col = col_number;
                                col_number+=1;
                                while next != None && next.unwrap().is_numeric(){
                                    let digit = next.unwrap();
                                    number.push(digit);
                                    next = line_chars.next();
                                    col_number+=1;
                                }

                                // set next token
                                next_token_set = true;
                                current = next;

                                line_tokens.push(Token::create(TokenType::Number,line_number,start_col,number,self.current_state));
                            }
                        }
                    }
                }

            }

            if !next_token_set{
                current = line_chars.next();
            }
        }

       Ok(line_tokens)

    }
}
