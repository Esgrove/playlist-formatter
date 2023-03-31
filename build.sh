#!/bin/bash
set -eo pipefail

REPO_ROOT=$(git rev-parse --show-toplevel || (cd "$(dirname "../${BASH_SOURCE[0]}")" && pwd))

if [ -z "$(command -v cargo)" ]; then
    echo "Cargo not found in path. Maybe install rustup?"
    exit 1
fi

pushd "$REPO_ROOT" > /dev/null
cargo build --release
mv ./target/release/playlist_tool .
./playlist_tool --version
./playlist_tool -h
popd > /dev/null
