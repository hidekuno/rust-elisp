FROM node:12-buster as builder
MAINTAINER hidekuno@gmail.com

ENV HOME /root
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH $PATH:$HOME/.cargo/bin
RUN rustup target add wasm32-unknown-unknown && cargo install wasm-pack

WORKDIR $HOME
RUN git clone https://github.com/hidekuno/rust-elisp

WORKDIR $HOME/rust-elisp/wasmlisp
RUN wasm-pack build

FROM node:12-alpine as wasmlisp
MAINTAINER hidekuno@gmail.com

WORKDIR /usr/src/app

COPY --from=builder /root/rust-elisp/wasmlisp/package.json ./
COPY --from=builder /root/rust-elisp/wasmlisp/pkg/ ./pkg/

RUN npm install
COPY --from=builder /root/rust-elisp/wasmlisp/webpack.config.js ./
COPY --from=builder /root/rust-elisp/wasmlisp/index.html ./
COPY --from=builder /root/rust-elisp/wasmlisp/index.js ./
COPY --from=builder /root/rust-elisp/wasmlisp/loading.gif ./

EXPOSE 8080

CMD [ "npm", "run", "start" ]
