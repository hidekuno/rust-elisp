FROM rust:1.71-buster as builder
MAINTAINER hidekuno@gmail.com

ENV HOME /root
RUN apt-get update && apt-get -y install libgtk-3-dev |true

WORKDIR $HOME
RUN git clone https://github.com/hidekuno/rust-elisp && git clone https://github.com/hidekuno/picture-language

WORKDIR $HOME/rust-elisp/elisp
RUN cargo build --release --bin lisp && strip target/release/lisp

WORKDIR $HOME/rust-elisp/glisp
RUN cargo build --release --features animation --bin glisp && strip target/release/glisp

FROM debian:buster-slim as glisp
MAINTAINER hidekuno@gmail.com

RUN apt-get update && apt-get -y install libgtk-3-0
COPY --from=builder /root/picture-language/fractal/ /root/picture-language/fractal/
COPY --from=builder /root/picture-language/sicp/ /root/picture-language/sicp/
COPY --from=builder /root/picture-language/sicp.scm /root/picture-language/
COPY --from=builder /root/rust-elisp/elisp/target/release/lisp /root/
COPY --from=builder /root/rust-elisp/glisp/target/release/glisp /root/
