# syntax=docker/dockerfile:1.4

FROM --platform=linux/amd64 rustlang/rust:nightly AS base

ARG RUST_NIGHTLY=nightly-2024-08-15

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        default-jdk-headless \
        curl \
        pkg-config \
        libssl-dev \
        uuid-dev \
        ca-certificates \
        make \
        cmake \
        gcc \
        g++ \
        clang \
        zlib1g-dev \
        libbsd-dev \
        gdb \
        valgrind \
        libclang-rt-19-dev \
        openssh-server \
    && rm -rf /var/lib/apt/lists/*

COPY rustc-asan-wrapper.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/rustc-asan-wrapper.sh

RUN rustup toolchain install ${RUST_NIGHTLY} --component rustfmt --component rust-src \
    && rustup default ${RUST_NIGHTLY}

WORKDIR /work

RUN mkdir -p /var/run/sshd && \
    { \
      echo 'Port 22'; \
      echo 'ListenAddress 127.0.0.1'; \
      echo 'PasswordAuthentication no'; \
      echo 'PubkeyAuthentication yes'; \
      echo 'PermitRootLogin prohibit-password'; \
    } >> /etc/ssh/sshd_config

RUN mkdir -p /root/.ssh && chmod 700 /root/.ssh
COPY id_ed25519.pub /root/.ssh/authorized_keys
RUN chmod 600 /root/.ssh/authorized_keys && chown -R root:root /root/.ssh

EXPOSE 22

FROM base AS asan

ARG RUST_NIGHTLY=nightly-2024-08-15

ENV RUSTUP_TOOLCHAIN=${RUST_NIGHTLY}-x86_64-unknown-linux-gnu
ENV RUSTC_WRAPPER=/usr/local/bin/rustc-asan-wrapper.sh
ENV RUSTC_REAL=/usr/local/rustup/toolchains/${RUSTUP_TOOLCHAIN}/bin/rustc
ENV HOME=/work/target/asan
ENV TMP=/work/target/asan/tmp
ENV TEMP=/work/target/asan/tmp
ENV GRADLE_USER_HOME=/work/target/asan/gradle
ENV CARGO_HOME=/work/target/asan/cargo-home
ENV CARGO_TARGET_DIR=/work/target/asan/target
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS=-Zsanitizer=address
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTDOCFLAGS=-Zsanitizer=address
ENV RUSTFLAGS=-Zsanitizer=address
ENV RUSTDOCFLAGS=-Zsanitizer=address
ENV CFLAGS=-fsanitize=address
ENV RUST_TEST_THREADS=1
ENV ASAN_OPTIONS=detect_leaks=1,abort_on_error=1,verify_asan_link_order=0,detect_odr_violation=0

CMD ["cargo", "+nightly", "test", "--workspace", "--all", "--all-targets", "--", "--nocapture", "--test-threads=1"]

FROM base AS valgrind

ARG RUST_NIGHTLY=nightly-2024-08-15

CMD ["cargo", "+nightly", "test", "--workspace", "--all", "--all-targets", "--", "--nocapture", "--test-threads=1"]
