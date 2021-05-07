use std::cell::{Cell, RefCell};
use std::io::Write;

use crate::compiler::{Program, Register};



pub struct VirtualMachine{
    pub mode          : Cell<u8>,  // MODE FLAGS[? ? ? ? ? ? COLOR_FLAGS ENABLED]
    instruction_count : Cell<usize>,
    program_counter   : Cell<u8>,
    register_a        : Cell<u8>,
    register_b        : Cell<u8>,
    register_c        : Cell<u8>,
    register_d        : Cell<u8>,
    ram               : RefCell<[u8;256]>,
    pub program_edge  : Cell<u8>,
    pub flags         : Flags,
    halt              : Cell<bool>

}


pub struct Flags{
    pub zero        : Cell<bool>,
    pub less_than   : Cell<bool>,
    pub overflow    : Cell<bool>,
    pub sign        : Cell<bool>,
    pub carry       : Cell<bool>,

}

impl Flags{
    fn create()->Self{
        Flags{
            zero        : Cell::new(false),
            less_than   : Cell::new(false),
            overflow    : Cell::new(false),
            sign        : Cell::new(false),
            carry       : Cell::new(false),
        }
    }

    pub fn set_zero(&self, f : bool){
        self.zero.set(f);
    }

    pub fn set_less(&self, f: bool){
        self.less_than.set(f);
    }

    pub fn set_over(&self, f: bool){
        self.overflow.set(f);
    }

    pub fn set_sign(&self, f: bool){
        self.sign.set(f);
    }

    pub fn set_carry(&self, f: bool){
        self.carry.set(f);
    }

    // pub fn reset(&self){
    //     self.zero.set(false);
    //     self.less_than.set(false);
    //     self.overflow.set(false);
    //     self.sign.set(false);
    //     self.carry.set(false);
    // }
}

pub struct ALU{}
impl ALU{

    /// add the values contained in the specified registers
    /// and store it in left register
    /// (updates flags) carry, sign, zero, overflow, less_than
    pub fn add(vm : &VirtualMachine, left : Register, right : Register){

        let left_value  = vm.get_register_data(left) as u8;
        let right_value = vm.get_register_data(right) as u8;

        let result = left_value as isize + right_value as isize;


        let msb_x = (left_value >> 7)  != 0;
        let msb_y = (right_value >> 7) != 0;
        let msb_s = (result as u8 >> 7)  != 0;

        vm.flags.set_over(if (msb_x && msb_y && !msb_s) || (!msb_x && !msb_y && msb_s) {true} else {false});

        // set less than flag
        vm.flags.set_less(if msb_s != vm.flags.overflow.get(){true} else {false});


        if result > 255 {
            vm.flags.set_carry(true);
        }else{
            vm.flags.set_carry(false);
        }

        if (result as u8) == 0{
            vm.flags.set_zero(true);
        }else{
            vm.flags.set_zero(false);
        }

        if (result as i8) < 0{
            vm.flags.set_sign(true);
        }else{
            vm.flags.set_sign(false);
        }


        // store result
        vm.set_register_data(left, result as  u8);
    }

    pub fn sub(vm : &VirtualMachine, left : Register, right: Register){

        // perform same flag as compare so we reuse
        ALU::cmp(vm,left,right);

        let left_value  = vm.get_register_data(left) as i8;
        let right_value = vm.get_register_data(right) as i8;
        vm.set_register_data(left, (left_value as isize - right_value as isize) as u8)
    }

