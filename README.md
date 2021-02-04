RustによるLisp(subset版)の実装
=================

## 概要
- Rust手習いのため、Lispの縮小版を実装した。(とりあえずschemeベース)
- 実装目標として、フラクタル図形プログラムを簡単に動作させるための環境を提供する。

## 完成度合い
- 簡単なプログラムが動くレベル  
    - https://github.com/hidekuno/rust-elisp/tree/master/elisp/samples  
    - https://github.com/hidekuno/rust-elisp/blob/master/elisp/tests/integration_test.rs

- SICPの図形言語プログラムが動作するところまで確認した。
    - https://github.com/hidekuno/picture-language

## 構成
| crate   | instructions |
|--------|--------|
| elisp  | lisp本体 |
| glisp  | 図形描画用GUI |
| wasmlisp | Web Assembly用UI |
| weblisp | マルチスレッド検証用プログラム |
| zlearning |rust学習プログラム(hidekuno/tree-viewerとのパフォーマンス比較検証)|

## 動かし方
### 動作条件
- dockerが動いていること
- Xサーバ(macの場合、XQuartz 2.7.11)が動いていること

### macOS
```
docker pull hidekuno/rust-elisp
xhost +
docker run --name glisp -e DISPLAY=docker.for.mac.localhost:0 hidekuno/rust-elisp /root/glisp
```

<img src="https://user-images.githubusercontent.com/22115777/68745629-5dcff100-063a-11ea-81cc-bf20d05562eb.png" width=80%>

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


## 動かし方(Web Assembly版)
### 動作条件
- dockerが動いていること

```
docker pull hidekuno/rust-elisp-wasm
docker run --name wasmlisp -p 18080:8080 -d hidekuno/rust-elisp-wasm
```
<img src="https://user-images.githubusercontent.com/22115777/68744951-08471480-0639-11ea-8461-b7d32f38189d.png" width=80%>
