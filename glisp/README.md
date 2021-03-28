GUI for drawing on lisp interpreter
=================

## Overview
- Implemented a Lisp for Rust lessons. (It's Scheme base)

### Requirement
- rust(rustc, cargo..) installed.
- X server is running.
- gtk3 installed.

## Test
```
cd ${HOME}
git clone https://github.com/hidekuno/rust-elisp
cd rust-elisp/glisp
cargo test --lib
```

## Build & Run
```
cd ${HOME}
git clone https://github.com/hidekuno/picture-language
cd rust-elisp/glisp
cargo build --release --features animation --bin glisp
./target/release/glisp
```

## Run SICP Demo Program
- click "SICP" menu button.

![image](https://user-images.githubusercontent.com/22115777/112742610-f2868500-8fca-11eb-813f-956a3051a2d9.png)

- click "Eval" menu button.

![image](https://user-images.githubusercontent.com/22115777/112742527-480e6200-8fca-11eb-8968-2237a7e923b6.png)

- input "(demo)" on textarea, and click "Eval" menu button.
```
(demo)
```
