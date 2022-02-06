Implementation of Lisp (subset version) by Rust
=================

## Overview
- Implemented a Lisp for Rust lessons. (It's Scheme base)
- As an implementation goal, we will provide an environment for easily operating a graphic program.

## Quality
- Level at which a simple program works
    - https://github.com/hidekuno/rust-elisp/tree/master/elisp/samples
    - https://github.com/hidekuno/rust-elisp/blob/master/elisp/tests/integration_test.rs

- I confirmed that the SICP graphic language program works.
    - https://github.com/hidekuno/picture-language

## Directory structure
| crate   | instructions |
|--------|--------|
| [elisp]  | lisp interpreter |
| [glisp]  | GUI for drawing on GTK3 |
| [wasmlisp] | GUI for drawing on Web Assembly |
| [tinywasm](https://github.com/hidekuno/rust-elisp/tree/master/tinywasm) | Sample Program by Web Assembly|
| [weblisp] | Easy Web Serve(Test program for Multithreaded elisp) |
| zlearning |Program for my learning(Not related to elisp)|

## Run on docker(hidekuno/rust-elisp)
### Requirement
- docker is running.
- X Server is running.(XQuartz 2.7.11 for mac)

### Build(my operation log)
```
cd ${HOME}/rust-elisp/docker/glisp
docker build --target=glisp -t hidekuno/rust-elisp --file=./Dockerfile .
docker login
docker push hidekuno/rust-elisp
docker logout
```

### macOS
```
docker pull hidekuno/rust-elisp
xhost +
docker run --name glisp -e DISPLAY=docker.for.mac.localhost:0 hidekuno/rust-elisp /root/glisp
```

<img src="https://user-images.githubusercontent.com/22115777/68745629-5dcff100-063a-11ea-81cc-bf20d05562eb.png" width=50% height=50%>

### Linux
```
docker pull hidekuno/rust-elisp
xhost +
docker run --name glisp -e DISPLAY=${host_ipaddr}:0.0 hidekuno/rust-elisp /root/glisp
```
### Windows11+WSL2+WSLg
```
docker pull hidekuno/rust-elisp
docker run -v /tmp/.X11-unix:/tmp/.X11-unix -e DISPLAY=:0 --name glisp hidekuno/rust-elisp /root/glisp
```

### For environments where the X server is not running
```
docker pull hidekuno/rust-elisp
docker run -it --name elisp hidekuno/rust-elisp /root/lisp
```
<img src="https://user-images.githubusercontent.com/22115777/65646335-bb31c380-e035-11e9-8d12-34b6ce0ee667.png" width=80% height=80%>

## Run on docker(hidekuno/rust-elisp-wasm)
### Requirement
- docker is running.
### Build(my operation log)
```
cd ${HOME}/rust-elisp/docker/wasmlisp
docker build -t hidekuno/rust-elisp-wasm --file=./Dockerfile .
docker login
docker push hidekuno/rust-elisp-wasm
docker logout
```

### RUN
```
docker pull hidekuno/rust-elisp-wasm
docker run --name wasmlisp -p 18080:8080 -d hidekuno/rust-elisp-wasm
```
<img src="https://user-images.githubusercontent.com/22115777/68744951-08471480-0639-11ea-8461-b7d32f38189d.png" width=50% height=50%>

[elisp]: https://github.com/hidekuno/rust-elisp/tree/master/elisp
[glisp]: https://github.com/hidekuno/rust-elisp/tree/master/glisp
[wasmlisp]: https://github.com/hidekuno/rust-elisp/tree/master/wasmlisp
[weblisp]: https://github.com/hidekuno/rust-elisp/tree/master/weblisp
