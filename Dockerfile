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
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

#RUN rustup toolchain install nightly-2024-12-05-x86_64-unknown-linux-gnu \
#    && rustup component add rust-src --toolchain nightly-2024-12-05-x86_64-unknown-linux-gnu \
#    && rustup component add rustfmt --toolchain nightly-2024-12-05-x86_64-unknown-linux-gnu \
#    && rustup default nightly-2024-12-05-x86_64-unknown-linux-gnu
RUN rustup component add rustfmt && rustup component add rust-src

#ENV RUSTUP_TOOLCHAIN=nightly-2024-12-05-x86_64-unknown-linux-gnu \
ENV CC=clang \
    CXX=clang++ \
    HOME=/work/target/asan \
    TMP=/work/target/asan/tmp \
    TEMP=/work/target/asan/tmp \
    GRADLE_USER_HOME=/work/target/asan/gradle \
    CARGO_HOME=/work/target/asan/cargo-home \
    CARGO_TARGET_DIR=/work/target/asan/target \
    RUSTFLAGS="-Zsanitizer=address" \
    RUSTDOCFLAGS="-Zsanitizer=address" \
    CFLAGS="-fsanitize=address" \
    RUST_TEST_THREADS=1 \
    ASAN_OPTIONS="detect_leaks=1,abort_on_error=1,verify_asan_link_order=0,detect_odr_violation=0"

WORKDIR /work

CMD ["cargo", "+nightly", "test", "-Z", "build-std", "--workspace", "--all", "--all-targets", "--", "--nocapture", "--test-threads=1"]