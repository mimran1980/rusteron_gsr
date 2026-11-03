#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
docker build --platform linux/amd64 -t aeron-2213-repro .
exec docker run --rm --platform linux/amd64 aeron-2213-repro
