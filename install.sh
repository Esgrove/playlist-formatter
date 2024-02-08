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

cargo install --path "$REPO_ROOT"

if [ -z "$(command -v playfmt)" ]; then
    print_error_and_exit "Binary not found. Is the Cargo install directory in path?"
fi

executable=$(get_rust_executable_name)
echo "$($executable --version) from $(which "$executable")"