    pub fn cmp(vm : &VirtualMachine, left : Register, right: Register){
        let left_value  = vm.get_register_data(left) as i8;
        let right_value = vm.get_register_data(right) as i8;

        // subtract by upcasting to isize
        // this will allow us to check
        // for flags without having to go into
        // logisim implementation.
        let result : isize = left_value as isize - right_value as isize ;

        if result == 0 {
            vm.flags.set_zero(true);
        }else{
            vm.flags.set_zero(false);
        }

        if (result as i8) < 0 {
            vm.flags.set_sign(true);  // true if negative
        }else{
            vm.flags.set_sign(false);
        }

        //overflow

        let msb_x = (left_value >> 7)  != 0;
        let msb_y = (right_value >> 7) != 0;
        let msb_d = (result as u8 >> 7)  != 0;

        vm.flags.set_over( if (msb_x && !msb_y && !msb_d) || (!msb_x && msb_y && msb_d){true} else {false});

        //less than
        vm.flags.set_less( if vm.flags.overflow.get() != msb_d {true} else {false} );

        // the way we check carry is if we overflow
        // or if the left value is greater than the right value
        // this means that left value is a non negative value
        if (result as u8) > 128 || (left_value  > right_value){
            vm.flags.set_carry(true);
        }else{
            vm.flags.set_carry(false);
        }
    }

    pub fn and(vm : &VirtualMachine, left : Register, right: Register){

        let left_value  = vm.get_register_data(left);
        let right_value = vm.get_register_data(right);

        let result = left_value & right_value;

        vm.flags.set_less(if (left_value as i8) < (right_value as i8) { true } else {false});
        vm.flags.set_zero(if result == 0 {true} else {false});
        vm.flags.set_sign(if (result as i8) < 0 {true} else {false});

        vm.set_register_data(left, result);

    }

    pub fn or(vm : &VirtualMachine, left : Register, right : Register){

        let left_value  = vm.get_register_data(left);
        let right_value = vm.get_register_data(right);

        let result = left_value | right_value;

        vm.flags.set_less(if (left_value as i8) < (right_value as i8) { true } else {false});
        vm.flags.set_zero(if result == 0 {true} else {false});
        vm.flags.set_sign(if (result as i8) < 0 {true} else {false});

        vm.set_register_data(left, result);
    }

    /// not operation on the passed in x Register
    /// NOTE: in the Assembler manual it states that this should update
    ///       the Zero, Less_Than, and Sign Flags but it might be a missprint
    ///       since the operation doesnt seem to change any of the flag registers
    ///       within logisim
    pub fn not(vm : &VirtualMachine, x : Register){

        let value = vm.get_register_data(x);
        let result = !value;

        // TODO: figure out if this is intended
        // FIXME: ---^
        vm.flags.set_less(if (value as i8) < (result as i8) {true} else{false});
        vm.flags.set_zero(if result == 0 {true} else {false});
        vm.flags.set_sign(if (result as i8) < 0 {true} else {false});

        vm.set_register_data(x,result);
    }

    pub fn right_shift(vm : &VirtualMachine, operand : Register, amount : Register){
        let operand_value = vm.get_register_data(operand);
        let amount_value  = vm.get_register_data(amount);
        let result = operand_value >> amount_value;


        //
        vm.flags.set_less(if (operand_value as i8) < (result as i8) {true} else{false});
        vm.flags.set_zero(if result == 0 {true} else {false});
        vm.flags.set_sign(if (result as i8) < 0 {true} else {false});

        vm.set_register_data(operand,result);
    }

    pub fn inc(vm  : &VirtualMachine, x : Register){
        let value = vm.get_register_data(x);
        vm.set_register_data(x,(value as isize +1) as u8);
    }

    pub fn dec(vm : &VirtualMachine, x : Register){
        let value = vm.get_register_data(x);
        vm.set_register_data(x,(value as isize -1) as u8);
    }

}


impl VirtualMachine{

    pub fn create()->Self{
        VirtualMachine{
            mode              : Cell::new(0),
            instruction_count : Cell::new(0),
            program_counter   : Cell::new(0),
            register_a        : Cell::new(0),
            register_b        : Cell::new(0),
            register_c        : Cell::new(0),
            register_d        : Cell::new(0),
            ram               : RefCell::new([0;256]),
            program_edge      : Cell::new(0),
            flags             : Flags::create(),
            halt              : Cell::new(false)
        }
    }

    /// write the provided data to the memory address
    pub fn write(&self, addr : isize, data : isize){
        let mut arr = self.ram.borrow_mut();
        arr[(addr as u8) as usize] = VirtualMachine::get_wrapped_value(data) as u8;
    }

    pub fn read(&self, addr: isize)->u8{
        self.ram.borrow()[(addr as u8) as usize]
    }

