FROM centos:centos7 as builder
MAINTAINER hidekuno@gmail.com

RUN yum update -y && yum install -y epel-release && yum install -y npm git gcc openssl openssl-devel |true
ENV HOME /root

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH $PATH:$HOME/.cargo/bin
RUN rustup target add wasm32-unknown-unknown && cargo install wasm-pack

WORKDIR $HOME
RUN git clone https://github.com/hidekuno/rust-elisp

WORKDIR $HOME/rust-elisp/wasmlisp
RUN wasm-pack build

FROM node:6.1 as wasmlisp
MAINTAINER hidekuno@gmail.com

WORKDIR /usr/src/app

COPY --from=builder /root/rust-elisp/wasmlisp/package.json ./
COPY --from=builder /root/rust-elisp/wasmlisp/pkg/ ./pkg/

RUN npm install
COPY --from=builder /root/rust-elisp/wasmlisp/webpack.config.js ./
COPY --from=builder /root/rust-elisp/wasmlisp/index.html ./
COPY --from=builder /root/rust-elisp/wasmlisp/index.js ./

EXPOSE 8080

CMD [ "npm", "run", "lisp" ]