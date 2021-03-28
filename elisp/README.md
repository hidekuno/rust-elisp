Lisp interpreter  by Rust
=================

## Overview
- Implemented a Lisp for Rust lessons. (It's Scheme base)

### Requirement
- rust(rustc, cargo..) installed.

## Test
```
cd ${HOME}
git clone https://github.com/hidekuno/rust-elisp
cd rust-elisp/elisp
cargo test --lib
cargo test --lib --features thread
cargo test --test integration_test
cargo test --test integration_test  --features thread
```

## Build & Run(single thread)
```
cargo build --release --bin lisp
./target/release/lisp
```

## Build & Run(multi thread)
```
cargo build --release --features thread --bin lisp
./target/release/lisp
```
