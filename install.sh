#!/bin/bash
set -eo pipefail

# Install the Rust playlist tool version.

# Import common functions
DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=./common.sh
source "$DIR/common.sh"

if [ -z "$(command -v cargo)" ]; then
    print_error_and_exit "Cargo not found in path. Maybe install rustup?"
fi

cd "$REPO_ROOT"

cargo install --path "$REPO_ROOT"

executable=$(get_rust_executable_name)
if [ -z "$(command -v "$executable")" ]; then
    print_error_and_exit "Binary $executable not found. Is the Cargo install directory in path?"
fi
echo "$($executable --version) from $(which "$executable")"