    pub fn run(&self,interrupt : bool, after : isize){
        println!("TRACE:");
        while !self.halt.get(){

            // interrupt if interrupt set
            if interrupt && self.instruction_count.get() > after as usize {break;}


            if self.mode.get() & 1 != 0 { // checker mode enabled

                let dark = self.instruction_count.get() % 2 == 0;
                print!("{}{}{}\n",if dark {"\x1b[48;5;245m\x1b[38;5;233m"}else{""},self.run_instruction(),"\x1b[0m");
                std::io::stdout().flush().unwrap();
            }else{
                // no checker mode
                println!("{}{}","",self.run_instruction());
            }


        }

    }

    pub fn load(&self,program : &Program)->Result<(),String>{
        let mut ram = self.ram.borrow_mut();
        let pi_size = program.instructions.len();
        if pi_size > ram.len() {
            return Err(format!("Too many instructions in program. Was [{}]bytes but only [{}]bytes of ram available.",pi_size,ram.len()));
        }

        for i in 0..pi_size{
            ram[i] = program.instructions[i].data;
            self.program_edge.set(i as u8);
        }

        Ok(())
    }

    pub fn get_register_data(&self, register : Register)->u8{
        match register{
            Register::A => self.register_a.get(),
            Register::B => self.register_b.get(),
            Register::C => self.register_c.get(),
            Register::D => self.register_d.get()
        }
    }

    pub fn set_register_data(&self, register : Register, data : u8){
        match register{
            Register::A => self.register_a.set(data),
            Register::B => self.register_b.set(data),
            Register::C => self.register_c.set(data),
            Register::D => self.register_d.set(data)
        }
    }

