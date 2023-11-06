Lisp interpreter for FFI(Foreign Function Interface)
=================

## Overview
- This is a lisp interpreter that can be called from scripting languages such as python.

### Requirement
- rust(rustc, cargo..) installed.

## Build
```
cd ${HOME}
git clone https://github.com/hidekuno/rust-elisp
cd rust-elisp/ffilisp
cargo build --release --lib
```

## Test
```
cd ${HOME}
cd rust-elisp/ffilisp
python test.py
```
