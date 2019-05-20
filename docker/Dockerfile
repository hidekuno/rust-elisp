FROM centos:centos7
MAINTAINER hidekuno@gmail.com

RUN yum update -y|true
RUN yum install -y epel-release |true
RUN yum install -y rust cargo git gtk3-devel |true
ENV HOME /root

WORKDIR $HOME
RUN git clone https://github.com/hidekuno/rust-elisp

WORKDIR $HOME/rust-elisp/elisp
RUN cargo build --release --lib 
RUN cargo build --release --bin lisp

WORKDIR $HOME/rust-elisp/glisp
RUN cargo build --release --bin glisp
