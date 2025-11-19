---
layout: default
title: C Foreign Function Interface (FFI)
parent: Guides
nav_order: 2
---

This document explains how to use JCL from C and other languages via the C FFI.

## Overview

The JCL C FFI provides a stable, C-compatible API for embedding JCL in other languages. This allows you to:

- Parse and validate JCL configuration files from C/C++ applications
- Format JCL code programmatically
- Run lint checks and get code quality feedback
- Generate documentation from JCL source files
- Embed JCL in applications written in C, C++, Go, Python, Ruby, etc.

## Building

### Quick Build

```bash
./build-ffi.sh
```

### Manual Build

```bash
cargo build --release --features ffi
```

This produces:
- `target/release/libjcl.so` (Linux)
- `target/release/libjcl.dylib` (macOS)
- `target/release/jcl.dll` (Windows)

## Installation

### System-wide Installation (Linux/macOS)

```bash
# Copy library
sudo cp target/release/libjcl.* /usr/local/lib/

# Copy header
sudo cp include/jcl.h /usr/local/include/

# Update library cache (Linux only)
sudo ldconfig
```

### Project-local Installation

```bash
# Copy to your project
cp target/release/libjcl.* your-project/lib/
cp include/jcl.h your-project/include/
```

## Usage

### Basic Example (C)

```c
#include <jcl.h>
#include <stdio.h>

int main() {
    // Initialize
    jcl_init();

    // Parse code
    const char* source = "x = 42\ny = x + 1";
    JclResult result = jcl_parse(source);

    if (result.success) {
        printf("Parse successful: %s\n", result.value);
    } else {
        printf("Parse error: %s\n", result.error);
    }

    // Clean up
    jcl_free_result(&result);

    return 0;
}
```

### Compilation

```bash
# With system-installed library
gcc -o myapp myapp.c -ljcl

# With project-local library
gcc -o myapp myapp.c -I./include -L./lib -ljcl

# Run (Linux)
LD_LIBRARY_PATH=./lib ./myapp

# Run (macOS)
DYLD_LIBRARY_PATH=./lib ./myapp
```

## API Reference

### Initialization

```c
int jcl_init(void);
```

Initialize the JCL library. Must be called before using any other functions.
Returns 0 on success.

### Parse

```c
JclResult jcl_parse(const char* source);
```

Validate JCL syntax. Returns result with success status.

### Format

```c
JclResult jcl_format(const char* source);
```

Auto-format JCL source code. Returns formatted code on success.

### Lint

```c
JclResult jcl_lint(const char* source);
```

Check code for style issues. Returns JSON array of lint issues.

### Generate Documentation

```c
JclResult jcl_generate_docs(const char* source, const char* module_name);
```

Extract documentation from source code. Returns Markdown documentation.

### Version

```c
const char* jcl_version(void);
```

Get JCL version string. Returns static string (do NOT free).

### Memory Management

```c
void jcl_free_result(JclResult* result);
void jcl_free_string(char* ptr);
```

Free memory allocated by JCL. Always call `jcl_free_result()` after using a `JclResult`.

## JclResult Structure

```c
typedef struct {
    bool success;      // True if operation succeeded
    char* value;       // Result value (NULL on error)
    char* error;       // Error message (NULL on success)
} JclResult;
```

## Language Bindings

### C++

```cpp
#include <jcl.h>
#include <iostream>
#include <memory>

class JclResultGuard {
    JclResult result;
public:
    JclResultGuard(JclResult r) : result(r) {}
    ~JclResultGuard() { jcl_free_result(&result); }

    bool success() const { return result.success; }
    const char* value() const { return result.value; }
    const char* error() const { return result.error; }
};

int main() {
    jcl_init();

    JclResultGuard result(jcl_parse("x = 42"));

    if (result.success()) {
        std::cout << "Success: " << result.value() << std::endl;
    } else {
        std::cerr << "Error: " << result.error() << std::endl;
    }

    return 0;
}
```

### Python (ctypes)

```python
import ctypes
import os

# Load library
lib = ctypes.CDLL("./libjcl.so")

# Define structures
class JclResult(ctypes.Structure):
    _fields_ = [
        ("success", ctypes.c_bool),
        ("value", ctypes.c_char_p),
        ("error", ctypes.c_char_p)
    ]

# Define functions
lib.jcl_init.restype = ctypes.c_int
lib.jcl_parse.argtypes = [ctypes.c_char_p]
lib.jcl_parse.restype = JclResult
lib.jcl_free_result.argtypes = [ctypes.POINTER(JclResult)]

# Initialize
lib.jcl_init()

# Parse
source = b"x = 42\ny = x + 1"
result = lib.jcl_parse(source)

if result.success:
    print(f"Success: {result.value.decode('utf-8')}")
else:
    print(f"Error: {result.error.decode('utf-8')}")

# Clean up
lib.jcl_free_result(ctypes.byref(result))
```

### Go

