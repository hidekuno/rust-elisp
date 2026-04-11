/*
   Rust study program.

   Bytecode compiler: Expression AST -> Chunk (Vec<Opcode>).

   hidekuno@gmail.com
*/
#[allow(unused_imports)]
use log::{debug, error, info, warn}; // ex.) export RUST_LOG=debug
use std::collections::HashMap;

use crate::bytecode::opcode::{Chunk, Opcode};
use crate::create_error;
use crate::create_error_value;
use crate::lisp::{Environment, ErrCode, Error, Expression};
use crate::reference_obj;

//========================================================================
#[derive(Debug, Clone)]
struct UpvalueDesc {
    is_local: bool, // true = parent frame; false = parent upvalue
    index: u16,
}

#[derive(Debug)]
struct FunctionScope {
    chunk_idx: usize,
    locals: Vec<String>,
    upvalues: Vec<UpvalueDesc>,
    #[allow(dead_code)]
    name: String,
}

impl FunctionScope {
    fn new(name: impl Into<String>, params: Vec<String>, chunk_idx: usize) -> Self {
        FunctionScope {
            chunk_idx,
            locals: params,
            upvalues: Vec::new(),
            name: name.into(),
        }
    }

    // Search for local variables by name (from the end: prioritize inner `define` statements)
    fn find_local(&self, name: &str) -> Option<u16> {
        self.locals
            .iter()
            .rposition(|s| s == name)
            .map(|i| i as u16)
    }

    // Add a new local variable to return the slot number
    fn add_local(&mut self, name: String) -> u16 {
        let idx = self.locals.len() as u16;
        self.locals.push(name);
        idx
    }

    // Add upvalue (return the existing index if a duplicate exists)
    fn add_upvalue(&mut self, desc: UpvalueDesc) -> u16 {
        for (i, u) in self.upvalues.iter().enumerate() {
            if u.is_local == desc.is_local && u.index == desc.index {
                return i as u16;
            }
        }
        let idx = self.upvalues.len() as u16;
        self.upvalues.push(desc);
        idx
    }
}

