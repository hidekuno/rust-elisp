RustによるLisp(subset版)の実装
=================

## 概要
- Rust手習いのため、Lispの縮小版を実装した。(とりあえずschemeベース)
- 実装目標として、フラクタル図形プログラムを簡単に動作させるための環境を提供する。


## 完成度合い
- 簡単なプログラム(quick-sort, merge-sort, permutations,combination,Eratosthenes等)が動くレベル  

- SICPの図形言語プログラムが動作するところまで確認した。

![image](https://user-images.githubusercontent.com/22115777/57493176-d8702a80-72fe-11e9-86d2-bc3e563e4c40.png)
![image](https://user-images.githubusercontent.com/22115777/57909858-dc7fe780-78be-11e9-9509-3ea7cac4cba9.png)

## 開発環境
| Item   | Ver. |備考|
|--------|--------|--------|
| OS     | CentOS7 | draw系を使わなければ特になし|
| rust   | 1.36.0|1.35以下ではglispでコンパイルエラーになる|
| Gtk+   | 3.22.30||
| rust-gtk |0.7.1|https://github.com/gtk-rs/gtk|
