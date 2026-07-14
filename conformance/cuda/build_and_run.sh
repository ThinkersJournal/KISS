#!/usr/bin/env bash
# KISS-Conform §6.6 on-device slice — POSIX build+run driver (Linux/WSL).
#
# Compiles fmax_ieee.cu with nvcc (host compiler = gcc/clang, no vcvars dance)
# and runs it on the local GPU, propagating the program's exit code
# (0 = PASS, 1 = FAIL, 2 = CUDA env error) for tests/device.rs.
#
# Extra args pass straight through to nvcc, so the negative control is:
#   ./build_and_run.sh -DUSE_FMAXF_INTRINSIC
#
# Env knobs: KISS_CUDA_ARCH (default sm_89).
set -euo pipefail
ARCH="${KISS_CUDA_ARCH:-sm_89}"
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT="$(mktemp -u)_kiss_fmax_ieee"
if ! nvcc -O2 -arch="$ARCH" "$@" "$DIR/fmax_ieee.cu" -o "$OUT"; then
  echo "ERROR: nvcc compile failed"
  exit 2
fi
"$OUT"; rc=$?
rm -f "$OUT"
exit $rc
