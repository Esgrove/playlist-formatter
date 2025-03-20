#!/bin/bash
set -eo pipefail

# Import common functions
DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=./common.sh
source "$DIR/common.sh"

USAGE="Usage: $0 [OPTIONS]

Create git version tags for a Rust project.

OPTIONS: All options are optional
    -h | --help
        Display these instructions.

    -d | --dryrun
        Only print commands instead of executing them.

    -f | --force
        Force create tags and push if specified.

    -p | --push
        Push tags to remote.

    --verbose
        Display commands being executed.
"

DRYRUN=false
PUSH=false
FORCE=false
while [ $# -gt 0 ]; do
    case "$1" in
        -h | --help)
            print_usage_and_exit
            ;;
        -d | --dryrun)
            DRYRUN=true
            ;;
        -f | --force)
            FORCE=true
            ;;
        -p | --push)
            PUSH=true
            ;;
        -v | --verbose)
            set -x
            ;;
    esac
    shift
done

current_tag=""
for commit_hash in $(git log --format="%H" --reverse -- Cargo.toml); do
    # Use git show to display changes in the commit for Cargo.toml,
    # and filter for added lines modifying the version
    version_line=$(git show "$commit_hash":Cargo.toml | grep '^version = ' || true)
    if [ -n "$version_line" ]; then
        version_number=$(echo "$version_line" | awk -F'"' '/version = / {print $2}')
        if [ "$current_tag" = "$version_number" ]; then
            print_yellow "Skip $version_number: $commit_hash"
            continue
        else
            current_tag="$version_number"
        fi
        print_magenta "Version $version_number"
        if [ -n "$version_number" ]; then
            tag="v$version_number"
            if [ "$FORCE" = true ]; then
                run_command git tag -af "$tag" "$commit_hash" -m "Rust version $version_number"
            else
                if git tag -l | grep -q "^${tag}$"; then
                    print_red "Tag $tag already exists, skipping..."
                    continue
                else
                    run_command git tag -a "$tag" "$commit_hash" -m "Rust version $version_number"
                fi
            fi
            if [ "$PUSH" = true ]; then
                if [ "$FORCE" = true ]; then
                    run_command git push --force origin "$tag"
                else
                    run_command git push origin "$tag"
                fi
            fi
        fi
    fi
done
