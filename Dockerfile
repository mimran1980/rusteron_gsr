# syntax=docker/dockerfile:1.4
FROM --platform=linux/amd64 rustlang/rust:nightly

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        default-jdk-headless curl pkg-config libssl-dev uuid-dev ca-certificates make cmake gcc g++ clang zlib1g-dev libbsd-dev \
        gdb \
        valgrind \
        libclang-rt-19-dev \
        openssh-server \
    && rm -rf /var/lib/apt/lists/*

COPY rustc-asan-wrapper.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/rustc-asan-wrapper.sh

#RUN rustup toolchain install nightly-2024-12-05-x86_64-unknown-linux-gnu \
#    && rustup component add rust-src --toolchain nightly-2024-12-05-x86_64-unknown-linux-gnu \
#    && rustup component add rustfmt --toolchain nightly-2024-12-05-x86_64-unknown-linux-gnu \
#    && rustup default nightly-2024-12-05-x86_64-unknown-linux-gnu
RUN rustup component add rustfmt && rustup component add rust-src && rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu

#ENV RUSTUP_TOOLCHAIN=nightly-2024-12-05-x86_64-unknown-linux-gnu \
WORKDIR /work

RUN mkdir -p /var/run/sshd && \
    { echo 'Port 22'; echo 'ListenAddress 127.0.0.1'; echo 'PasswordAuthentication no'; \
      echo 'PubkeyAuthentication yes'; echo 'PermitRootLogin prohibit-password'; } >> /etc/ssh/sshd_config

RUN mkdir -p /root/.ssh && chmod 700 /root/.ssh
COPY id_ed25519.pub /root/.ssh/authorized_keys
RUN chmod 600 /root/.ssh/authorized_keys && chown -R root:root /root/.ssh
EXPOSE 22
# Runtime env only applies to final container runtime
ENV RUSTC_WRAPPER=/usr/local/bin/rustc-asan-wrapper.sh \
    RUSTC_REAL=/usr/local/rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustc \
    HOME=/work/target/asan \
    TMP=/work/target/asan/tmp \
    TEMP=/work/target/asan/tmp \
    GRADLE_USER_HOME=/work/target/asan/gradle \
    CARGO_HOME=/work/target/asan/cargo-home \
    CARGO_TARGET_DIR=/work/target/asan/target \
    CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-Zsanitizer=address" \
    CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTDOCFLAGS="-Zsanitizer=address" \
    RUSTFLAGS="-Zsanitizer=address" \
    RUSTDOCFLAGS="-Zsanitizer=address" \
    CFLAGS="-fsanitize=address" \
    RUST_TEST_THREADS=1 \
    ASAN_OPTIONS="detect_leaks=1,abort_on_error=1,verify_asan_link_order=0,detect_odr_violation=0"
#CMD ["/usr/sbin/sshd", "-D"]

CMD ["cargo", "+nightly", "test", "--workspace", "--all", "--all-targets", "--", "--nocapture", "--test-threads=1"]
