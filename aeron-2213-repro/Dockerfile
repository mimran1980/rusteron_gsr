# Deterministic ASAN reproducer for aeron-io/aeron#2213.
#
#   docker build --platform linux/amd64 -t aeron-2213-repro .
#   docker run --rm --platform linux/amd64 aeron-2213-repro
#
# The --platform flag is only needed on arm64 hosts (Docker Desktop with
# Rosetta or qemu). On native x86_64 it can be omitted.

FROM --platform=linux/amd64 ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        git \
        curl \
        ca-certificates \
        libbsd-dev \
        uuid-dev \
        zlib1g-dev \
    && rm -rf /var/lib/apt/lists/*

# Aeron 1.53.3's CMakeLists requires CMake >= 3.30; Ubuntu 24.04 ships 3.28.
ARG CMAKE_VERSION=3.30.9
RUN curl -fsSL \
        "https://github.com/Kitware/CMake/releases/download/v${CMAKE_VERSION}/cmake-${CMAKE_VERSION}-linux-x86_64.tar.gz" \
        | tar -xz -C /opt \
    && ln -s /opt/cmake-${CMAKE_VERSION}-linux-x86_64/bin/* /usr/local/bin/

ARG AERON_REF=1.53.3
RUN git clone --depth 1 --branch "${AERON_REF}" https://github.com/aeron-io/aeron.git /aeron

WORKDIR /aeron

RUN cmake -S . -B build \
        -DCMAKE_BUILD_TYPE=Debug \
        -DBUILD_AERON_DRIVER=ON \
        -DBUILD_AERON_ARCHIVE_API=OFF \
        -DAERON_TESTS=OFF \
        -DAERON_UNIT_TESTS=OFF \
        -DAERON_SYSTEM_TESTS=OFF \
        -DAERON_BUILD_SAMPLES=OFF \
        -DAERON_BUILD_DOCUMENTATION=OFF \
        -DAERON_INSTALL_TARGETS=OFF \
        -DSANITISE_BUILD=ON \
    && cmake --build build --target aeron aeron_driver -j"$(nproc)"

COPY repro.c /repro.c

RUN gcc -g -O1 -fno-omit-frame-pointer -fsanitize=address,undefined \
        -I /aeron/aeron-client/src/main/c \
        -I /aeron/aeron-driver/src/main/c \
        /repro.c -o /repro \
        -L /aeron/build/lib -laeron_driver -laeron \
        -lpthread -lbsd -lrt -lm -luuid

ENV LD_LIBRARY_PATH=/aeron/build/lib
ENV ASAN_OPTIONS=verify_asan_link_order=0:detect_leaks=0:abort_on_error=1

CMD ["/repro"]
