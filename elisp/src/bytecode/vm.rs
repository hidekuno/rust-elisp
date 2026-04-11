/*
   Rust study program.
   Bytecode Virtual Machine for the Scheme interpreter.

   Stack layout within a call frame:
     value_stack[frame.base + 0]       = param[0]
     value_stack[frame.base + 1]       = param[1]
     ...
     value_stack[frame.base + arity-1] = param[arity-1]
     value_stack[frame.base + arity..] = local defines + temporaries

   Call convention: [fn, arg0, arg1, ..., argN-1] on the stack
   After Call(N):  new frame.base = old stack_top - N

   hidekuno@gmail.com
*/
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::time::Instant;

#[cfg(not(feature = "thread"))]
use std::rc::Rc;
#[cfg(feature = "thread")]
use std::sync::{Arc, Mutex};

#[allow(unused_imports)]
use log::{debug, error, info, warn}; // ex.) export RUST_LOG=debug

use crate::bytecode::compiler::Compiler;
use crate::bytecode::opcode::{Chunk, Opcode};
use crate::create_error;
use crate::create_error_value;
use crate::lisp::{parse, tokenize};
use crate::lisp::{
    BasicBuiltIn, Environment, ErrCode, Error, Expression, ExtFunctionRc, Int, ResultExpression,
};
use crate::mut_env;
use crate::reference_env;
use crate::reference_obj;

//========================================================================
pub enum Upvalue {
    Open(usize), // Values are on the stack (absolute index)
    Closed(Val), // Captured (saved to the heap)
}

#[cfg(not(feature = "thread"))]
pub type UpvalueRef = Rc<RefCell<Upvalue>>;
#[cfg(feature = "thread")]
pub type UpvalueRef = Arc<Mutex<Upvalue>>;

pub struct VmClosure {
    pub chunk_idx: usize,
    pub upvalues: Vec<UpvalueRef>,
}
#[cfg(not(feature = "thread"))]
pub type ClosureRef = Rc<VmClosure>;
#[cfg(feature = "thread")]
pub type ClosureRef = Arc<VmClosure>;

pub struct CallFrame {
    pub closure: ClosureRef,
    pub ip: usize,
    pub base: usize, // value_stack[base..] is local area
}

#[cfg(not(feature = "thread"))]
fn new_upvalue_ref(u: Upvalue) -> UpvalueRef {
    Rc::new(RefCell::new(u))
}
#[cfg(not(feature = "thread"))]
fn new_closure_ref(c: VmClosure) -> ClosureRef {
    Rc::new(c)
}

#[cfg(feature = "thread")]
fn new_upvalue_ref(u: Upvalue) -> UpvalueRef {
    Arc::new(Mutex::new(u))
}
#[cfg(feature = "thread")]
fn new_closure_ref(c: VmClosure) -> ClosureRef {
    Arc::new(c)
}

// VM runtime value
#[derive(Clone)]
pub enum Val {
    Int(Int),
    Float(f64),
    Bool(bool),
    Char(char),
    Nil,
    Scheme(Expression),                  // string list pair vector ..
    Closure(ClosureRef),                 // Compiled Closure
    Builtin(BasicBuiltIn, &'static str), // Built-in functions
    BuiltinExt(ExtFunctionRc),           // Ext Built-in functions
}

impl Val {
    // #f is false; everything else is true
    #[inline]
    pub fn is_false(&self) -> bool {
        matches!(self, Val::Bool(false))
    }
}

impl fmt::Display for Val {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Val::Int(n) => write!(f, "{}", n),
            Val::Float(n) => write!(f, "{}", n),
            Val::Bool(true) => write!(f, "#t"),
            Val::Bool(false) => write!(f, "#f"),
            Val::Char(c) => write!(f, "{}", Expression::Char(*c)),
            Val::Nil => write!(f, "nil"),
            Val::Scheme(e) => write!(f, "{}", e),
            Val::Closure(c) => write!(f, "<closure:{}>", c.chunk_idx),
            Val::Builtin(_, s) => write!(f, "<{}> BuildIn Function", s),
            Val::BuiltinExt(_) => write!(f, "BuildIn Function Ext"),
        }
    }
}

impl From<Expression> for Val {
    fn from(e: Expression) -> Self {
        match e {
            Expression::Integer(n) => Val::Int(n),
            Expression::Float(f) => Val::Float(f),
            Expression::Boolean(b) => Val::Bool(b),
            Expression::Char(c) => Val::Char(c),
            Expression::Nil() => Val::Nil,
            Expression::BuildInFunction(s, f) => Val::Builtin(f, s),
            Expression::BuildInFunctionExt(f) => Val::BuiltinExt(*f),
            e => Val::Scheme(e),
        }
    }
}

