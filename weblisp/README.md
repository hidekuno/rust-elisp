Lisp interpreter on easy web Server
=================

## Overview
- Implemented a Lisp for Rust lessons. (It's Scheme base)

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
