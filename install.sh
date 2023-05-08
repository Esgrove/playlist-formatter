#!/bin/bash
set -eo pipefail

# Import common functions
DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=./common.sh
source "$DIR/common.sh"

if [ -z "$(command -v cargo)" ]; then
    print_error "Cargo not found in path. Maybe install rustup?"
fi

cargo install --path "$REPO_ROOT"

if [ -z "$(command -v playfmt)" ]; then
    print_error "Binary not found. Is the Cargo install directory in path?"
fi