impl From<Val> for Expression {
    fn from(v: Val) -> Self {
        match v {
            Val::Int(n) => Expression::Integer(n),
            Val::Float(f) => Expression::Float(f),
            Val::Bool(b) => Expression::Boolean(b),
            Val::Char(c) => Expression::Char(c),
            Val::Nil => Expression::Nil(),
            Val::Scheme(e) => e,
            Val::Closure(_) => Expression::Nil(), // VM closure cannot be converted to an Expression
            Val::Builtin(f, s) => Expression::BuildInFunction(s, f),
            Val::BuiltinExt(f) => Expression::BuildInFunctionExt(Box::new(f)),
        }
    }
}

pub struct Vm {
    pub stack: Vec<Val>,            // ex. (+ 1 2)
    pub call_stack: Vec<CallFrame>, // frame when call function
    pub globals: Vec<Val>,
    global_idx_to_name: Vec<String>, // The variable name for `globals[i]` (for lazy lookup)
    pub chunks: Vec<Chunk>,
    open_upvalues: Vec<(usize, UpvalueRef)>, // (abs_stack_idx, upvalue)
    timer_stack: Vec<Instant>,               // StartTimer/StopTimer
    env: Environment,
}

impl Vm {
    // Generating a VM from the compiler and runtime environment
    pub fn new(compiler: &Compiler, env: &Environment) -> Self {
        let name_map = compiler.global_names();
        let global_count = name_map.len();

        // Initialize the global table in index order
        let mut globals = vec![Val::Nil; global_count];
        let mut global_idx_to_name = vec![String::new(); global_count];

        for (name, &idx) in name_map {
            global_idx_to_name[idx as usize] = name.clone();
            // Remove built-in functions from the environment
            if let Some((s, f)) = env.get_builtin_func(name) {
                globals[idx as usize] = Val::Builtin(f, s);
            } else if let Some(f) = env.get_builtin_ext_func(name) {
                globals[idx as usize] = Val::BuiltinExt(f);
            }
        }

        Vm {
            stack: Vec::with_capacity(256),
            call_stack: Vec::with_capacity(64),
            globals,
            global_idx_to_name,
            chunks: compiler.chunks.clone(),
            open_upvalues: Vec::new(),
            timer_stack: Vec::new(),
            env: env.clone(),
        }
    }

    // --- operate global variable ------------------------------

    fn set_global(&mut self, idx: u32, val: Val) {
        let i = idx as usize;
        if i >= self.globals.len() {
            self.globals.resize(i + 1, Val::Nil);
        }
        self.globals[i] = val;
    }

    fn get_global(&self, idx: u32) -> Val {
        self.globals.get(idx as usize).cloned().unwrap_or(Val::Nil)
    }

    // --- upvalue managed ---------------------------------------

    fn get_or_create_open_upvalue(&mut self, abs_idx: usize) -> UpvalueRef {
        for (i, uv) in &self.open_upvalues {
            if *i == abs_idx {
                return uv.clone();
            }
        }
        let uv = new_upvalue_ref(Upvalue::Open(abs_idx));
        self.open_upvalues.push((abs_idx, uv.clone()));
        uv
    }

    // At the end of the frame: Close all open upvalues of type base or higher
    fn close_upvalues(&mut self, base: usize) {
        //  Close the items to be closed first, then release the borrow
        let to_close: Vec<(usize, UpvalueRef)> = self
            .open_upvalues
            .iter()
            .filter(|(i, _)| *i >= base)
            .map(|(i, uv)| (*i, uv.clone()))
            .collect();

        for (abs_idx, uv) in &to_close {
            let val = self.stack[*abs_idx].clone();
            *mut_env!(uv) = Upvalue::Closed(val);
        }

        self.open_upvalues.retain(|(i, _)| *i < base);
    }

    // --- main loop ---------------------------------------------