    fn run_instruction(&self)->String{
        let instruction_count = self.instruction_count.get();
        let pc_value = self.program_counter.get();
        let instruction = self.read(pc_value as isize);

        let left  = Register::from_bits((instruction & 0b0000_1100) >> 2);
        let right = Register::from_bits(instruction & 0b0000_0011);

        let mut left_str    = String::new();
        let mut right_str   = String::new();
        let mut reg_str     = String::new();
        let mut ram_str     = String::new();
        let mut op_str      = String::new();

        match instruction{
            0b0000_0000 =>{ // NOOP
                //no_op;
                op_str.push_str("nop");
            },
            0b0000_0001 =>{ // HALT
                self.halt.set(true);
                op_str.push_str("halt");
            },
            0b0100_0000 =>{ // JUMP IMMEDIATE

                op_str.push_str("jmpi");
                let jmp_location = self.read(pc_value as isize +1) as isize;
                ram_str = format!("RAM_R[{:02x}]={:02x}",pc_value as isize +1,jmp_location);
                self.program_counter.set((jmp_location - 1) as u8);
            },
            0b0100_0001 =>{ // JUMP IMMEDIATE IF LESS

                op_str.push_str("jli");
                let jmp_location = self.read(pc_value as isize +1) as isize;
                if self.flags.less_than.get() {
                    self.program_counter.set((jmp_location - 1) as u8);
                    ram_str = format!("RAM_R[{:02x}]={:02x}",pc_value as isize +1,jmp_location);
                }else{
                    self.program_counter.set((pc_value as isize +1 ) as u8);
                }
            },
            0b0100_0010 =>{ // JUMP IMMEDIATE IF OVERFLOW
                op_str.push_str("joi");
                let jmp_location = self.read(pc_value as isize +1) as isize;
                if self.flags.overflow.get() {
                    self.program_counter.set((jmp_location - 1) as u8);
                    ram_str = format!("RAM_R[{:02x}]={:02x}",pc_value as isize +1,jmp_location);
                }else{
                    self.program_counter.set((pc_value as isize +1 ) as u8);
                }
            },
            0b0100_0011 =>{ // JUMP IMMEDIATE IF SIGN

                op_str.push_str("jsi");
                let jmp_location = self.read(pc_value as isize +1) as isize;
                if self.flags.sign.get() {
                    self.program_counter.set((jmp_location - 1) as u8);
                    ram_str = format!("RAM_R[{:02x}]={:02x}",pc_value as isize +1,jmp_location);
                }else{
                    self.program_counter.set((pc_value as isize +1 ) as u8);
                }
            },
            0b0100_0100 =>{ // JUMP IMMEDIATE IF CARRY

                op_str.push_str("jci");

                let jmp_location = self.read(pc_value as isize +1) as isize;
                if self.flags.carry.get() {
                    self.program_counter.set((jmp_location - 1) as u8);
                    ram_str = format!("RAM_R[{:02x}]={:02x}",pc_value as isize +1,jmp_location);
                }else{
                    self.program_counter.set((pc_value as isize +1 ) as u8);
                }
            },
            0b0100_0101 =>{ // JUMP IMMEDIATE IF ZERO

                op_str.push_str("jzi");


                let jmp_location = self.read(pc_value as isize +1) as isize;
                if self.flags.zero.get() {
                    self.program_counter.set((jmp_location - 1) as u8);
                    ram_str = format!("RAM_R[{:02x}]={:02x}",pc_value as isize +1,jmp_location);
                }else{
                    self.program_counter.set((pc_value as isize +1 ) as u8);
                }

            },
            0b0110_0000 =>{ // JUMP TO A IF LESS


                op_str.push_str("jl");

                left_str = format!("{:?}={:02x}",right,self.get_register_data(right));
                let jmp_location = self.get_register_data(right) as isize;
                if self.flags.less_than.get() {
                    self.program_counter.set((jmp_location - 1) as u8);
                }
            },
            0b0110_0001 =>{ // JUMP TO B IF LESS

                op_str.push_str("jl");

                left_str = format!("{:?}={:02x}",right,self.get_register_data(right));

                let jmp_location = self.get_register_data(right) as isize;
                if self.flags.less_than.get() {
                    self.program_counter.set((jmp_location - 1) as u8);
                }
            },
            0b0110_0010 =>{ // JUMP TO C IF LESS

                op_str.push_str("jl");

                left_str = format!("{:?}={:02x}",right,self.get_register_data(right));
                let jmp_location = self.get_register_data(right) as isize;
                if self.flags.less_than.get() {
                    self.program_counter.set((jmp_location - 1) as u8);
                }
            },
            0b0110_0011 =>{ // JUMP TO D IF LESS

                op_str.push_str("jl");

                left_str = format!("{:?}={:02x}",right,self.get_register_data(right));


                let jmp_location = self.get_register_data(right) as isize;
                if self.flags.overflow.get() {
                    self.program_counter.set((jmp_location - 1) as u8);
                }

            },
            0b0110_0100 =>{ // JUMP TO A IF OVERFLOW

                op_str.push_str("jo");

                left_str = format!("{:?}={:02x}",right,self.get_register_data(right));
                let jmp_location = self.get_register_data(right) as isize;
                if self.flags.overflow.get() {
                    self.program_counter.set((jmp_location - 1) as u8);
                }
            },
            0b0110_0101 =>{ // JUMP TO B IF OVERFLOW

                op_str.push_str("jo");

                left_str = format!("{:?}={:02x}",right,self.get_register_data(right));
                let jmp_location = self.get_register_data(right) as isize;
                if self.flags.overflow.get() {
                    self.program_counter.set((jmp_location - 1) as u8);
                }
            },
            0b0110_0110 =>{ // JUMP TO C IF OVERFLOW

                op_str.push_str("jo");

                left_str = format!("{:?}={:02x}",right,self.get_register_data(right));
                let jmp_location = self.get_register_data(right) as isize;
                if self.flags.overflow.get() {
                    self.program_counter.set((jmp_location - 1) as u8);
                }
            },
            0b0110_0111 =>{ // JUMP TO D IF OVERFLOW

                op_str.push_str("jo");

                left_str = format!("{:?}={:02x}",right,self.get_register_data(right));
                let jmp_location = self.get_register_data(right) as isize;
                if self.flags.overflow.get() {
                    self.program_counter.set((jmp_location - 1) as u8);
                }
            },
            0b0110_1000 =>{ // JUMP TO A IF SIGN

                op_str.push_str("js");

                left_str = format!("{:?}={:02x}",right,self.get_register_data(right));
                let jmp_location = self.get_register_data(right) as isize;
                if self.flags.sign.get() {
                    self.program_counter.set((jmp_location - 1) as u8);
                }
            },
            0b0110_1001 =>{ // JUMP TO B IF SIGN

                op_str.push_str("js");

                left_str = format!("{:?}={:02x}",right,self.get_register_data(right));
                let jmp_location = self.get_register_data(right) as isize;
                if self.flags.sign.get() {
                    self.program_counter.set((jmp_location - 1) as u8);
                }
            },
            0b0110_1010 =>{ // JUMP TO C IF SIGN

                op_str.push_str("js");

                left_str = format!("{:?}={:02x}",right,self.get_register_data(right));
                let jmp_location = self.get_register_data(right) as isize;
                if self.flags.sign.get() {
                    self.program_counter.set((jmp_location - 1) as u8);
                }
            },
            0b0110_1011 =>{ // JUMP TO D IF SIGN

                op_str.push_str("js");

                left_str = format!("{:?}={:02x}",right,self.get_register_data(right));
                let jmp_location = self.get_register_data(right) as isize;
                if self.flags.sign.get() {
                    self.program_counter.set((jmp_location - 1) as u8);
                }
            },
            0b0110_1100 =>{ // LOAD IMMEDIATE TO A
                op_str.push_str("ldi");
                let data : u8 = self.read((pc_value as isize)+1);

                ram_str = format!("RAM_R[{:02x}]={:02x}",(pc_value as isize +1) as u8,data);

                self.program_counter.set((pc_value as isize +1) as u8);
                left_str = format!("{:?}={:02x}",right,self.get_register_data(right));
                reg_str = format!("{:?}={:02x}",right,data);
                self.set_register_data(right, data);

            },
            0b0110_1101 =>{ // LOAD IMMEDIATE TO B
                op_str.push_str("ldi");

                let data : u8 = self.read((pc_value as isize)+1);
                ram_str = format!("RAM_R[{:02x}]={:02x}",(pc_value as isize +1) as u8,data);
                left_str = format!("{:?}={:02x}",right,self.get_register_data(right));
                reg_str = format!("{:?}={:02x}",right,data);
                self.program_counter.set((pc_value as isize +1) as u8);
                self.set_register_data(right, data);
            },
            0b0110_1110 =>{ // LOAD IMMEDIATE TO C
                op_str.push_str("ldi");

                let data : u8 = self.read((pc_value as isize)+1);
                self.program_counter.set((pc_value as isize +1) as u8);
                ram_str = format!("RAM_R[{:02x}]={:02x}",(pc_value as isize +1) as u8,data);
                left_str = format!("{:?}={:02x}",right,self.get_register_data(right));
                reg_str = format!("{:?}={:02x}",right,data);
                self.set_register_data(right, data);
            },
            0b0110_1111 =>{ // LOAD IMMEDIATE TO D
                op_str.push_str("ldi");

                let data : u8 = self.read((pc_value as isize)+1);
                self.program_counter.set((pc_value as isize +1) as u8);
                ram_str = format!("RAM_R[{:02x}]={:02x}",(pc_value as isize +1) as u8,data);
                left_str = format!("{:?}={:02x}",right,self.get_register_data(right));
                reg_str = format!("{:?}={:02x}",right,data);
                self.set_register_data(right, data);
            },
            _=>{
                let instruction_head = (instruction >> 4) as u8;
                if instruction_head == 0b0111{ // LOAD X = RAM[Y]

                    op_str.push_str("load");


                    left_str = format!("{:?}={:02x}",left,self.get_register_data(left));
                    right_str = format!(",{:?}={:02x}",right,self.get_register_data(right));

                    let ram_addr = self.get_register_data(right) as isize;
                    let data = self.read(ram_addr);
                    ram_str = format!("RAM_R[{:02x}]={:02x}",self.get_register_data(right) as isize,data);
                    self.set_register_data(left,data);

                    reg_str = format!("{:?}={:02x}",left,data);


                }else if instruction_head == 0b0101{ // COPY REGISTER X=Y
                    op_str.push_str("cpr");

                    left_str = format!("{:?}={:02x}",left,self.get_register_data(left));
                    right_str = format!(",{:?}={:02x}",right,self.get_register_data(right));
                    self.set_register_data(left,self.get_register_data(right));

                    reg_str = format!("{:?}={:02x}",left,self.get_register_data(left));

                }else if instruction_head == 0b1000 { // ADD X = X+Y
                    op_str.push_str("add");

                    left_str = format!("{:?}={:02x}",left,self.get_register_data(left));
                    right_str = format!(",{:?}={:02x}",right,self.get_register_data(right));

                    ALU::add(self,left,right);
                    reg_str = format!("{:?}={:02x}",left,self.get_register_data(left));
                }else if instruction_head == 0b1001 { // SUB X = X-Y
                    op_str.push_str("sub");

                    left_str = format!("{:?}={:02x}",left,self.get_register_data(left));
                    right_str = format!(",{:?}={:02x}",right,self.get_register_data(right));

                    ALU::sub(self,left,right);
                    reg_str = format!("{:?}={:02x}",left,self.get_register_data(left));
                }else if instruction_head == 0b1010 { // SHIFT RIGHT
                    op_str.push_str("rsh");

                    left_str = format!("{:?}={:02x}",left,self.get_register_data(left));
                    right_str = format!(",{:?}={:02x}",right,self.get_register_data(right));

                    ALU::right_shift(self,left , right);
                    reg_str = format!("{:?}={:02x}",left,self.get_register_data(left));

                }else if instruction_head == 0b1011 {

                    let butt = (instruction & 0b0000_0011) as u8;

                    if butt == 0 {           // NOT  X
                        op_str.push_str("not");
                        left_str = format!("{:?}={:02x}",left,self.get_register_data(left));
                        ALU::not(self, left);
                        reg_str = format!("{:?}={:02x}",left,self.get_register_data(left));
                    }else if butt == 1 {     // JUMP TO X

                        op_str.push_str("jmp");
                        left_str = format!("{:?}={:02x}",left,self.get_register_data(left));
                        let jmp_location = self.get_register_data(left);
                        self.program_counter.set((jmp_location - 1) as u8);


                    }else if butt == 2 {     // JUMP TO X IF CARRY

                        op_str.push_str("jc");
                        left_str = format!("{:?}={:02x}",left,self.get_register_data(left));
                        let jmp_location = self.get_register_data(left) as isize;
                        if self.flags.carry.get() {
                            self.program_counter.set((jmp_location - 1) as u8);
                        }

                    }else if butt == 3 {     // JUMP TO X IF ZERO

                        op_str.push_str("jz");
                        left_str = format!("{:?}={:02x}",left,self.get_register_data(left));
                        let jmp_location = self.get_register_data(left) as isize;
                        if self.flags.zero.get() {
                            self.program_counter.set((jmp_location - 1) as u8);
                        }
                    }

                }else if instruction_head == 0b1100{  // AND X = X&Y
                    op_str.push_str("and");

                    left_str = format!("{:?}={:02x}",left,self.get_register_data(left));
                    right_str = format!(",{:?}={:02x}",right,self.get_register_data(right));
                    ALU::and(self,left,right);

                    reg_str = format!("{:?}={:02x}",left,self.get_register_data(left));
                }else if instruction_head == 0b1101{ //  OR  X = X|Y

                    left_str = format!("{:?}={:02x}",left,self.get_register_data(left));
                    if left == right{
                        ALU::inc(self,left);
                        op_str.push_str("inc");

                    }else{
                        right_str = format!(",{:?}={:02x}",right,self.get_register_data(right));
                        op_str.push_str("or");
                        ALU::or(self, left, right);
                    }
                    reg_str = format!("{:?}={:02x}",left,self.get_register_data(left));

                }else if instruction_head == 0b1110{ // COMPARE X - Y


                    if left == right { // DECREMENT
                        op_str.push_str("dec");
                        // no flag change
                        left_str = format!("{:?}={:02x}",left,self.get_register_data(left));
                        ALU::dec(self,left);
                        reg_str = format!("{:?}={:02x}",left,self.get_register_data(left));

                    }else{          // COMP CONT
                        op_str.push_str("cmp");

                        left_str = format!("{:?}={:02x}",left,self.get_register_data(left));
                        right_str = format!(",{:?}={:02x}",right,self.get_register_data(right));

                        ALU::cmp(self,left,right);
                    }

                }else if instruction_head == 0b1111{ // STORE RAM[Y]= X

                    op_str.push_str("st");

                    left_str = format!("{:?}={:02x}",right,self.get_register_data(right));
                    right_str = format!(",{:?}={:02x}",left,self.get_register_data(left));


                    let store_addr = self.get_register_data(right) as isize;
                    let data = self.get_register_data(left);

                    ram_str = format!("RAM_W[{:02x}]={:02x}",store_addr,data);
                    self.write(store_addr, data as isize);
                }

            }


        }

        // increase pc -- wraps
        self.program_counter.set((self.program_counter.get() as isize + 1) as u8);
        self.instruction_count.set(instruction_count +1);


        let c = self.flags.carry.get();
        let z = self.flags.zero.get();
        let s = self.flags.sign.get();
        let o = self.flags.overflow.get();
        let l = self.flags.less_than.get();

        let dark = self.mode.get() & 1 != 0 && instruction_count % 2 == 0;
        let color_flags = (self.mode.get() & 2) != 0;

        //        000 : PC[00]->(OP[    ] A=00,B=00) | A=FF | RAM_R[00]=00 | FLAGS[ c=0 z=0 s=0 o=0 l=0 ]
        format!("{:0>3} : PC[{:02X}]->(OP[{:<4}] {:<4}{:<5}) | {:<4} | {:<12} | FLAGS[ c={} z={} s={} o={} l={} ]",
                instruction_count,
                pc_value,
                op_str,
                left_str,
                right_str,
                reg_str,
                ram_str,
                format!("{}{}{}",
                        if color_flags && c {"\x1b[38;5;46m"}
                        else if color_flags && !c{"\x1b[38;5;196m"}
                        else{""}
                        ,c as u8,
                        if dark && color_flags{"\x1b[38;5;233m"}
                        else if !dark && color_flags{"\x1b[0m"}
                        else{""}),
                format!("{}{}{}",
                        if color_flags && z {"\x1b[38;5;46m"}
                        else if color_flags && !z{"\x1b[38;5;196m"}
                        else{""}
                        ,z as u8,
                        if dark && color_flags{"\x1b[38;5;233m"}
                        else if !dark && color_flags{"\x1b[0m"}
                        else{""}),
                format!("{}{}{}",if color_flags && s {"\x1b[38;5;46m"}
                        else if color_flags && !s{"\x1b[38;5;196m"}
                        else{""}
                        ,s as u8,
                        if dark && color_flags{"\x1b[38;5;233m"}
                        else if !dark && color_flags{"\x1b[0m"}
                        else{""}),
                format!("{}{}{}",
                        if color_flags && o {"\x1b[38;5;46m"}
                        else if color_flags && !o{"\x1b[38;5;196m"}
                        else{""}
                        ,o as u8,
                        if dark && color_flags{"\x1b[38;5;233m"}
                        else if !dark && color_flags{"\x1b[0m"}
                        else{""}),
                format!("{}{}{}",
                        if color_flags && l {"\x1b[38;5;46m"}
                        else if color_flags && !l{"\x1b[38;5;196m"}
                        else{""}
                        ,l as u8,
                        if dark && color_flags{"\x1b[38;5;233m"}
                        else if !dark && color_flags{"\x1b[0m"}
                        else{""}),
        )

    }



    fn get_wrapped_value(value: isize)->usize{
        let max = 256;
        let mut _result : usize = 0_usize;

        if value < 0 {
            _result = (max - value.abs() % max ) as usize;
        }else if value >= max{
            _result = (value % max) as usize;
        }else{
            _result = (value) as usize;
        }

        _result
    }
}