// -------------------- Types of variable references ------------------------
#[derive(Debug, Clone)]
enum VarRef {
    Local(u16),
    Upvalue(u16),
    Global(u32),
}
//========================================================================
pub struct Compiler {
    pub chunks: Vec<Chunk>,
    scopes: Vec<FunctionScope>,
    global_names: HashMap<String, u32>,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            chunks: Vec::new(),
            scopes: Vec::new(),
            global_names: HashMap::new(),
        }
    }

    // Regenerate the compiler from the existing session state (for REPL continuations)
    pub fn with_state(chunks: Vec<Chunk>, global_names: HashMap<String, u32>) -> Self {
        Compiler {
            chunks,
            scopes: Vec::new(),
            global_names,
        }
    }

    // ------------ Current access to the chunk ------------------
    fn current_chunk_idx(&self) -> usize {
        self.scopes.last().unwrap().chunk_idx
    }

    fn emit(&mut self, op: Opcode) -> usize {
        let idx = self.current_chunk_idx();
        self.chunks[idx].emit(op)
    }

    fn add_constant(&mut self, val: Expression) -> u16 {
        let idx = self.current_chunk_idx();
        self.chunks[idx].add_constant(val)
    }

    fn patch_jump(&mut self, offset: usize) {
        let idx = self.current_chunk_idx();
        self.chunks[idx].patch_jump(offset);
    }

    // ---------------  Global Variable Management ----------------
    fn get_or_create_global(&mut self, name: &str) -> u32 {
        debug!("get_or_create_global: {}", name);
        if let Some(&idx) = self.global_names.get(name) {
            return idx;
        }
        let idx = self.global_names.len() as u32;
        self.global_names.insert(name.to_string(), idx);
        idx
    }

    // -------------- Variable resolution ------------------------
    fn resolve_variable(&mut self, name: &str) -> VarRef {
        let depth = self.scopes.len();

        // Currently in the local scope
        if let Some(slot) = self.scopes[depth - 1].find_local(name) {
            debug!("resolve_variable: find_local: {} {}", name, slot);
            return VarRef::Local(slot);
        }

        // Outer scope -> upvalue chain
        if let Some(uv_idx) = self.resolve_upvalue(depth - 1, name) {
            debug!("resolve_variable: upvalue: {}", name);
            return VarRef::Upvalue(uv_idx);
        }

        // global
        let idx = self.get_or_create_global(name);
        debug!("resolve_variable: global: {} {}", name, idx);
        VarRef::Global(idx)
    }

    // ------- Recursively search outer scopes to construct the upvalue chain -----
    fn resolve_upvalue(&mut self, scope_idx: usize, name: &str) -> Option<u16> {
        if scope_idx == 0 {
            return None;
        }
        let parent = scope_idx - 1;

        // Find the local variable in the parent scope (release the `borrow` first, then use `mut borrow`)
        let local_opt = self.scopes[parent].find_local(name);
        if let Some(local_idx) = local_opt {
            return Some(self.scopes[scope_idx].add_upvalue(UpvalueDesc {
                is_local: true,
                index: local_idx,
            }));
        }

        // Search the outer layers recursively
        let outer_opt = self.resolve_upvalue(parent, name);
        if let Some(outer_uv) = outer_opt {
            return Some(self.scopes[scope_idx].add_upvalue(UpvalueDesc {
                is_local: false,
                index: outer_uv,
            }));
        }

        None
    }

    // --- Issuing Load/Store Instructions --------------------------
    fn emit_load(&mut self, vref: VarRef) {
        debug!("emit_load: {:?}", vref);
        match vref {
            VarRef::Local(i) => {
                self.emit(Opcode::LoadLocal(i));
            }
            VarRef::Upvalue(i) => {
                self.emit(Opcode::LoadUpvalue(i));
            }
            VarRef::Global(i) => {
                self.emit(Opcode::LoadGlobal(i));
            }
        }
    }

    fn emit_store(&mut self, vref: VarRef) {
        debug!("emit_store {:?}", vref);
        match vref {
            VarRef::Local(i) => {
                self.emit(Opcode::StoreLocal(i));
            }
            VarRef::Upvalue(i) => {
                self.emit(Opcode::StoreUpvalue(i));
            }
            VarRef::Global(i) => {
                self.emit(Opcode::StoreGlobal(i));
            }
        }
    }

    // ----------- Top-level compilation -------------------------------
    // Compile multiple expressions and return the index of the top-level chunk
    pub fn compile_program(&mut self, exp: &[Expression]) -> Result<usize, Box<Error>> {
        let chunk_idx = self.chunks.len();
        self.chunks.push(Chunk::new("<toplevel>", 0, false));
        self.scopes
            .push(FunctionScope::new("<toplevel>", vec![], chunk_idx));

        let n = exp.len();
        for (i, e) in exp.iter().enumerate() {
            self.compile_expression(e, false)?;
            if i < n - 1 {
                self.emit(Opcode::Pop);
            }
        }
        self.emit(Opcode::Halt);

        self.scopes.pop();
        Ok(chunk_idx)
    }

    // ---- Compiling expression -----------------------------
    // recursive call like lisp.rs:eval
    fn compile_expression(&mut self, exp: &Expression, tail: bool) -> Result<(), Box<Error>> {
        #[cfg(debug_assertions)]
        {
            self.debug_print_system(exp);
        }

        match exp {
            Expression::Integer(n) => {
                self.emit(Opcode::PushInt(*n));
            }
            Expression::Float(f) => {
                self.emit(Opcode::PushFloat(*f));
            }
            Expression::Boolean(b) => {
                self.emit(Opcode::PushBool(*b));
            }
            Expression::Char(c) => {
                self.emit(Opcode::PushChar(*c));
            }
            Expression::Nil() => {
                self.emit(Opcode::PushNil);
            }

            Expression::String(s) => {
                let e = Expression::String(s.clone());
                let idx = self.add_constant(e);
                self.emit(Opcode::PushConst(idx));
            }

            Expression::Symbol(s) => {
                let name = s.as_str().to_string();
                debug!("Expression::Symbol: {}", name);
                let vref = self.resolve_variable(&name);
                self.emit_load(vref);
            }

            Expression::List(l) => {
                debug!("Expression::List: {}", tail);
                // Clone the project, release the borrow, and then compile
                let list = reference_obj!(l).clone();
                self.compile_list(&list, tail)?;
            }

            // Others (e.g., quoted data) -> Store as constants
            _ => {
                debug!("compile_expression other: {}", exp);
                let e = exp.clone();
                let idx = self.add_constant(e);
                self.emit(Opcode::PushConst(idx));
            }
        }
        Ok(())
    }

    // ----- Compilation in list format --------------------------
    fn compile_list(&mut self, list: &[Expression], tail: bool) -> Result<(), Box<Error>> {
        if list.is_empty() {
            self.emit(Opcode::PushNil);
            return Ok(());
        }

        // Extract the symbol name from the first element
        let head_name: Option<&str> = match &list[0] {
            Expression::Symbol(s) => Some(s.as_str()),
            _ => None,
        };

        if let Some(name) = head_name {
            let args = &list[1..];
            match name {
                "quote" => return self.compile_quote(args),
                "if" => return self.compile_if(args, tail),
                "cond" => return self.compile_cond(args, tail),
                "begin" => return self.compile_begin(args, tail),
                "define" => return self.compile_define(args),
                "set!" => return self.compile_set(args),
                "lambda" => return self.compile_lambda(args, "<lambda>"),
                "let" => return self.compile_let(args, tail),
                "let*" => return self.compile_let_star(args, tail),
                "and" => return self.compile_and(args, tail),
                "or" => return self.compile_or(args, tail),
                "time" => return self.compile_time(args),
                _ => {}
            }

            // Built-in fast path (limited to a fixed number of arguments)
            if let Some(fast_op) = Self::fast_op(name, args.len()) {
                for a in args {
                    self.compile_expression(a, false)?;
                }
                self.emit(fast_op);
                return Ok(());
            }
        }

        // Standard function call
        self.compile_call(list, tail)
    }

    // ----- Selecting a High-Speed Pass Command --------------------------
    fn fast_op(name: &str, argc: usize) -> Option<Opcode> {
        match (name, argc) {
            ("+", 2) => Some(Opcode::Add),
            ("-", 2) => Some(Opcode::Sub),
            ("*", 2) => Some(Opcode::Mul),
            ("/", 2) => Some(Opcode::Div),
            ("-", 1) => Some(Opcode::Neg),
            ("=", 2) => Some(Opcode::NumEq),
            ("<", 2) => Some(Opcode::NumLt),
            ("<=", 2) => Some(Opcode::NumLe),
            (">", 2) => Some(Opcode::NumGt),
            (">=", 2) => Some(Opcode::NumGe),
            ("car", 1) => Some(Opcode::Car),
            ("cdr", 1) => Some(Opcode::Cdr),
            ("cons", 2) => Some(Opcode::Cons),
            ("null?", 1) => Some(Opcode::IsNull),
            ("not", 1) => Some(Opcode::Not),
            _ => None,
        }
    }

    // ----- Special Format --------------------------------------------------

    // (quote datum)
    fn compile_quote(&mut self, exp: &[Expression]) -> Result<(), Box<Error>> {
        if exp.len() != 1 {
            return Err(create_error_value!(ErrCode::E2001, exp.len()));
        }
        let idx = self.add_constant(exp[0].clone());
        self.emit(Opcode::PushConst(idx));
        Ok(())
    }

    // (if cond then [else])
    fn compile_if(&mut self, exp: &[Expression], tail: bool) -> Result<(), Box<Error>> {
        if exp.len() < 2 || exp.len() > 3 {
            return Err(create_error_value!(ErrCode::E2001, exp.len()));
        }

        self.compile_expression(&exp[0], false)?;
        let jump_false = self.emit(Opcode::JumpIfFalse(0));

        self.compile_expression(&exp[1], tail)?;
        let jump_end = self.emit(Opcode::Jump(0));

        self.patch_jump(jump_false);
        if exp.len() == 3 {
            self.compile_expression(&exp[2], tail)?;
        } else {
            self.emit(Opcode::PushNil);
        }
        self.patch_jump(jump_end);
        Ok(())
    }

    // (cond (test expr...)... [(else expr...)])
    fn compile_cond(&mut self, exp: &[Expression], tail: bool) -> Result<(), Box<Error>> {
        let mut end_jumps: Vec<usize> = Vec::new();

        for clause in exp {
            let l = match clause {
                Expression::List(l) => reference_obj!(l).clone(),
                e => return Err(create_error_value!(ErrCode::E2002, e)),
            };
            if l.is_empty() {
                return Err(create_error!(ErrCode::E2003));
            }

            // (else
            if matches!(&l[0], Expression::Symbol(s) if s.as_str() == "else") {
                self.compile_begin(&l[1..], tail)?;
                for j in &end_jumps {
                    self.patch_jump(*j);
                }
                return Ok(());
            }

            // (test expr...)
            self.compile_expression(&l[0], false)?;
            let jf = self.emit(Opcode::JumpIfFalse(0));

            if l.len() > 1 {
                self.compile_begin(&l[1..], tail)?;
            } else {
                // Use the value of `test` as-is (Scheme specification: no body)
                // Since `JumpIfFalse` causes a pop, re-evaluation is difficult -> Nil in Phase 1
                self.emit(Opcode::PushNil);
            }

            let je = self.emit(Opcode::Jump(0));
            self.patch_jump(jf);
            end_jumps.push(je);
        }

        // else None -> nil
        self.emit(Opcode::PushNil);
        for j in &end_jumps {
            self.patch_jump(*j);
        }
        Ok(())
    }

    // (begin exp...)
    fn compile_begin(&mut self, exp: &[Expression], tail: bool) -> Result<(), Box<Error>> {
        if exp.is_empty() {
            self.emit(Opcode::PushNil);
            return Ok(());
        }
        let n = exp.len();
        for (i, e) in exp.iter().enumerate() {
            self.compile_expression(e, tail && i == n - 1)?;
            if i < n - 1 {
                self.emit(Opcode::Pop);
            }
        }
        Ok(())
    }

    // (define name expr) / (define (name params...) body...)
    fn compile_define(&mut self, exp: &[Expression]) -> Result<(), Box<Error>> {
        if exp.is_empty() {
            return Err(create_error_value!(ErrCode::E2001, exp.len()));
        }

        match &exp[0].clone() {
            Expression::Symbol(s) => {
                debug!("compile_define symbol:{}", s);
                // (define name expr)
                let name = s.as_str().to_string();
                if exp.len() >= 2 {
                    self.compile_expression(&exp[1], false)?;
                } else {
                    self.emit(Opcode::PushNil);
                }
                self.emit_define(&name);
            }
            Expression::List(l) => {
                // (define (name params...) body...)
                let l = reference_obj!(l).clone();
                if l.is_empty() {
                    return Err(create_error!(ErrCode::E2004));
                }
                debug!("compile_define list:{}", &l[0]);
                let fname = match &l[0] {
                    Expression::Symbol(s) => s.as_str().to_string(),
                    e => return Err(create_error_value!(ErrCode::E2005, e)),
                };
                let params: Result<Vec<String>, _> = l[1..]
                    .iter()
                    .map(|e| match e {
                        Expression::Symbol(s) => Ok(s.as_str().to_string()),
                        e => Err(create_error_value!(ErrCode::E2005, e)),
                    })
                    .collect();

                let params = params?;
                let body = &exp[1..];
                if self.scopes.len() > 1 {
                    // letrec-style: add-local store local before compiling the body
                    // so that recursive references to fname inside the body resolve correctly
                    let slot = self.scopes.last_mut().unwrap().add_local(fname.clone());
                    self.emit(Opcode::PushNil);
                    self.emit(Opcode::StoreLocal(slot));
                    self.compile_lambda_body(params, false, body, &fname)?;
                    self.emit(Opcode::StoreLocal(slot));
                    let sym = Environment::create_symbol(fname.clone());
                    let cidx = self.add_constant(sym);
                    self.emit(Opcode::PushConst(cidx));
                } else {
                    // top level!!
                    self.compile_lambda_body(params, false, body, &fname)?;
                    self.emit_define(&fname);
                }
            }
            e => return Err(create_error_value!(ErrCode::E2006, e)),
        }
        Ok(())
    }

    // Issue a variable binding statement following the `define` statement
    fn emit_define(&mut self, name: &str) {
        if self.scopes.len() == 1 {
            // top level !!
            debug!("emit_define: top_level: {}", name);
            let idx = self.get_or_create_global(name);
            self.emit(Opcode::DefineGlobal(idx));
        } else {
            // The value is already on the stack
            let slot = self.scopes.last_mut().unwrap().add_local(name.to_string());
            self.emit(Opcode::StoreLocal(slot));

            // The return value of `define` is a variable name symbol
            let sym = Environment::create_symbol(name.to_string());
            debug!("emit_define: scopes: sym = {}", sym);
            let cidx = self.add_constant(sym);
            debug!("emit_define: scopes: cidx = {}", cidx);
            self.emit(Opcode::PushConst(cidx));
        }
    }

    // (set! name expr)
    fn compile_set(&mut self, exp: &[Expression]) -> Result<(), Box<Error>> {
        if exp.len() != 2 {
            return Err(create_error_value!(ErrCode::E2001, exp.len()));
        }
        let name = match &exp[0] {
            Expression::Symbol(s) => s.as_str().to_string(),
            e => return Err(create_error_value!(ErrCode::E2005, e)),
        };

        self.compile_expression(&exp[1], false)?;
        let vref = self.resolve_variable(&name);
        self.emit_store(vref);
        self.emit(Opcode::PushNil);
        Ok(())
    }

    // (lambda (params...) body...)
    fn compile_lambda(&mut self, exp: &[Expression], name: &str) -> Result<(), Box<Error>> {
        if exp.len() < 2 {
            return Err(create_error_value!(ErrCode::E2001, exp.len()));
        }
        let (params, is_variadic) = self.parse_params(&exp[0])?;
        self.compile_lambda_body(params, is_variadic, &exp[1..], name)
    }

    // Parses the parameter list and returns (params, is_variadic)
    fn parse_params(&self, exp: &Expression) -> Result<(Vec<String>, bool), Box<Error>> {
        match exp {
            Expression::List(l) => {
                let params: Result<Vec<String>, _> = reference_obj!(l)
                    .iter()
                    .map(|e| match e {
                        Expression::Symbol(s) => Ok(s.as_str().to_string()),
                        e => Err(create_error_value!(ErrCode::E2005, e)),
                    })
                    .collect();
                Ok((params?, false))
            }
            Expression::Symbol(s) => {
                // (lambda args body) - Takes all arguments as a list
                Ok((vec![s.as_str().to_string()], true))
            }
            Expression::Nil() => Ok((vec![], false)),
            _ => Err(create_error!(ErrCode::E2007)),
        }
    }

    // Compiling the lambda body: Create a new Chunk + Scope and issue MakeClosure
    fn compile_lambda_body(
        &mut self,
        params: Vec<String>,
        is_variadic: bool,
        body: &[Expression],
        name: &str,
    ) -> Result<(), Box<Error>> {
        let arity = params.len();
        let chunk_idx = self.chunks.len();
        self.chunks.push(Chunk::new(name, arity, is_variadic));
        self.scopes
            .push(FunctionScope::new(name, params, chunk_idx));

        // complile body
        if body.is_empty() {
            self.emit(Opcode::PushNil);
        } else {
            let n = body.len();
            for (i, e) in body.iter().enumerate() {
                debug!("compile_lambda_body: tail={}", i == n - 1);
                self.compile_expression(e, i == n - 1)?;
                if i < n - 1 {
                    self.emit(Opcode::Pop);
                }
            }
        }
        self.emit(Opcode::Return);

        // Write the upvalue information back to the chunk
        let scope = self.scopes.pop().unwrap();
        let upvalues = scope.upvalues.clone();
        self.chunks[chunk_idx].upvalue_count = upvalues.len();

        // Issue a MakeClosure command followed by a capture statement in the outer chunk
        self.emit(Opcode::MakeClosure(chunk_idx as u16));
        for uv in &upvalues {
            if uv.is_local {
                self.emit(Opcode::CaptureLocal(uv.index));
            } else {
                self.emit(Opcode::CaptureUpvalue(uv.index));
            }
        }
        Ok(())
    }

    // (let ((var init)...) body...)  /  (let name ((var init)...) body...)
    fn compile_let(&mut self, exp: &[Expression], tail: bool) -> Result<(), Box<Error>> {
        if exp.is_empty() {
            return Err(create_error_value!(ErrCode::E2001, exp.len()));
        }

        // Named let: (let name ((var init)...) body...)
        if let Expression::Symbol(loop_name) = &exp[0] {
            let loop_name = loop_name.as_str().to_string();
            return self.compile_named_let(&loop_name, &exp[1..], tail);
        }

        let (params, inits) = self.extract_bindings(&exp[0])?;
        let body = &exp[1..];

        // ((lambda (params...) body...) inits...)
        self.compile_lambda_body(params, false, body, "<let>")?;
        for init in &inits {
            self.compile_expression(init, false)?;
        }
        self.emit_call(inits.len() as u16, tail);
        Ok(())
    }

    // Named let: (let loop ((i 0)) body...)
    // Defines a lambda globally and calls it immediately
    // TODO: Improve this to convert to `letrec` to avoid global pollution
    fn compile_named_let(
        &mut self,
        loop_name: &str,
        exp: &[Expression],
        tail: bool,
    ) -> Result<(), Box<Error>> {
        if exp.is_empty() {
            return Err(create_error_value!(ErrCode::E2001, exp.len()));
        }
        let (params, inits) = self.extract_bindings(&exp[0])?;
        let body = &exp[1..];

        debug!("named_let scope_len: = {}", self.scopes.len());

        if self.scopes.len() > 1 {
            let fname = loop_name.to_string();
            let slot = self.scopes.last_mut().unwrap().add_local(fname.clone());
            self.emit(Opcode::PushNil);
            self.emit(Opcode::StoreLocal(slot));
            self.compile_lambda_body(params, false, body, &fname)?;
            self.emit(Opcode::StoreLocal(slot));
        } else {
            // Compile the loop function as a lambda
            self.compile_lambda_body(params, false, body, loop_name)?;

            // Declare it as a global variable (so it can be self-referenced within the loop itself)
            // stack : [closure]
            self.emit(Opcode::Dup);
            let g_idx = self.get_or_create_global(loop_name);
            self.emit(Opcode::DefineGlobal(g_idx)); // Dup DefineGlobal (pop + push nil)
            self.emit(Opcode::Pop); // drop nil -> stack : [closure]
        }
        // Compile the init values
        for init in &inits {
            self.compile_expression(init, false)?;
        }
        // stack : [closure, init0, init1, ...]
        self.emit_call(inits.len() as u16, tail);
        Ok(())
    }

    // (let* ((x 1)(y (+ x 1))) body...) Expand as a nested lambda
    fn compile_let_star(&mut self, exp: &[Expression], tail: bool) -> Result<(), Box<Error>> {
        if exp.is_empty() {
            return Err(create_error_value!(ErrCode::E2001, exp.len()));
        }
        let bindings = match &exp[0] {
            Expression::List(l) => reference_obj!(l).clone(),
            e => return Err(create_error_value!(ErrCode::E2002, e)),
        };
        let body = &exp[1..];

        if bindings.is_empty() {
            return self.compile_begin(body, tail);
        }

        // Remove the first bind and nest the rest inside the inner `let*`
        let first = bindings[0].clone();
        let rest: Vec<Expression> = bindings[1..].to_vec();

        let (var, init) = self.extract_single_binding(&first)?;

        // inner body: (let* (rest...) body...)  or  body itself
        let inner_body: Vec<Expression> = if rest.is_empty() {
            body.to_vec()
        } else {
            let inner = vec![
                Environment::create_symbol("let*".to_string()),
                Environment::create_list(rest),
            ]
            .into_iter()
            .chain(body.iter().cloned())
            .collect();
            vec![Environment::create_list(inner)]
        };

        self.compile_lambda_body(vec![var], false, &inner_body, "<let*>")?;
        self.compile_expression(&init, false)?;
        self.emit_call(1, tail);
        Ok(())
    }

    // ---------- and / or ---------------------------------------

    // (and e1 e2 ...)
    fn compile_and(&mut self, exp: &[Expression], tail: bool) -> Result<(), Box<Error>> {
        if exp.is_empty() {
            self.emit(Opcode::PushBool(true));
            return Ok(());
        }
        if exp.len() == 1 {
            return self.compile_expression(&exp[0], tail);
        }

        // If e1 is false, it's a short circuit -> return false
        // If e1 is true, proceed to the next step
        let mut end_jumps: Vec<usize> = Vec::new();
        let n = exp.len();
        for (i, a) in exp.iter().enumerate() {
            let is_last = i == n - 1;
            self.compile_expression(a, tail && is_last)?;
            if !is_last {
                self.emit(Opcode::Dup);
                let j = self.emit(Opcode::JumpIfFalse(0));
                self.emit(Opcode::Pop);
                end_jumps.push(j);
            }
        }
        for j in &end_jumps {
            self.patch_jump(*j);
        }
        Ok(())
    }

    // (or e1 e2 ...)
    fn compile_or(&mut self, exp: &[Expression], tail: bool) -> Result<(), Box<Error>> {
        if exp.is_empty() {
            self.emit(Opcode::PushBool(false));
            return Ok(());
        }
        if exp.len() == 1 {
            return self.compile_expression(&exp[0], tail);
        }
        let mut end_jumps: Vec<usize> = Vec::new();
        let n = exp.len();
        for (i, a) in exp.iter().enumerate() {
            let is_last = i == n - 1;
            self.compile_expression(a, tail && is_last)?;
            if !is_last {
                self.emit(Opcode::Dup);
                let j = self.emit(Opcode::JumpIfTrue(0));
                self.emit(Opcode::Pop);
                end_jumps.push(j);
            }
        }
        for j in &end_jumps {
            self.patch_jump(*j);
        }
        Ok(())
    }

    // (time expr) — Measures and displays the execution time of an expression
    fn compile_time(&mut self, args: &[Expression]) -> Result<(), Box<Error>> {
        if args.len() != 1 {
            return Err(create_error_value!(ErrCode::E2001, args.len()));
        }
        self.emit(Opcode::StartTimer);
        self.compile_expression(&args[0], false)?;
        self.emit(Opcode::StopTimer);
        Ok(())
    }

    // ------------ Call function --------------------------------

    // Standard function call : stack [fn, arg0, arg1, ...]
    fn compile_call(&mut self, list: &[Expression], tail: bool) -> Result<(), Box<Error>> {
        let argc = (list.len() - 1) as u16;
        self.compile_expression(&list[0], false)?;
        for a in &list[1..] {
            self.compile_expression(a, false)?;
        }
        self.emit_call(argc, tail);
        Ok(())
    }

    fn emit_call(&mut self, argc: u16, tail: bool) {
        if tail {
            self.emit(Opcode::TailCall(argc));
        } else {
            self.emit(Opcode::Call(argc));
        }
    }

    // --- Binding Analysis Helper -------------------------------

    // let ((var init)...) -> (Vec<String>, Vec<Expression>)
    fn extract_bindings(
        &self,
        expr: &Expression,
    ) -> Result<(Vec<String>, Vec<Expression>), Box<Error>> {
        let bindings = match expr {
            Expression::List(l) => reference_obj!(l).clone(),
            e => return Err(create_error_value!(ErrCode::E2002, e)),
        };
        let mut params = Vec::new();
        let mut inits = Vec::new();
        for b in &bindings {
            let (var, init) = self.extract_single_binding(b)?;
            params.push(var);
            inits.push(init);
        }
        Ok((params, inits))
    }

    // (var init)  -> (String, Expression)
    fn extract_single_binding(&self, exp: &Expression) -> Result<(String, Expression), Box<Error>> {
        let pair = match exp {
            Expression::List(l) => reference_obj!(l).clone(),
            e => return Err(create_error_value!(ErrCode::E2002, e)),
        };
        if pair.len() < 2 {
            return Err(create_error_value!(ErrCode::E2001, pair.len()));
        }
        let var = match &pair[0] {
            Expression::Symbol(s) => s.as_str().to_string(),
            e => return Err(create_error_value!(ErrCode::E2005, e)),
        };
        Ok((var, pair[1].clone()))
    }
    pub fn global_names(&self) -> &HashMap<String, u32> {
        &self.global_names
    }
    // --------- debug ------------------------------------------
    pub fn disassemble_all(&self) -> String {
        let mut out = String::new();
        for chunk in &self.chunks {
            out.push_str(&chunk.disassemble());
            out.push('\n');
        }
        out
    }
    #[cfg(debug_assertions)]
    fn debug_print_system(&self, exp: &Expression) {
        debug!("compile_expression: {}", exp);

        for e in &self.scopes {
            debug!("scopes = {:?}", e);
        }
        for (k, v) in &self.global_names {
            debug!("global_names = {}: {}", k, v);
        }
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compile_ok(exprs: Vec<Expression>) -> Compiler {
        let mut c = Compiler::new();
        c.compile_program(&exprs)
            .unwrap_or_else(|e| panic!("compile failed: {}", e.get_msg()));
        c
    }

    #[test]
    fn test_compile_integer() {
        let c = compile_ok(vec![Expression::Integer(42)]);
        assert_eq!(c.chunks.len(), 1);
        assert!(matches!(c.chunks[0].code[0], Opcode::PushInt(42)));
        assert!(matches!(c.chunks[0].code[1], Opcode::Halt));
    }

    #[test]
    fn test_compile_bool() {
        let c = compile_ok(vec![Expression::Boolean(true)]);
        assert!(matches!(c.chunks[0].code[0], Opcode::PushBool(true)));
    }

    #[test]
    fn test_compile_float() {
        let c = compile_ok(vec![Expression::Float(std::f64::consts::FRAC_1_PI)]);
        assert!(matches!(c.chunks[0].code[0], Opcode::PushFloat(_)));
    }

    #[test]
    fn test_compile_nil() {
        let c = compile_ok(vec![Expression::Nil()]);
        assert!(matches!(c.chunks[0].code[0], Opcode::PushNil));
    }

    #[test]
    fn test_disassemble() {
        let c = compile_ok(vec![Expression::Integer(1), Expression::Integer(2)]);
        let dis = c.disassemble_all();
        assert!(dis.contains("PUSH_INT"));
        assert!(dis.contains("HALT"));
    }

    #[test]
    fn test_fast_op() {
        assert!(matches!(Compiler::fast_op("+", 2), Some(Opcode::Add)));
        assert!(matches!(Compiler::fast_op("-", 2), Some(Opcode::Sub)));
        assert!(matches!(Compiler::fast_op("*", 2), Some(Opcode::Mul)));
        assert!(matches!(Compiler::fast_op("/", 2), Some(Opcode::Div)));
        assert!(matches!(Compiler::fast_op("-", 1), Some(Opcode::Neg)));
        assert!(matches!(Compiler::fast_op("=", 2), Some(Opcode::NumEq)));
        assert!(matches!(Compiler::fast_op("<", 2), Some(Opcode::NumLt)));
        assert!(matches!(Compiler::fast_op("car", 1), Some(Opcode::Car)));
        assert!(matches!(Compiler::fast_op("cdr", 1), Some(Opcode::Cdr)));
        assert!(matches!(Compiler::fast_op("cons", 2), Some(Opcode::Cons)));
        assert!(matches!(
            Compiler::fast_op("null?", 1),
            Some(Opcode::IsNull)
        ));
        assert!(matches!(Compiler::fast_op("not", 1), Some(Opcode::Not)));
        assert!(Compiler::fast_op("+", 3).is_none());
        assert!(Compiler::fast_op("foo", 2).is_none());
    }

    #[test]
    fn test_compile_lambda_creates_chunk() {
        use crate::lisp::Environment;
        // (lambda (x) x)
        let params = Environment::create_list(vec![Environment::create_symbol("x".to_string())]);
        let body = Environment::create_symbol("x".to_string());
        let lambda = Environment::create_list(vec![
            Environment::create_symbol("lambda".to_string()),
            params,
            body,
        ]);
        let c = compile_ok(vec![lambda]);
        // toplevel chunk + lambda chunk = 2 chunks
        assert_eq!(c.chunks.len(), 2);
        assert_eq!(c.chunks[1].arity, 1);
        assert_eq!(c.chunks[1].name, "<lambda>");
    }

    #[test]
    fn test_global_define() {
        use crate::lisp::Environment;
        // (define x 10)
        let define = Environment::create_list(vec![
            Environment::create_symbol("define".to_string()),
            Environment::create_symbol("x".to_string()),
            Expression::Integer(10),
        ]);
        let c = compile_ok(vec![define]);
        // PUSH_INT 10 -> DEFINE_GLOBAL 0 -> HALT
        assert!(matches!(c.chunks[0].code[0], Opcode::PushInt(10)));
        assert!(matches!(c.chunks[0].code[1], Opcode::DefineGlobal(0)));
        assert!(matches!(c.chunks[0].code[2], Opcode::Halt));
    }
}
#[cfg(test)]
mod normal_tests {
    use crate::bytecode::vm::do_vm_logic;
    use crate::lisp::Environment;
    fn make_env() -> Environment {
        let _g = crate::lisp::COMPILE_LOCK.lock().unwrap();
        crate::lisp::set_compile_mode(true);
        let env = Environment::new();
        crate::lisp::set_compile_mode(false);
        env
    }

