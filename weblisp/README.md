Lisp interpreter on easy web Server
=================

## Overview
- Implemented a Lisp for Rust lessons. (It's Scheme base)
- This program was maked based on this URL.  
  https://doc.rust-lang.org/stable/book/ch20-00-final-project-a-web-server.html

### Requirement
- rust(rustc, cargo..) installed.

## Test
```
cd ${HOME}
git clone https://github.com/hidekuno/rust-elisp
cd rust-elisp/weblisp
cargo test --lib -- --test-threads=1
```

## Run
```
cd ${HOME}
cd rust-elisp/weblisp/samples/examples
cargo run --bin weblisp
```

## Run wasm
```
cd ${HOME}
cd rust-elisp/weblisp
cargo build --release --bin weblisp
cd ../wasmlisp
wasm-pack build --target web --out-dir web
sh cpweb.sh
cd web
RUST_LOG=info ~/rust-elisp/weblisp/target/release/weblisp --tp
```