    pub fn run(&mut self, chunk_idx: usize) -> Result<Val, Box<Error>> {
        // push top level frame
        let top_closure = new_closure_ref(VmClosure {
            chunk_idx,
            upvalues: vec![],
        });
        self.call_stack.push(CallFrame {
            closure: top_closure,
            ip: 0,
            base: 0,
        });

        loop {
            // get ip,chunk_idx from frame, forward
            let (ci, ip) = {
                let frame = self.call_stack.last_mut().unwrap();
                let ci = frame.closure.chunk_idx;
                let ip = frame.ip;
                frame.ip += 1;
                (ci, ip)
            };

            // fetch instruction
            let op = self.chunks[ci].code[ip].clone();
            #[cfg(debug_assertions)]
            {
                self.debug_print_system(&op);
            }
            match op {
                // --- Measurement -------------------------------
                Opcode::StartTimer => {
                    self.timer_stack.push(Instant::now());
                }
                Opcode::StopTimer => {
                    if let Some(start) = self.timer_stack.pop() {
                        let d = start.elapsed();
                        println!("{}.{:03}(s)", d.as_secs(), d.subsec_millis());
                    }
                    // save stack top
                }

                // --- endpoint ----------------------------------
                Opcode::Halt => {
                    return Ok(self.stack.pop().unwrap_or(Val::Nil));
                }

                // --- push constant ------------------------------
                Opcode::PushInt(n) => self.stack.push(Val::Int(n)),
                Opcode::PushFloat(f) => self.stack.push(Val::Float(f)),
                Opcode::PushBool(b) => self.stack.push(Val::Bool(b)),
                Opcode::PushChar(c) => self.stack.push(Val::Char(c)),
                Opcode::PushNil => self.stack.push(Val::Nil),
                Opcode::PushConst(i) => {
                    let val = self.chunks[ci].constants[i as usize].clone();
                    self.stack.push(Val::from(val));
                }

                // --- local variable -----------------------------
                Opcode::LoadLocal(slot) => {
                    let base = self.call_stack.last().unwrap().base;
                    let abs = base + slot as usize;
                    let val = self.stack[abs].clone();
                    self.stack.push(val);
                }
                Opcode::StoreLocal(slot) => {
                    let val = self.stack.pop().unwrap();
                    let base = self.call_stack.last().unwrap().base;
                    let abs = base + slot as usize;
                    // When extending the stack using an internal `define`
                    if abs >= self.stack.len() {
                        self.stack.resize(abs + 1, Val::Nil);
                    }
                    self.stack[abs] = val;
                }

                // --- upvalue ------------------------------------
                Opcode::LoadUpvalue(i) => {
                    let uv = {
                        let frame = self.call_stack.last().unwrap();
                        frame.closure.upvalues[i as usize].clone()
                    };
                    let val = match &*reference_env!(uv) {
                        Upvalue::Open(abs) => self.stack[*abs].clone(),
                        Upvalue::Closed(v) => v.clone(),
                    };
                    self.stack.push(val);
                }
                Opcode::StoreUpvalue(i) => {
                    let new_val = self.stack.pop().unwrap();
                    let uv = {
                        let frame = self.call_stack.last().unwrap();
                        frame.closure.upvalues[i as usize].clone()
                    };
                    let abs_opt = match &*reference_env!(uv) {
                        Upvalue::Open(abs) => Some(*abs),
                        Upvalue::Closed(_) => None,
                    };
                    if let Some(abs) = abs_opt {
                        self.stack[abs] = new_val;
                    } else {
                        *mut_env!(uv) = Upvalue::Closed(new_val);
                    }
                }

                // --- global variable ------------------------------------
                Opcode::LoadGlobal(i) => {
                    let mut val = self.get_global(i);

                    // lookup: Search the environment when `global` is `Nil`
                    if matches!(val, Val::Nil) {
                        if let Some(name) = self.global_idx_to_name.get(i as usize) {
                            let name = name.clone();
                            if let Some((s, f)) = self.env.get_builtin_func(&name) {
                                val = Val::Builtin(f, s);
                                self.globals[i as usize] = val.clone();
                            } else if let Some(f) = self.env.get_builtin_ext_func(&name) {
                                val = Val::BuiltinExt(f.clone());
                                self.globals[i as usize] = val.clone();
                            }
                        }
                    }
                    self.stack.push(val);
                }
                Opcode::StoreGlobal(i) => {
                    let val = self.stack.pop().unwrap();
                    self.set_global(i, val);
                    self.stack.push(Val::Nil);
                }
                Opcode::DefineGlobal(i) => {
                    let val = self.stack.pop().unwrap();
                    self.set_global(i, val);
                    self.stack.push(Val::Nil);
                }

                // --- operate stack  -----------------------------
                Opcode::Pop => {
                    self.stack.pop();
                }
                Opcode::Dup => {
                    let v = self.stack.last().unwrap().clone();
                    self.stack.push(v);
                }

                // --- control flow -------------------------------
                Opcode::Jump(offset) => {
                    let frame = self.call_stack.last_mut().unwrap();
                    frame.ip = (frame.ip as i32 + offset) as usize;
                }
                Opcode::JumpIfFalse(offset) => {
                    let val = self.stack.pop().unwrap();
                    if val.is_false() {
                        let frame = self.call_stack.last_mut().unwrap();
                        frame.ip = (frame.ip as i32 + offset) as usize;
                    }
                }
                Opcode::JumpIfTrue(offset) => {
                    let val = self.stack.pop().unwrap();
                    if !val.is_false() {
                        let frame = self.call_stack.last_mut().unwrap();
                        frame.ip = (frame.ip as i32 + offset) as usize;
                    }
                }

                // --- create closure -------------------------------
                Opcode::MakeClosure(closure_ci) => {
                    let closure_ci = closure_ci as usize;
                    let uv_count = self.chunks[closure_ci].upvalue_count;
                    let mut upvalues = Vec::with_capacity(uv_count);

                    for _ in 0..uv_count {
                        // Continue reading about CaptureLocal and CaptureUpvalue immediately after MakeClosure
                        let cap_op = {
                            let frame = self.call_stack.last_mut().unwrap();
                            let outer_ci = frame.closure.chunk_idx;
                            let cap = self.chunks[outer_ci].code[frame.ip].clone();
                            frame.ip += 1;
                            cap
                        };
                        let uv: UpvalueRef = match cap_op {
                            Opcode::CaptureLocal(slot) => {
                                let abs = {
                                    let frame = self.call_stack.last().unwrap();
                                    frame.base + slot as usize
                                };
                                self.get_or_create_open_upvalue(abs)
                            }
                            Opcode::CaptureUpvalue(idx) => {
                                let frame = self.call_stack.last().unwrap();
                                frame.closure.upvalues[idx as usize].clone()
                            }
                            _ => panic!("expected CaptureLocal/CaptureUpvalue after MakeClosure"),
                        };
                        upvalues.push(uv);
                    }

                    let closure = new_closure_ref(VmClosure {
                        chunk_idx: closure_ci,
                        upvalues,
                    });
                    self.stack.push(Val::Closure(closure));
                }

                // It is consumed within the MakeClosure handler, so it is never reached on its own
                Opcode::CaptureLocal(_) | Opcode::CaptureUpvalue(_) => {
                    panic!("CaptureLocal/CaptureUpvalue reached outside MakeClosure");
                }

                // --- function call ------------------------------
                Opcode::Call(argc) => {
                    self.do_call(argc as usize, false)?;
                }
                Opcode::TailCall(argc) => {
                    self.do_call(argc as usize, true)?;
                }
                Opcode::Return => {
                    self.do_return();
                }

                Opcode::Apply => {
                    // (apply fn args-list) -> stack: [fn, args-list]
                    let args_val = self.stack.pop().unwrap();
                    let fn_val = self.stack.pop().unwrap();
                    let args: Vec<Val> = match args_val {
                        Val::Scheme(Expression::List(l)) => reference_obj!(l)
                            .iter()
                            .map(|e| Val::from(e.clone()))
                            .collect(),
                        Val::Nil => vec![],
                        _ => return Err(create_error!(ErrCode::E1005)),
                    };
                    let argc = args.len();
                    self.stack.push(fn_val);
                    for a in args {
                        self.stack.push(a);
                    }
                    self.do_call(argc, false)?;
                }

                // --- Arithmetic operators -----------------------
                Opcode::Add => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(self.arith_add(a, b)?);
                }
                Opcode::Sub => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(self.arith_sub(a, b)?);
                }
                Opcode::Mul => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(self.arith_mul(a, b)?);
                }
                Opcode::Div => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(self.arith_div(a, b)?);
                }
                Opcode::Neg => {
                    let a = self.stack.pop().unwrap();
                    let result = match a {
                        Val::Int(n) => Val::Int(-n),
                        Val::Float(f) => Val::Float(-f),
                        _ => return Err(create_error!(ErrCode::E1003)),
                    };
                    self.stack.push(result);
                }

                // --- compare operators -----------------------
                // stack: [left, right] -> left OP right
                Opcode::NumEq => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(Val::Bool(
                        self.num_cmp(&a, &b)? == std::cmp::Ordering::Equal,
                    ));
                }
                Opcode::NumLt => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack
                        .push(Val::Bool(self.num_cmp(&a, &b)? == std::cmp::Ordering::Less));
                }
                Opcode::NumLe => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(Val::Bool(
                        self.num_cmp(&a, &b)? != std::cmp::Ordering::Greater,
                    ));
                }
                Opcode::NumGt => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(Val::Bool(
                        self.num_cmp(&a, &b)? == std::cmp::Ordering::Greater,
                    ));
                }
                Opcode::NumGe => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack
                        .push(Val::Bool(self.num_cmp(&a, &b)? != std::cmp::Ordering::Less));
                }

                // --- list operators -----------------------
                Opcode::Car => {
                    let v = self.stack.pop().unwrap();
                    self.stack.push(self.do_car(v)?);
                }
                Opcode::Cdr => {
                    let v = self.stack.pop().unwrap();
                    self.stack.push(self.do_cdr(v)?);
                }
                Opcode::Cons => {
                    let cdr = self.stack.pop().unwrap();
                    let car = self.stack.pop().unwrap();
                    self.stack.push(self.do_cons(car, cdr));
                }
                Opcode::IsNull => {
                    let v = self.stack.pop().unwrap();
                    let is_null = matches!(v, Val::Nil)
                        || matches!(&v, Val::Scheme(Expression::List(l))
                            if reference_obj!(l).is_empty());
                    self.stack.push(Val::Bool(is_null));
                }
                Opcode::Not => {
                    let v = self.stack.pop().unwrap();
                    self.stack.push(Val::Bool(v.is_false()));
                }
            }
        }
    }

    // --- implements function call

    fn do_call(&mut self, argc: usize, tail: bool) -> Result<(), Box<Error>> {
        // stack: [..., fn, arg0, ..., argN-1]
        let fn_pos = self.stack.len() - argc - 1;
        let callee = self.stack[fn_pos].clone();

        match callee {
            Val::Closure(closure) => {
                let chunk = &self.chunks[closure.chunk_idx];
                // arithmetic operator check
                if !chunk.is_variadic && argc != chunk.arity {
                    return Err(create_error_value!(ErrCode::E1007, argc));
                }

                if tail {
                    // ---- Tail call: Reuse the current frame --------------
                    let new_args: Vec<Val> = self.stack[fn_pos + 1..].to_vec();

                    // --- Close the upvalue of the current frame ---
                    let base = self.call_stack.last().unwrap().base;
                    self.close_upvalues(base);

                    // --- Write the new argument to the frame's local scope ---
                    for (i, a) in new_args.into_iter().enumerate() {
                        let abs = base + i;
                        if abs < self.stack.len() {
                            self.stack[abs] = a;
                        } else {
                            self.stack.resize(abs + 1, Val::Nil);
                            self.stack[abs] = a;
                        }
                    }
                    // Trim the stack to the new local count
                    self.stack.truncate(base + argc);

                    // Update the frame (without increasing the call stack)
                    let frame = self.call_stack.last_mut().unwrap();
                    frame.closure = closure;
                    frame.ip = 0;
                    debug!("do_call tail call");
                } else {
                    // standard call: push new frame
                    let new_base = fn_pos + 1;
                    debug!("do_call standard call");
                    self.call_stack.push(CallFrame {
                        closure,
                        ip: 0,
                        base: new_base,
                    });
                }
            }

            Val::Builtin(f, _) => {
                // call builtin function
                let args: Vec<Expression> = self.stack[fn_pos + 1..]
                    .iter()
                    .map(|v| Expression::from(v.clone()))
                    .collect();
                let result = f(&args, &self.env)?;
                self.stack.truncate(fn_pos);
                self.stack.push(Val::from(result));
            }

            Val::BuiltinExt(f) => {
                let args: Vec<Expression> = self.stack[fn_pos + 1..]
                    .iter()
                    .map(|v| Expression::from(v.clone()))
                    .collect();
                let result = f(&args, &self.env)?;
                self.stack.truncate(fn_pos);
                self.stack.push(Val::from(result));
            }

            e => {
                return Err(create_error_value!(ErrCode::E1006, e)); // Not a function
            }
        }
        Ok(())
    }

    fn do_return(&mut self) {
        let ret_val = self.stack.pop().unwrap_or(Val::Nil);
        let frame = self.call_stack.pop().unwrap();

        // Close the frame's upvalue
        self.close_upvalues(frame.base);

        // Rewind the stack to the (base - 1)th fn slot
        let truncate_to = frame.base.saturating_sub(1);
        self.stack.truncate(truncate_to);
        self.stack.push(ret_val);
    }

    // ---- Arithmetic Help ----------------------------------------------------

    // Argument order: a = left operand, b = right operand (since the stack is LIFO, b = pop, a = pop)
    fn arith_add(&self, a: Val, b: Val) -> Result<Val, Box<Error>> {
        match (a, b) {
            (Val::Int(x), Val::Int(y)) => Ok(Val::Int(x + y)),
            (Val::Float(x), Val::Float(y)) => Ok(Val::Float(x + y)),
            (Val::Int(x), Val::Float(y)) => Ok(Val::Float(x as f64 + y)),
            (Val::Float(x), Val::Int(y)) => Ok(Val::Float(x + y as f64)),
            _ => Err(create_error!(ErrCode::E1003)),
        }
    }

    fn arith_sub(&self, a: Val, b: Val) -> Result<Val, Box<Error>> {
        match (a, b) {
            (Val::Int(x), Val::Int(y)) => Ok(Val::Int(x - y)),
            (Val::Float(x), Val::Float(y)) => Ok(Val::Float(x - y)),
            (Val::Int(x), Val::Float(y)) => Ok(Val::Float(x as f64 - y)),
            (Val::Float(x), Val::Int(y)) => Ok(Val::Float(x - y as f64)),
            _ => Err(create_error!(ErrCode::E1003)),
        }
    }

    fn arith_mul(&self, a: Val, b: Val) -> Result<Val, Box<Error>> {
        match (a, b) {
            (Val::Int(x), Val::Int(y)) => Ok(Val::Int(x * y)),
            (Val::Float(x), Val::Float(y)) => Ok(Val::Float(x * y)),
            (Val::Int(x), Val::Float(y)) => Ok(Val::Float(x as f64 * y)),
            (Val::Float(x), Val::Int(y)) => Ok(Val::Float(x * y as f64)),
            _ => Err(create_error!(ErrCode::E1003)),
        }
    }

    fn arith_div(&self, a: Val, b: Val) -> Result<Val, Box<Error>> {
        match (a, b) {
            (Val::Int(x), Val::Int(y)) => {
                if y == 0 {
                    return Err(create_error!(ErrCode::E1013));
                }
                Ok(Val::Int(x / y))
            }
            (Val::Float(x), Val::Float(y)) => Ok(Val::Float(x / y)),
            (Val::Int(x), Val::Float(y)) => Ok(Val::Float(x as f64 / y)),
            (Val::Float(x), Val::Int(y)) => Ok(Val::Float(x / y as f64)),
            _ => Err(create_error!(ErrCode::E1003)),
        }
    }

    fn num_cmp(&self, a: &Val, b: &Val) -> Result<std::cmp::Ordering, Box<Error>> {
        use std::cmp::Ordering;
        match (a, b) {
            (Val::Int(x), Val::Int(y)) => Ok(x.cmp(y)),
            (Val::Float(x), Val::Float(y)) => Ok(x.partial_cmp(y).unwrap_or(Ordering::Equal)),
            (Val::Int(x), Val::Float(y)) => {
                Ok((*x as f64).partial_cmp(y).unwrap_or(Ordering::Equal))
            }
            (Val::Float(x), Val::Int(y)) => {
                Ok(x.partial_cmp(&(*y as f64)).unwrap_or(Ordering::Equal))
            }
            _ => Err(create_error!(ErrCode::E1003)),
        }
    }

    // --- List Helpers --------------------------------------------------------
    fn do_car(&self, v: Val) -> Result<Val, Box<Error>> {
        match v {
            Val::Scheme(Expression::List(l)) => {
                let l = reference_obj!(l);
                if l.is_empty() {
                    return Err(create_error!(ErrCode::E1005));
                }
                Ok(Val::from(l[0].clone()))
            }
            Val::Scheme(Expression::Pair(p)) => Ok(Val::from(p.0.clone())),
            e => Err(create_error_value!(ErrCode::E1005, e)),
        }
    }

    fn do_cdr(&self, v: Val) -> Result<Val, Box<Error>> {
        match v {
            Val::Scheme(Expression::List(l)) => {
                let l = reference_obj!(l);
                if l.is_empty() {
                    return Err(create_error!(ErrCode::E1005));
                }
                let tail: Vec<Expression> = l[1..].to_vec();
                Ok(Val::Scheme(Environment::create_list(tail)))
            }
            Val::Scheme(Expression::Pair(p)) => Ok(Val::from(p.1.clone())),
            _ => Err(create_error!(ErrCode::E1005)),
        }
    }

    fn do_cons(&self, car: Val, cdr: Val) -> Val {
        let c = Expression::from(car);
        match Expression::from(cdr) {
            Expression::List(l) => {
                let mut new_l = vec![c];
                new_l.extend(reference_obj!(l).iter().cloned());
                Val::Scheme(Environment::create_list(new_l))
            }
            Expression::Nil() => Val::Scheme(Environment::create_list(vec![c])),
            e => Val::Scheme(Expression::Pair(Box::new((c, e)))),
        }
    }
    #[cfg(debug_assertions)]
    fn debug_print_system(&self, op: &Opcode) {
        debug!("vm::run: {}", op);
        for (i, v) in self.stack.clone().iter().enumerate() {
            debug!("stack[{}] = {}", i, v);
        }
        for (i, v) in self.call_stack.iter().enumerate() {
            debug!(
                "call stack[{}] = base:{} ip:{} closer:{}",
                i, v.base, v.ip, v.closure.chunk_idx
            );
        }
        for (i, v) in self.global_idx_to_name.iter().enumerate() {
            debug!("global[{}] = {}", i, v);
        }
        for (i, v) in self.global_idx_to_name.iter().enumerate() {
            debug!("global_idx_to_name[{}] = {}", i, v);
        }
    }
}