```go
package main

/*
#cgo LDFLAGS: -L. -ljcl
#include "jcl.h"
#include <stdlib.h>
*/
import "C"
import (
    "fmt"
    "unsafe"
)

func main() {
    C.jcl_init()

    source := C.CString("x = 42\ny = x + 1")
    defer C.free(unsafe.Pointer(source))

    result := C.jcl_parse(source)
    defer C.jcl_free_result(&result)

    if result.success {
        fmt.Println("Success:", C.GoString(result.value))
    } else {
        fmt.Println("Error:", C.GoString(result.error))
    }
}
```

### Ruby (FFI)

```ruby
require 'ffi'

module JCL
  extend FFI::Library
  ffi_lib './libjcl.so'

  class Result < FFI::Struct
    layout :success, :bool,
           :value, :pointer,
           :error, :pointer
  end

  attach_function :jcl_init, [], :int
  attach_function :jcl_parse, [:string], Result.by_value
  attach_function :jcl_free_result, [Result.ptr], :void
end

JCL.jcl_init

result = JCL.jcl_parse("x = 42\ny = x + 1")

if result[:success]
  puts "Success: #{result[:value].read_string}"
else
  puts "Error: #{result[:error].read_string}"
end

JCL.jcl_free_result(result.pointer)
```

## Examples

See `examples/c/example.c` for a comprehensive example demonstrating all API functions.

To build and run:

```bash
cd examples/c
gcc -o example example.c -I../../include -L../../target/release -ljcl
LD_LIBRARY_PATH=../../target/release ./example
```

## Thread Safety

The current implementation is **not thread-safe**. If you need to use JCL from multiple threads:

1. Use separate JCL instances per thread (not yet supported)
2. Serialize access with mutexes
3. Process JCL operations in a dedicated thread

Future versions may add thread-safe APIs.

## Memory Management

### Rules

1. **Always free JclResult**: Call `jcl_free_result()` after using any `JclResult`
2. **Don't free version string**: The string from `jcl_version()` is static
3. **Don't use after free**: Pointers become invalid after freeing
4. **No double-free**: Free each result exactly once

### Example

```c
JclResult result = jcl_parse(source);

// Use result
if (result.success) {
    printf("%s\n", result.value);
}

// Free exactly once
jcl_free_result(&result);

// Don't use result.value or result.error after this point!
```

## Error Handling

All functions return `JclResult` with:
- `success == true` and `value != NULL` on success
- `success == false` and `error != NULL` on error

Always check the `success` field before accessing `value` or `error`.

## Performance

- **Parse**: ~1ms for typical files (1-10KB)
- **Format**: ~2ms for typical files
- **Lint**: ~5ms for typical files
- **Memory**: ~1-5MB overhead

Performance is comparable to the native Rust implementation.

## Debugging

### Check Library Loading

```bash
# Linux
ldd myapp
nm -D libjcl.so | grep jcl_

# macOS
otool -L myapp
nm -g libjcl.dylib | grep jcl_
```

### Enable Debug Output

```bash
# Build with debug symbols
cargo build --features ffi

# Run with debug info
RUST_BACKTRACE=1 ./myapp
```

## Limitations

Compared to the native Rust API, the C FFI:

❌ **Not Available:**
- File I/O functions (filesystem operations)
- Direct AST manipulation
- Custom function registration
- Streaming/async operations

✅ **Available:**
- All parsing and formatting features
- Full linter with all rules
- Documentation generation
- All basic JCL functionality

## Building for Distribution

### Static Linking

For fully static binaries:

```bash
RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --target x86_64-unknown-linux-gnu --features ffi
```

### Stripping Debug Symbols

```bash
cargo build --release --features ffi
strip target/release/libjcl.so
```

### Cross-Compilation

```bash
# Install target
rustup target add x86_64-pc-windows-gnu

# Build
cargo build --release --target x86_64-pc-windows-gnu --features ffi
```

## Troubleshooting

### "Library not found" error

Make sure the library is in your library path:

```bash
# Linux
export LD_LIBRARY_PATH=/path/to/lib:$LD_LIBRARY_PATH

# macOS
export DYLD_LIBRARY_PATH=/path/to/lib:$DYLD_LIBRARY_PATH

# Or install system-wide
sudo cp libjcl.* /usr/local/lib/
sudo ldconfig  # Linux only
```

### Compilation errors

Make sure the header is in your include path:

```bash
gcc -I/path/to/include -L/path/to/lib -ljcl myapp.c
```

### Crashes or segfaults

Common causes:
1. Not freeing results (memory leak)
2. Using pointers after freeing
3. Passing NULL to functions that don't allow it
4. Double-freeing results

Use valgrind to debug:

```bash
valgrind --leak-check=full ./myapp
```

## Contributing

Contributions welcome! Areas for improvement:

- [ ] Thread-safe API
- [ ] Async operations
- [ ] Direct AST access
- [ ] Custom function registration
- [ ] Bindings for more languages
- [ ] Performance optimizations
- [ ] Better error messages

## License

Same as JCL project (MIT OR Apache-2.0)

## Resources

- [JCL Documentation](https://github.com/turner-hemmer/jcl)
- [C FFI Header](./include/jcl.h)
- [Example Code](./examples/c/)
- [Rust FFI Book](https://doc.rust-lang.org/nomicon/ffi.html)
