#!/bin/bash

# This script regenerates the Rust Protocol Buffers code from the definition
# files. It relies on having the protoc compiler and protoc-gen-rust generator
# available in your path. This script does *not* need to be run regularly - only
# when message Protocol Buffers definitions change. The Rust Protocol Buffers
# implementation also (very fortunately) does not rely on the C++ library
# implementation, so all we need on build system is a Rust/Cargo setup. If you
# really need to perform this regeneration on Windows, either use Bash via
# mintty or just perform this invocation by hand.

# Compute the path to the Mutagen source directory.
MUTAGEN_SOURCE="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Compute the path to the proto module directory.
PROTO_SOURCE="${MUTAGEN_SOURCE}/src/proto"

# Perform generation.
pushd "${PROTO_SOURCE}" > /dev/null || exit $?
protoc --rust_out . *.proto || exit $?
popd > /dev/null || exit $?