    fn do_lisp(program: &str) -> String {
        let env = make_env();
        match do_vm_logic(program, &env) {
            Ok(v) => v.to_string(),
            Err(e) => e.get_code(),
        }
    }

    #[test]
    fn compile_quote() {
        assert_eq!(do_lisp("(quote a)"), "a");
        assert_eq!(do_lisp("(quote (1 2 3))"), "(1 2 3)");
        assert_eq!(do_lisp("'hello"), "hello");
    }

    #[test]
    fn compile_if() {
        assert_eq!(do_lisp("(if #t 1 2)"), "1");
        assert_eq!(do_lisp("(if #f 1 2)"), "2");
        assert_eq!(do_lisp("(if #t 1)"), "1");
        assert_eq!(do_lisp("(if (= 1 1) 10 20)"), "10");
        assert_eq!(do_lisp("(if (= 1 2) 10 20)"), "20");
    }

    #[test]
    fn compile_cond() {
        assert_eq!(do_lisp("(cond (#t 1))"), "1");
        assert_eq!(do_lisp("(cond (#f 1) (#t 2))"), "2");
        assert_eq!(do_lisp("(cond (else 3))"), "3");
        assert_eq!(do_lisp("(cond ((= 1 1) 10) (else 20))"), "10");
        assert_eq!(do_lisp("(cond ((= 1 2) 10) (else 20))"), "20");
    }

