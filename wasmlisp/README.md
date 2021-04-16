Web UI for drawing on lisp interpreter
=================

## Overview
- Implemented a Lisp for Rust lessons. (It's Scheme base)

### Requirement
- rust(rustup, rustc, cargo..) installed.
- npm installed.
- firefox installed.(for unit test)

## Install wasm-bindgen
```
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```

## Test
```
cd ${HOME}
git clone https://github.com/hidekuno/rust-elisp
cd rust-elisp/wasmlisp
wasm-pack test --headless --firefox -- --lib
```

## Build & Run(on webpack-dev-server)
```
cd ${HOME}
cd rust-elisp/wasmlisp
wasm-pack build
npm install
npm start
```

## Build & Run(on generic web server)
```
cd ${HOME}
cd rust-elisp/wasmlisp
wasm-pack build --target web --out-dir web
sh cpweb.sh
cd web
python3 -m http.server
```

## Run SICP Demo Program
- click "Download SICP" button.

![image](https://user-images.githubusercontent.com/22115777/112743021-b5bc8d00-8fce-11eb-82ed-d5b005951534.png)

- click "Eval Lisp Code" button.

![image](https://user-images.githubusercontent.com/22115777/112743024-bd7c3180-8fce-11eb-961d-c211322f7a97.png)

- click "Demo" button.

![image](https://user-images.githubusercontent.com/22115777/112743027-c40aa900-8fce-11eb-9d2c-ee0b1459d5f4.png)

- click "Eval Lisp Code" button.
