#!/bin/bash
set -eo pipefail

USAGE="Usage: $0 [OPTIONS]

Build the C++ Qt version of the playlist tool.

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
    BUILD_TYPE="Release"

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

    CMAKE_BUILD_DIR="$REPO_ROOT/cmake-build-$PLATFORM-$(echo "$BUILD_TYPE" | tr '[:upper:]' '[:lower:]')"
}

generate_msvc_project() {
    cmake -B "$CMAKE_BUILD_DIR" \
        -G "Visual Studio 17 2022" \
        -A x64 \
        -S "$REPO_ROOT" \
        -DCMAKE_BUILD_TYPE="$BUILD_TYPE"
}

generate_ninja_project() {
    cmake -B "$CMAKE_BUILD_DIR" \
        -G Ninja \
        -S "$REPO_ROOT" \
        -DCMAKE_BUILD_TYPE="$BUILD_TYPE"
}

build_project() {
    pushd "$REPO_ROOT" > /dev/null
    mkdir -p "$CMAKE_BUILD_DIR"
    if [ "$PLATFORM" = windows ]; then
        print_magenta "Generating Visual Studio project..."
        if ! generate_msvc_project; then
            print_yellow "CMake failed, removing existing cache and trying again..."
            rm -rf "$CMAKE_BUILD_DIR"
            generate_msvc_project
        fi
        print_magenta "Building $BUILD_TYPE..."
        cmake --build "$CMAKE_BUILD_DIR" --target "PlaylistTool" --config "$BUILD_TYPE"
    else
        print_magenta "Generating Ninja build..."
        if ! generate_ninja_project; then
            print_yellow "CMake failed, removing existing cache and trying again..."
            rm -rf "$CMAKE_BUILD_DIR"
            generate_ninja_project
        fi
        print_magenta "Building $BUILD_TYPE..."
        time ninja -C "$CMAKE_BUILD_DIR" PlaylistTool
    fi
    print_green "Build succeeded"
    popd > /dev/null
}

move_exe_to_root() {
    pushd "$REPO_ROOT" > /dev/null
    if [ "$PLATFORM" = windows ]; then
        APP_NAME="PlaylistTool.exe"
        APP_EXECUTABLE="PlaylistTool.exe"
    else
        APP_NAME="PlaylistTool.app"
        APP_EXECUTABLE="PlaylistTool.app/Contents/MacOS/PlaylistTool"
    fi
    rm -rf "$APP_NAME"
    # Move executable from build dir to project root
    mv "$(find "$CMAKE_BUILD_DIR" -name "$APP_NAME")" "$APP_NAME"
    file "$APP_EXECUTABLE"
    # Run executable to check it works and print the version info
    ./"$APP_EXECUTABLE" --version
    ./"$APP_EXECUTABLE" --help
    popd > /dev/null
}

init_options "$@"
build_project
move_exe_to_root