    #[test]
    fn compile_define() {
        assert_eq!(do_lisp("(define x 10) x"), "10");
        assert_eq!(do_lisp("(define (f x) x) (f 5)"), "5");
        assert_eq!(do_lisp("(define (add a b) (+ a b)) (add 3 4)"), "7");
    }

    #[test]
    fn compile_set() {
        assert_eq!(do_lisp("(define x 1) (set! x 2) x"), "2");
        assert_eq!(do_lisp("(define n 10) (set! n (+ n 5)) n"), "15");
    }

    #[test]
    fn compile_lambda() {
        assert_eq!(do_lisp("((lambda (x) x) 5)"), "5");
        assert_eq!(do_lisp("((lambda (a b) (+ a b)) 3 4)"), "7");
        assert_eq!(do_lisp("(define f (lambda (x) (* x x))) (f 6)"), "36");
    }

    #[test]
    fn compile_let() {
        assert_eq!(do_lisp("(let ((x 1)) x)"), "1");
        assert_eq!(do_lisp("(let ((x 1) (y 2)) (+ x y))"), "3");
        assert_eq!(do_lisp("(let ((x 10)) (let ((y 20)) (+ x y)))"), "30");
    }

    #[test]
    fn compile_named_let() {
        assert_eq!(
            do_lisp("(let loop ((n 5) (acc 1)) (if (= n 0) acc (loop (- n 1) (* acc n))))"),
            "120"
        );
        assert_eq!(
            do_lisp("(let loop ((i 0) (s 0)) (if (= i 5) s (loop (+ i 1) (+ s i))))"),
            "10"
        );
    }

