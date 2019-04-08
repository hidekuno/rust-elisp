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

yum search  valgrind
sudo yum install  valgrind
cargo install cargo-profiler
export PATH=$PATH:/home/kunohi/.cargo/bin
cargo build --release --bin profile_lisp
cargo profiler callgrind --bin ./target/release/profile_lisp -n 10
```
