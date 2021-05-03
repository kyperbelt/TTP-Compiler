use std::{borrow::BorrowMut, cell::{Cell, RefCell}};

use crate::compiler::{Program, Register};



pub struct VirtualMachine{
    program_counter : Cell<u8>,
    register_a      : Cell<u8>,
    register_b      : Cell<u8>,
    register_c      : Cell<u8>,
    register_d      : Cell<u8>,
    ram             : RefCell<[u8;256]>,
    pub program_edge    : Cell<u8>,
    pub flags           : Flags

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

    pub fn reset(&self){
        self.zero.set(false);
        self.less_than.set(false);
        self.overflow.set(false);
        self.sign.set(false);
        self.carry.set(false);
    }
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


        // set less than flag
        if (left_value as i8) < (right_value as i8){
            vm.flags.set_less(true);
        }else{
            vm.flags.set_less(false);
        }

        if result > 255 {
            vm.flags.set_carry(true);
            vm.flags.set_over(true);
        }else{
            vm.flags.set_carry(false);
            vm.flags.set_over(false);
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

        if left_value < right_value {
            vm.flags.set_less(true);
        }else{
            vm.flags.set_less(false);
        }

        if result.abs() > 128 && !(left_value == 0 || right_value == 0){
            vm.flags.set_over(true);
            vm.flags.set_over(true);
        }else{
            vm.flags.set_over(false);
            vm.flags.set_over(false);
        }
    }

    pub fn and(vm : &VirtualMachine, left : Register, right: Register){

    }

    pub fn or(vm : &VirtualMachine, left : Register, right : Register){

    }

    pub fn not(vm : &VirtualMachine, x : Register){

    }

    pub fn right_shift(vm : &VirtualMachine, operand : Register, amount : Register){

    }

    pub fn inc(vm  : &VirtualMachine, x : Register){

    }

    pub fn dec(vm : &VirtualMachine, x : Register){

    }

}


impl VirtualMachine{

    pub fn create()->Self{
        VirtualMachine{
            program_counter : Cell::new(0),
            register_a      : Cell::new(0),
            register_b      : Cell::new(0),
            register_c      : Cell::new(0),
            register_d      : Cell::new(0),
            ram             : RefCell::new([0;256]),
            program_edge    : Cell::new(0),
            flags           : Flags::create()
        }
    }

    /// write the provided data to the memory address
    pub fn write(&self, addr : isize, data : isize){
        let mut arr = self.ram.borrow_mut();
        arr[VirtualMachine::get_wrapped_value(addr)] = VirtualMachine::get_wrapped_value(data) as u8;
    }

    pub fn read(&self, addr: isize)->u8{
        self.ram.borrow()[VirtualMachine::get_wrapped_value(addr)]
    }

    pub fn run(&self){

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

    fn get_register_data(&self, register : Register)->u8{
        match register{
            Register::A => self.register_a.get(),
            Register::B => self.register_b.get(),
            Register::C => self.register_c.get(),
            Register::D => self.register_d.get()
        }
    }

    fn set_register_data(&self, register : Register, data : u8){
        match register{
            Register::A => self.register_a.set(data),
            Register::B => self.register_b.set(data),
            Register::C => self.register_c.set(data),
            Register::D => self.register_d.set(data)
        }
    }

    fn get_wrapped_value(value: isize)->usize{
        let max = 256;
        let mut result : usize = 0;

        if value < 0 {
            result = (max - value.abs() % max ) as usize;
        }else if value >= max{
            result = (value % max) as usize;
        }else{
            result = (value) as usize;
        }

        result
    }
}
