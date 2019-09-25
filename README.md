RustによるLisp(subset版)の実装
=================

## 概要
- Rust手習いのため、Lispの縮小版を実装した。(とりあえずschemeベース)
- 実装目標として、フラクタル図形プログラムを簡単に動作させるための環境を提供する。


## 完成度合い
- SICPの図形言語プログラムが動作するところまで確認した。(https://sicp.iijlab.net/fulltext/x224.html)

<img src="https://user-images.githubusercontent.com/22115777/65193640-bf4e6600-dab6-11e9-9851-eb001450a08d.png" width=60%>

## 開発環境
| Item   | Ver. |備考|
|--------|--------|--------|
| OS     | CentOS7 | draw系を使わなければ特になし|
| rust   | 1.37.0|1.35以下ではglispでコンパイルエラーになる|
| Gtk+   | 3.22.30||
| rust-gtk |0.7.1|https://github.com/gtk-rs/gtk|

## 動かし方
### 動作条件
- dockerが動いていること
- Xサーバ(macの場合、XQuartz)が動いていること

### macOS
```
docker pull hidekuno/rust-elisp
docker run --name glisp -e DISPLAY=docker.for.mac.localhost:0 hidekuno/rust-elisp /root/glisp
```

### Linux
```
docker pull hidekuno/rust-elisp
xhost + 
docker run --name glisp -e DISPLAY=${host_ipaddr}:0.0 hidekuno/rust-elisp /root/glisp
```

### Xサーバが動いていない環境向け(replのみ版)
```
docker pull hidekuno/rust-elisp
docker run -it --name elisp hidekuno/rust-elisp /root/lisp
```
<img src="https://user-images.githubusercontent.com/22115777/65646335-bb31c380-e035-11e9-8d12-34b6ce0ee667.png" width=80%>
