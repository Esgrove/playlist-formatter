#!/bin/bash
set -eo pipefail

# Get absolute path to repo root
REPO_ROOT=$(git rev-parse --show-toplevel || (cd "$(dirname "${BASH_SOURCE[0]}")" && pwd))
export REPO_ROOT

# Check platform:
# BSD (Mac) and GNU (Linux & Git for Windows) coreutils implementations
# have some differences for example in the available command line options.
case "$(uname -s)" in
    "Darwin")
        export PLATFORM="mac"
        ;;
    "MINGW"*)
        export PLATFORM="windows"
        ;;
    *)
        export PLATFORM="linux"
        ;;
esac

# Print a message with green color
print_green() {
    printf "\e[1;49;32m%s\e[0m\n" "$1"
}

# Print a message with magenta color
print_magenta() {
    printf "\e[1;49;35m%s\e[0m\n" "$1"
}

# Print a message with red color
print_red() {
    printf "\e[1;49;31m%s\e[0m\n" "$1"
}

# Print a message with yellow color
print_yellow() {
    printf "\e[1;49;33m%s\e[0m\n" "$1"
}

# Print an error and exit the program
print_error_and_exit() {
    print_red "ERROR: $1"
    # use exit code if given as second argument, otherwise default to 1
    exit "${2:-1}"
}

# Print usage and exit the program. An optional error message can be given as well.
print_usage_and_exit() {
    if [ $# -eq 1 ]; then
        print_red "ERROR: $1"
    fi
    if [ -z "$USAGE" ]; then
        print_red "No usage text provided in variable USAGE"
    else
        echo "$USAGE"
    fi
    # use exit code if given as second argument, otherwise default to 1
    exit "${2:-1}"
}

# Get the Rust executable name from Cargo.toml
get_rust_executable_name() {
    local executable
    executable=$(awk -F'=' '/\[\[bin\]\]/,/name/ {if($1 ~ /name/) print $2}' Cargo.toml | tr -d ' "')
    # If no name under [[bin]], get the package name
    if [ -z "$executable" ]; then
        executable=$(awk -F'=' '/\[package\]/,/name/ {if($1 ~ /name/) print $2}' Cargo.toml | tr -d ' "')
    fi
    if [ "$PLATFORM" = windows ]; then
        executable="${executable}.exe"
    fi
    echo "$executable"
}