//========================================================================
// Parses a program string -> compiles it -> executes it in the VM and returns an Expression
// Compiler and VM state persisted between REPL executions
pub struct VmSession {
    chunks: Vec<Chunk>,
    globals: Vec<Val>,
    global_idx_to_name: Vec<String>,
    global_names: HashMap<String, u32>,
}

impl VmSession {
    fn new() -> Self {
        VmSession {
            chunks: Vec::new(),
            globals: Vec::new(),
            global_idx_to_name: Vec::new(),
            global_names: HashMap::new(),
        }
    }
}

thread_local! {
    static SESSION: RefCell<VmSession> = RefCell::new(VmSession::new());
}

// Entry point for calling expressions one at a time from the REPL.
// Preserves session state (global variables and chunks) between calls.
pub fn run_expression(exp: Expression, env: &Environment) -> ResultExpression {
    SESSION.with(|cell| {
        let mut session = cell.borrow_mut();

        // 1. Generate the compiler using the existing session state
        let mut compiler =
            Compiler::with_state(session.chunks.clone(), session.global_names.clone());

        // 2. Compile a single expression
        let chunk_idx = compiler.compile_program(&[exp])?;
        debug!("vm run_expression: {}", compiler.disassemble_all());

        // 3. Extend the session’s global table (if new names have been added)
        let needed = compiler.global_names().len();
        let mut globals = session.globals.clone();
        let mut global_idx_to_name = session.global_idx_to_name.clone();

        if needed > globals.len() {
            let old_len = globals.len();
            globals.resize(needed, Val::Nil);
            global_idx_to_name.resize(needed, String::new());

            for (name, &idx) in compiler.global_names() {
                let i = idx as usize;
                if i >= old_len {
                    global_idx_to_name[i] = name.clone();
                    if let Some((s, f)) = env.get_builtin_func(name) {
                        globals[i] = Val::Builtin(f, s);
                    } else if let Some(f) = env.get_builtin_ext_func(name) {
                        globals[i] = Val::BuiltinExt(f);
                    }
                }
            }
        }

        // 4. Generate the VM
        let mut vm = Vm {
            stack: Vec::with_capacity(256),
            call_stack: Vec::with_capacity(64),
            globals,
            global_idx_to_name,
            chunks: compiler.chunks.clone(),
            open_upvalues: Vec::new(),
            timer_stack: Vec::new(),
            env: env.clone(),
        };
        // 5. Execute
        let val = vm.run(chunk_idx)?;

        // 6. Save the updated state to the session
        session.chunks = vm.chunks;
        session.globals = vm.globals;
        session.global_idx_to_name = vm.global_idx_to_name;
        session.global_names = compiler.global_names().clone();

        Ok(Expression::from(val))
    })
}

