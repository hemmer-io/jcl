#!/bin/bash
# Build script for JCL C FFI library

set -e

echo "🚀 Building JCL C FFI library..."

# Build the library with FFI feature
echo "🔨 Building shared library..."
cargo build --release --features ffi

# Get library name based on platform
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    LIB_EXT="so"
    LIB_NAME="libjcl.so"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    LIB_EXT="dylib"
    LIB_NAME="libjcl.dylib"
elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
    LIB_EXT="dll"
    LIB_NAME="jcl.dll"
else
    echo "❌ Unsupported platform: $OSTYPE"
    exit 1
fi

# Check if library was built
if [ -f "target/release/$LIB_NAME" ]; then
    LIB_SIZE=$(du -h "target/release/$LIB_NAME" | cut -f1)
    echo "✅ Build complete!"
    echo ""
    echo "📊 Library size: $LIB_SIZE"
    echo ""
    echo "📂 Output files:"
    echo "  - target/release/$LIB_NAME  (Shared library)"
    echo "  - include/jcl.h             (C header file)"
    echo ""
    echo "🔧 To use in your C project:"
    echo "  1. Copy the library and header:"
    echo "     cp target/release/$LIB_NAME /usr/local/lib/"
    echo "     cp include/jcl.h /usr/local/include/"
    echo ""
    echo "  2. Compile your program:"
    echo "     gcc -o myapp myapp.c -ljcl"
    echo ""
    echo "📝 To build and run the example:"
    echo "  cd examples/c"
    echo "  gcc -o example example.c -I../../include -L../../target/release -ljcl"
    if [[ "$OSTYPE" == "darwin"* ]]; then
        echo "  DYLD_LIBRARY_PATH=../../target/release ./example"
    else
        echo "  LD_LIBRARY_PATH=../../target/release ./example"
    fi
    echo ""
    echo "🎉 Happy coding!"
else
    echo "❌ Library not found at target/release/$LIB_NAME"
    exit 1
fi
