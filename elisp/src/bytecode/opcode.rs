/*
   Rust study program.
   Bytecode instruction set for the Scheme VM.

   hidekuno@gmail.com
*/
use std::fmt;

use crate::lisp::{Expression, Int};

// Bytecode instructions
#[derive(Debug, Clone)]
pub enum Opcode {
    // ------------------  Push constant --------------------------
    PushInt(Int),
    PushFloat(f64),
    PushBool(bool),
    PushChar(char),
    PushNil,
    PushConst(u16), // constant_pool[index] (string,symbol..)

    // -------------- Local variables (current frame <-> stack) -----
    LoadLocal(u16),  // stack[frame.base + slot] -> push
    StoreLocal(u16), // pop -> stack[frame.base + slot]

    // ------------- upvalue (variables captured by the closure) -------------
    LoadUpvalue(u16),  // closure.upvalues[i] -> push
    StoreUpvalue(u16), // pop -> closure.upvalues[i]

    // -------------- Global variables ---------------------------------------
    LoadGlobal(u32),   // globals[index] -> push
    StoreGlobal(u32),  // pop -> globals[index]  (set!)
    DefineGlobal(u32), // (define x v): pop value, define global, push nil

    // -------------- Stack operations ---------------------------------------
    Pop,
    Dup, // Duplicate Stack Top

    // -------------- Control Flow (Relative Offset) -------------------------
    Jump(i32),        // ip += offset
    JumpIfFalse(i32), // pop(); #f ip += offset
    JumpIfTrue(i32),  // pop(); #t ip += offset

    // --------------- Closure creation --------------------------------------
    MakeClosure(u16),
    CaptureLocal(u16),
    CaptureUpvalue(u16),

    // --------------- Function call -----------------------------------------
    // Stack Layout: [fn, arg0, arg1, ..., argN-1]
    Call(u16),
    TailCall(u16),
    Return,
    Apply,

    // ------- Arithmetic and Comparison: Fast Path (2 Arguments) -------------
    Add,
    Sub,
    Mul,
    Div,
    Neg,
    NumEq,
    NumLt, // b < a
    NumLe, // b <= a
    NumGt, // b > a
    NumGe, // b >= a

    // ----------- List --------------------------------------------
    Cons,
    Car,
    Cdr,
    IsNull,
    Not,

    // --------------- Measure time --------------------------------
    StartTimer,
    StopTimer,

