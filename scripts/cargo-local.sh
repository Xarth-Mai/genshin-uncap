#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "$0")/.."
toolchain="$PWD/target/toolchain"
if [[ ! -x "$toolchain/cargo/bin/cargo" ]]; then
    echo 'Isolated toolchain is missing; see README.md for normal Rust/MinGW setup' >&2
    exit 1
fi
export CARGO_HOME="$toolchain/cargo"
export RUSTUP_HOME="$toolchain/rustup"
export PATH="$toolchain/cargo/bin:$toolchain/mingw/usr/bin:$PATH"
exec cargo "$@"