    #[test]
    fn compile_let_star() {
        assert_eq!(do_lisp("(let* ((x 1)) x)"), "1");
        assert_eq!(do_lisp("(let* ((x 1) (y (+ x 1))) y)"), "2");
        assert_eq!(do_lisp("(let* ((a 3) (b (* a 2)) (c (+ b 1))) c)"), "7");
    }
}
#[cfg(test)]
mod error_tests {
    use crate::bytecode::vm::do_vm_logic;
    use crate::lisp::Environment;
    fn make_env() -> Environment {
        let _g = crate::lisp::COMPILE_LOCK.lock().unwrap();
        crate::lisp::set_compile_mode(true);
        let env = Environment::new();
        crate::lisp::set_compile_mode(false);
        env
    }

    fn do_lisp(program: &str) -> String {
        let env = make_env();
        match do_vm_logic(program, &env) {
            Ok(v) => v.to_string(),
            Err(e) => e.get_code(),
        }
    }

    #[test]
    fn compile_quote() {
        assert_eq!(do_lisp("(quote)"), "E2001");
        assert_eq!(do_lisp("(quote 1 2)"), "E2001");
    }

    #[test]
    fn compile_if() {
        assert_eq!(do_lisp("(if)"), "E2001");
        assert_eq!(do_lisp("(if #t)"), "E2001");
        assert_eq!(do_lisp("(if #t 1 2 3)"), "E2001");
    }

