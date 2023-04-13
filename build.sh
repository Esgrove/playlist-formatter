#!/bin/bash
set -eo pipefail

REPO_ROOT=$(git rev-parse --show-toplevel || (cd "$(dirname "../${BASH_SOURCE[0]}")" && pwd))

if [ -z "$(command -v cargo)" ]; then
    echo "Cargo not found in path. Maybe install rustup?"
    exit 1
fi

# Check platform
case "$(uname -s)" in
    "Darwin")
        PLATFORM="mac"
        ;;
    "MINGW"*)
        PLATFORM="windows"
        ;;
    *)
        PLATFORM="linux"
        ;;
esac

pushd "$REPO_ROOT" > /dev/null
cargo build --release

if [ "$PLATFORM" = windows ]; then
    executable="playfmt.exe"
else
    executable="playfmt"
fi

rm -f "$executable"
mv ./target/release/"$executable" "$executable"
./"$executable" --version
./"$executable" -h
popd > /dev/null
