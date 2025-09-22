ARG RUST_VERSION=nightly
FROM --platform=linux/amd64 rustlang/rust:${RUST_VERSION}

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

CMD ["cargo", "test", "--workspace", "--all", "--all-targets", "--", "--nocapture", "--test-threads=1"]
