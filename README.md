RustによるLisp(subset版)の実装
=================

## 概要
- Rust手習いのため、Lispの縮小版を実装した。(とりあえずschemeベース)

- メモ
```
(load-file "~/.emacs.d/rust-mode.el")
(setq rust-format-on-save t)
```

```
cargo run --bin lisp
cargo test --lib
cargo build --help
cargo build --release --bin lisp
```
