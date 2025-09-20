# syntax=docker/dockerfile:1.4
FROM --platform=linux/amd64 rustlang/rust:nightly

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        clang \
        lld \
        gdb \
        valgrind \
        build-essential \
        pkg-config \
        cmake \
        default-jdk-headless \
        libbsd-dev \
        libssl-dev \
    && rm -rf /var/lib/apt/lists/*

#RUN rustup toolchain install nightly-2024-12-05-x86_64-unknown-linux-gnu \
#    && rustup component add rust-src --toolchain nightly-2024-12-05-x86_64-unknown-linux-gnu \
#    && rustup component add rustfmt --toolchain nightly-2024-12-05-x86_64-unknown-linux-gnu \
#    && rustup default nightly-2024-12-05-x86_64-unknown-linux-gnu
RUN rustup component add rustfmt && rustup component add rust-src

#ENV RUSTUP_TOOLCHAIN=nightly-2024-12-05-x86_64-unknown-linux-gnu \
ENV CC=clang \
    CXX=clang++

WORKDIR /work
