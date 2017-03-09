#!/bin/bash

# This script regenerates the Rust Protocol Buffers code from the definition
# files. It relies on having the protoc compiler and protoc-gen-rust generator
# available in your path. This script does *not* need to be run regularly - only
# when message Protocol Buffers definitions change. The Rust Protocol Buffers
# implementation also (very fortunately) does not rely on the C++ library
# implementation, so all we need on build system is a Rust/Cargo setup. If you
# really need to perform this regeneration on Windows, either use Bash via
# mintty or just perform these invocations by hand.

# Define a list of directories where code generation is required.
SUBPATHS=(
    "src/url"
)

# Compute the path to the Mutagen source directory.
MUTAGEN_SOURCE="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Perform generation.
for p in "${SUBPATHS[@]}"; do
    echo "Processing $MUTAGEN_SOURCE/$p"
    pushd "$MUTAGEN_SOURCE/$p" > /dev/null
    protoc --rust_out . *.proto
    popd > /dev/null
done
