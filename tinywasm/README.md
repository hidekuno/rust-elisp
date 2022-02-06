Web UI on lisp interpreter
=================

## Overview
- Implemented a Lisp for Rust lessons. (It's Scheme base)

### Requirement
- rust(rustup, rustc, cargo..) installed.

## Install wasm-bindgen
```
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```

## Build & Run
```
cd ${HOME}
git clone https://github.com/hidekuno/rust-elisp
cd rust-elisp/tinywasm
wasm-pack build --target web
python3 -m http.server
```

## Browse this url
```
http://localhost:8000/index.html
```
