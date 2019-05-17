RustによるLisp(subset版)の実装
=================

## 概要
- Rust手習いのため、Lispの縮小版を実装した。(とりあえずschemeベース)
- 実装目標として、フラクタル図形プログラムを簡単に動作させるための環境を提供する。


## 完成度合い
- 簡単なプログラム(quick-sort, merge-sort, permutations,combination,Eratosthenes等)が動くレベル  

- SICPの図形言語プログラムが動作するところまで確認した。

![image](https://user-images.githubusercontent.com/22115777/57493176-d8702a80-72fe-11e9-86d2-bc3e563e4c40.png)
![image](https://user-images.githubusercontent.com/22115777/57898720-353a8a80-7895-11e9-9b0e-d8cc6afb21ca.png)

## 開発環境
| Item   | Ver. |備考|
|--------|--------|--------|
| OS     | CentOS7 | draw系を使わなければ特になし|
| Gtk+   | 3.22.30||
| rust   | 1.33.0||
| rust-gtk |0.6.0|https://github.com/gtk-rs/gtk|
