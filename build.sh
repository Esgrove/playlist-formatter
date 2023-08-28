#!/bin/bash
set -eo pipefail

USAGE="Usage: $0 [OPTIONS]

Build the Rust playlist tool.

OPTIONS: All options are optional
    --help
        Display these instructions.

    --verbose
        Display commands being executed.
"

# Import common functions
DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=./common.sh
source "$DIR/common.sh"

init_options() {
    while [ $# -gt 0 ]; do
        case "$1" in
            --help)
                echo "$USAGE"
                exit 1
                ;;
            --verbose)
                set -x
                ;;
        esac
        shift
    done
}

init_options "$@"

if [ -z "$(command -v cargo)" ]; then
    print_error_and_exit "Cargo not found in path. Maybe install rustup?"
fi

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
