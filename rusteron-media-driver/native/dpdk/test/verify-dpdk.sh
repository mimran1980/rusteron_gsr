#!/usr/bin/env bash
# Compile-verify and test the `dpdk` feature of rusteron-media-driver on
# Linux x86_64. Runs in Docker amd64 emulation (works on an arm64 host);
# builds the environment image once, then runs the full feature build + ABI
# test against the workspace mounted read-only (target dir lives in the
# container, so the host tree is untouched).
#
# Usage: ./verify-dpdk.sh [--build] [test_filter]
#   --build   (re)build the verification image (rusteron-dpdk-verify)
#   test_filter  optional cargo test filter, default `--test dpdk_abi`
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/../../../.." && pwd)"
IMAGE="rusteron-dpdk-verify"
FILTER="${1:---test dpdk_abi}"

if [[ "${1:-}" == "--build" ]]; then
  docker build --platform linux/amd64 -t "$IMAGE" "$HERE"
  shift
  FILTER="${1:---test dpdk_abi}"
fi

docker run --rm --platform linux/amd64 \
  -v "$REPO:/src:ro" \
  -w /src \
  "$IMAGE" \
  bash -c "set -o pipefail; cargo test -p rusteron-media-driver --features dpdk $FILTER 2>&1 | tail -40"