    #[test]
    fn compile_cond() {
        assert_eq!(do_lisp("(cond 10)"), "E2002");
        assert_eq!(do_lisp("(cond ())"), "E2003");
    }

    #[test]
    fn compile_define() {
        assert_eq!(do_lisp("(define)"), "E2001");
        assert_eq!(do_lisp("(define () 1)"), "E2004");
        assert_eq!(do_lisp("(define (10 x) x)"), "E2005");
        assert_eq!(do_lisp("(define (f 10) 10)"), "E2005");
        assert_eq!(do_lisp("(define 10 1)"), "E2006");
    }

    #[test]
    fn compile_set() {
        assert_eq!(do_lisp("(set!)"), "E2001");
        assert_eq!(do_lisp("(set! x)"), "E2001");
        assert_eq!(do_lisp("(set! x 1 2)"), "E2001");
        assert_eq!(do_lisp("(set! 10 1)"), "E2005");
    }

    #[test]
    fn compile_lambda() {
        assert_eq!(do_lisp("(lambda)"), "E2001");
        assert_eq!(do_lisp("(lambda ())"), "E2001");
        assert_eq!(do_lisp("(lambda 10 10)"), "E2007");
        assert_eq!(do_lisp("(lambda (10) 10)"), "E2005");
    }

    #[test]
    fn compile_let() {
        assert_eq!(do_lisp("(let)"), "E2001");
        assert_eq!(do_lisp("(let 10 10)"), "E2002");
        assert_eq!(do_lisp("(let ((x)) 10)"), "E2001");
        assert_eq!(do_lisp("(let ((10 1)) 10)"), "E2005");
        assert_eq!(do_lisp("(let (10) 10)"), "E2002");
    }

    #[test]
    fn compile_named_let() {
        assert_eq!(do_lisp("(let loop)"), "E2001");
        assert_eq!(do_lisp("(let loop 10 10)"), "E2002");
    }

    #[test]
    fn compile_let_star() {
        assert_eq!(do_lisp("(let*)"), "E2001");
        assert_eq!(do_lisp("(let* 10 10)"), "E2002");
        assert_eq!(do_lisp("(let* ((x)) 10)"), "E2001");
        assert_eq!(do_lisp("(let* ((10 1)) 10)"), "E2005");
        assert_eq!(do_lisp("(let* (10) 10)"), "E2002");
    }

    #[test]
    fn compile_time() {
        assert_eq!(do_lisp("(time)"), "E2001");
        assert_eq!(do_lisp("(time 1 2)"), "E2001");
    }
}