pub fn do_vm_logic(program: &str, env: &Environment) -> ResultExpression {
    // Parsing: Collect all expressions in a tokenize -> parse loop
    let mut tokens = tokenize(program);
    let mut exprs: Vec<Expression> = Vec::new();
    let mut c: i32 = 1;

    loop {
        let expr = parse(&tokens, &mut c, env)?;
        exprs.push(expr);
        if c == tokens.len() as i32 {
            break;
        }
        for _ in 0..c as usize {
            tokens.remove(0);
        }
        c = 1;
    }

    // Compile
    let mut compiler = Compiler::new();
    let chunk_idx = compiler.compile_program(&exprs)?;

    // VM execution
    let mut vm = Vm::new(&compiler, env);
    let val = vm.run(chunk_idx)?;

    Ok(Expression::from(val))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lisp::Environment;

    fn run_vm(program: &str) -> String {
        let env = Environment::new();
        match do_vm_logic(program, &env) {
            Ok(v) => v.to_string(),
            Err(e) => e.get_code(),
        }
    }

    #[test]
    fn test_integer() {
        assert_eq!(run_vm("42"), "42");
        assert_eq!(run_vm("-7"), "-7");
    }

    #[test]
    fn test_float() {
        assert_eq!(run_vm("3.14"), "3.14");
    }

    #[test]
    fn test_bool() {
        assert_eq!(run_vm("#t"), "#t");
        assert_eq!(run_vm("#f"), "#f");
    }

    #[test]
    fn test_add() {
        assert_eq!(run_vm("(+ 1 2)"), "3");
        assert_eq!(run_vm("(+ 10 -3)"), "7");
    }

    #[test]
    fn test_sub() {
        assert_eq!(run_vm("(- 10 3)"), "7");
    }

    #[test]
    fn test_mul() {
        assert_eq!(run_vm("(* 3 4)"), "12");
    }

    #[test]
    fn test_div() {
        assert_eq!(run_vm("(/ 10 2)"), "5");
    }

    #[test]
    fn test_nested_arith() {
        assert_eq!(run_vm("(+ (* 2 3) (- 10 4))"), "12");
    }

    #[test]
    fn test_comparison() {
        assert_eq!(run_vm("(= 3 3)"), "#t");
        assert_eq!(run_vm("(= 3 4)"), "#f");
        assert_eq!(run_vm("(< 2 3)"), "#t");
        assert_eq!(run_vm("(> 5 3)"), "#t");
        assert_eq!(run_vm("(<= 3 3)"), "#t");
        assert_eq!(run_vm("(>= 4 3)"), "#t");
    }

    #[test]
    fn test_if_true() {
        assert_eq!(run_vm("(if #t 1 2)"), "1");
    }

    #[test]
    fn test_if_false() {
        assert_eq!(run_vm("(if #f 1 2)"), "2");
    }

    #[test]
    fn test_if_no_else() {
        assert_eq!(run_vm("(if #f 42)"), "nil");
    }

    #[test]
    fn test_define_global() {
        assert_eq!(run_vm("(define x 100) x"), "100");
    }

    #[test]
    fn test_lambda_call() {
        assert_eq!(run_vm("((lambda (x) (* x x)) 5)"), "25");
    }

    #[test]
    fn test_define_function() {
        assert_eq!(run_vm("(define (square x) (* x x)) (square 7)"), "49");
    }

    #[test]
    fn test_tail_recursion_loop() {
        // (let loop ((i 0)) (if (>= i 10000) i (loop (+ i 1))))
        assert_eq!(
            run_vm("(let loop ((i 0)) (if (>= i 10000) i (loop (+ i 1))))"),
            "10000"
        );
    }

    #[test]
    fn test_tail_recursion_fact() {
        // (fact 10 1) = 3628800
        assert_eq!(
            run_vm("(define (fact n acc) (if (= n 0) acc (fact (- n 1) (* n acc)))) (fact 10 1)"),
            "3628800"
        );
    }

    #[test]
    fn test_tail_recursion_large() {
        // 100,000 tail recursions without a stack overflow
        assert_eq!(
            run_vm("(define (f i) (if (= i 0) 0 (f (- i 1)))) (f 100000)"),
            "0"
        );
    }

    #[test]
    fn test_closure_capture() {
        // (define (make-adder n) (lambda (x) (+ x n)))
        assert_eq!(
            run_vm("(define (make-adder n) (lambda (x) (+ x n))) ((make-adder 5) 3)"),
            "8"
        );
    }

    #[test]
    fn test_let() {
        assert_eq!(run_vm("(let ((x 3) (y 4)) (+ x y))"), "7");
    }

    #[test]
    fn test_begin() {
        assert_eq!(run_vm("(begin 1 2 3)"), "3");
    }

    #[test]
    fn test_and() {
        assert_eq!(run_vm("(and #t #t)"), "#t");
        assert_eq!(run_vm("(and #t #f)"), "#f");
        assert_eq!(run_vm("(and)"), "#t");
    }

    #[test]
    fn test_or() {
        assert_eq!(run_vm("(or #f #t)"), "#t");
        assert_eq!(run_vm("(or #f #f)"), "#f");
        assert_eq!(run_vm("(or)"), "#f");
    }

    #[test]
    fn test_not() {
        assert_eq!(run_vm("(not #f)"), "#t");
        assert_eq!(run_vm("(not #t)"), "#f");
        assert_eq!(run_vm("(not 42)"), "#f");
    }

    #[test]
    fn test_quote_symbol() {
        assert_eq!(run_vm("(quote hello)"), "hello");
    }

    #[test]
    fn test_quote_list() {
        assert_eq!(run_vm("'(1 2 3)"), "(1 2 3)");
    }

    #[test]
    fn test_car_cdr() {
        assert_eq!(run_vm("(car '(1 2 3))"), "1");
        assert_eq!(run_vm("(cdr '(1 2 3))"), "(2 3)");
    }

    #[test]
    fn test_null_check() {
        assert_eq!(run_vm("(null? '())"), "#t");
        assert_eq!(run_vm("(null? '(1))"), "#f");
    }
}