    // ---------------  Top-level terminator ---------------
    Halt,
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Opcode::PushInt(n) => write!(f, "PUSH_INT      {}", n),
            Opcode::PushFloat(n) => write!(f, "PUSH_FLOAT    {}", n),
            Opcode::PushBool(b) => write!(f, "PUSH_BOOL     {}", b),
            Opcode::PushChar(c) => write!(f, "PUSH_CHAR     {:?}", c),
            Opcode::PushNil => write!(f, "PUSH_NIL"),
            Opcode::PushConst(i) => write!(f, "PUSH_CONST    [{}]", i),
            Opcode::LoadLocal(i) => write!(f, "LOAD_LOCAL    {}", i),
            Opcode::StoreLocal(i) => write!(f, "STORE_LOCAL   {}", i),
            Opcode::LoadUpvalue(i) => write!(f, "LOAD_UPVALUE  {}", i),
            Opcode::StoreUpvalue(i) => write!(f, "STORE_UPVALUE {}", i),
            Opcode::LoadGlobal(i) => write!(f, "LOAD_GLOBAL   {}", i),
            Opcode::StoreGlobal(i) => write!(f, "STORE_GLOBAL  {}", i),
            Opcode::DefineGlobal(i) => write!(f, "DEFINE_GLOBAL {}", i),
            Opcode::Pop => write!(f, "POP"),
            Opcode::Dup => write!(f, "DUP"),
            Opcode::Jump(o) => write!(f, "JUMP          {:+}", o),
            Opcode::JumpIfFalse(o) => write!(f, "JUMP_IF_FALSE {:+}", o),
            Opcode::JumpIfTrue(o) => write!(f, "JUMP_IF_TRUE  {:+}", o),
            Opcode::MakeClosure(i) => write!(f, "MAKE_CLOSURE  chunk[{}]", i),
            Opcode::CaptureLocal(i) => write!(f, "CAPTURE_LOCAL {}", i),
            Opcode::CaptureUpvalue(i) => write!(f, "CAPTURE_UV  {}", i),
            Opcode::Call(n) => write!(f, "CALL          {}", n),
            Opcode::TailCall(n) => write!(f, "TAIL_CALL     {}", n),
            Opcode::Return => write!(f, "RETURN"),
            Opcode::Apply => write!(f, "APPLY"),
            Opcode::Add => write!(f, "ADD"),
            Opcode::Sub => write!(f, "SUB"),
            Opcode::Mul => write!(f, "MUL"),
            Opcode::Div => write!(f, "DIV"),
            Opcode::Neg => write!(f, "NEG"),
            Opcode::NumEq => write!(f, "NUM_EQ"),
            Opcode::NumLt => write!(f, "NUM_LT"),
            Opcode::NumLe => write!(f, "NUM_LE"),
            Opcode::NumGt => write!(f, "NUM_GT"),
            Opcode::NumGe => write!(f, "NUM_GE"),
            Opcode::Cons => write!(f, "CONS"),
            Opcode::Car => write!(f, "CAR"),
            Opcode::Cdr => write!(f, "CDR"),
            Opcode::IsNull => write!(f, "IS_NULL"),
            Opcode::Not => write!(f, "NOT"),
            Opcode::StartTimer => write!(f, "START_TIMER"),
            Opcode::StopTimer => write!(f, "STOP_TIMER"),
            Opcode::Halt => write!(f, "HALT"),
        }
    }
}

// The body of a compiled function
#[derive(Clone)]
pub struct Chunk {
    pub code: Vec<Opcode>,
    pub constants: Vec<Expression>, // Constant pool
    pub arity: usize,               // Number of fixed arguments
    pub is_variadic: bool,          // Variable-length arguments
    pub upvalue_count: usize,       // Number of upvalues to capture
    pub name: String,               // function name
}

impl Chunk {
    pub fn new(name: impl Into<String>, arity: usize, is_variadic: bool) -> Self {
        Chunk {
            code: Vec::new(),
            constants: Vec::new(),
            arity,
            is_variadic,
            upvalue_count: 0,
            name: name.into(),
        }
    }

    // Add an instruction and return its index
    pub fn emit(&mut self, op: Opcode) -> usize {
        self.code.push(op);
        self.code.len() - 1
    }

    // Add to the constant pool and return the index
    pub fn add_constant(&mut self, val: Expression) -> u16 {
        self.constants.push(val);
        (self.constants.len() - 1) as u16
    }

    // Backpatch: Determine the offset of the Jump instruction at the patch_offset location
    pub fn patch_jump(&mut self, patch_offset: usize) {
        let target = self.code.len() as i32 - patch_offset as i32 - 1;
        match &mut self.code[patch_offset] {
            Opcode::Jump(ref mut o)
            | Opcode::JumpIfFalse(ref mut o)
            | Opcode::JumpIfTrue(ref mut o) => *o = target,
            _ => panic!("patch_jump: not a jump instruction at {}", patch_offset),
        }
    }

    // Disassembly for debugging
    pub fn disassemble(&self) -> String {
        let mut out = format!(
            "=== chunk: \"{}\"  arity={}  upvalues={} ===\n",
            self.name, self.arity, self.upvalue_count
        );
        for (i, op) in self.code.iter().enumerate() {
            out.push_str(&format!("  {:04}  {}\n", i, op));
        }
        if !self.constants.is_empty() {
            out.push_str("  --- constants ---\n");
            for (i, c) in self.constants.iter().enumerate() {
                out.push_str(&format!("    [{}]  {}\n", i, c));
            }
        }
        out
    }
}
