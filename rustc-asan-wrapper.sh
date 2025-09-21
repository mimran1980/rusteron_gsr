#!/usr/bin/env bash
set -euo pipefail

real_rustc_env="${RUSTC_REAL:-}"
first_arg=""
if [[ $# -gt 0 ]]; then
  first_arg="$1"
  shift
fi

if [[ -n "$real_rustc_env" ]]; then
  real_rustc="$real_rustc_env"
else
  real_rustc="$first_arg"
fi

if [[ -z "$real_rustc" ]]; then
  echo "rustc-asan-wrapper: unable to determine real rustc path" >&2
  exit 1
fi

if [[ -n "${WRAP_DEBUG:-}" ]]; then
  echo "wrapper real rustc: $real_rustc" >&2
  [[ -n "$first_arg" ]] && echo "wrapper initial arg: $first_arg" >&2
fi

if [[ -n "$first_arg" && "$first_arg" != "$real_rustc" && "${first_arg##*/}" != rustc ]]; then
  set -- "$first_arg" "$@"
else
  [[ -n "${WRAP_DEBUG:-}" && -n "$first_arg" ]] && echo "wrapper stripping leading rustc path: $first_arg" >&2
fi

crate_type=""
crate_name="${CARGO_CRATE_NAME:-unknown}"
expect_type=0

for arg in "$@"; do
  if [[ $expect_type -eq 1 ]]; then
    crate_type="$arg"
    expect_type=0
    continue
  fi
  if [[ $arg == --crate-type ]]; then
    expect_type=1
  fi
  if [[ $arg == --crate-name=* ]]; then
    crate_name="${arg#--crate-name=}"
  fi
done

if [[ -n "${WRAP_DEBUG:-}" ]]; then
  echo "wrapper crate_name=$crate_name crate_type=$crate_type" >&2
fi

args=()
for arg in "$@"; do
  if [[ $crate_type == proc-macro ]]; then
    [[ $arg == -Zsanitizer=address ]] && continue
  fi
  args+=("$arg")
done

extra_args=()
if [[ $crate_type == proc-macro ]]; then
  extra_args+=(-Clink-arg=-Wl,-rpath,/usr/lib/llvm-19/lib/clang/19/lib/linux)
  extra_args+=(-Clink-arg=/usr/lib/llvm-19/lib/clang/19/lib/linux/libclang_rt.asan-x86_64.so)
fi

if [[ -n "${WRAP_DEBUG:-}" ]]; then
  printf 'wrapper final args:' >&2
  for arg in "${args[@]}"; do printf ' [%s]' "$arg" >&2; done
  for arg in "${extra_args[@]}"; do printf ' [%s]' "$arg" >&2; done
  printf '\n' >&2
fi

exec "$real_rustc" "${args[@]}" "${extra_args[@]}"
