FROM ubuntu:18.04 as builder
MAINTAINER hidekuno@gmail.com

ENV HOME /root
RUN apt-get update && apt-get -y install git curl libgtk-3-dev |true
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH $PATH:$HOME/.cargo/bin

WORKDIR $HOME
RUN git clone https://github.com/hidekuno/rust-elisp && git clone https://github.com/hidekuno/picture-language

WORKDIR $HOME/rust-elisp/elisp
RUN cargo build --release --bin lisp && strip target/release/lisp

WORKDIR $HOME/rust-elisp/glisp
RUN cargo build --release --features animation --bin glisp && strip target/release/glisp

FROM ubuntu:18.04 as glisp
MAINTAINER hidekuno@gmail.com

RUN apt-get update && apt-get -y install libgtk-3-0
COPY --from=builder /root/picture-language/fractal/ /root/picture-language/fractal/
COPY --from=builder /root/picture-language/sicp/ /root/picture-language/sicp/
COPY --from=builder /root/rust-elisp/elisp/target/release/lisp /root/
COPY --from=builder /root/rust-elisp/glisp/target/release/glisp /root/
