#!/usr/bin/env bash

#shellcheck source=../common/lib.sh
source "$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )/../common/lib.sh"


# make checksum
sha256sum target/release/wbuild/*-runtime/target/wasm32-unknown-unknown/release/*.wasm > checksum.sha256
# clean up - FIXME: can we reuse some of the artifacts?
cargo clean
# build again

# confirm checksum
sha256sum -c checksum.sha256
