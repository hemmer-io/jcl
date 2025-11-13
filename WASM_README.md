# JCL WebAssembly Build

This document explains how to build and use JCL as a WebAssembly module in the browser.

## Building

### Prerequisites

- Rust toolchain with `wasm32-unknown-unknown` target
- wasm-bindgen-cli (optional, for generating JavaScript bindings)

```bash
# Install the wasm32 target
rustup target add wasm32-unknown-unknown

# Install wasm-bindgen-cli (optional)
cargo install wasm-bindgen-cli
```

### Build for WebAssembly

```bash
# Build the WASM module
cargo build --target wasm32-unknown-unknown --lib --no-default-features --features wasm --release

# The output will be at:
# target/wasm32-unknown-unknown/release/jcl.wasm
```

### Generate JavaScript Bindings (Optional)

If you have `wasm-bindgen-cli` installed, you can generate JavaScript bindings:

```bash
wasm-bindgen target/wasm32-unknown-unknown/release/jcl.wasm \
    --out-dir pkg \
    --target web
```

This will create:
- `pkg/jcl.js` - JavaScript bindings
- `pkg/jcl_bg.wasm` - WebAssembly binary
- `pkg/jcl.d.ts` - TypeScript definitions

## Usage

### Basic Example

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>JCL WebAssembly Demo</title>
</head>
<body>
    <h1>JCL WebAssembly Demo</h1>

    <h2>Input</h2>
    <textarea id="input" rows="10" cols="80">
/// Calculates the sum of two numbers
fn add(x: int, y: int): int = x + y

result = add(5, 3)
    </textarea>

    <h2>Actions</h2>
    <button onclick="parseCode()">Parse</button>
    <button onclick="formatCode()">Format</button>
    <button onclick="lintCode()">Lint</button>
    <button onclick="generateDocs()">Generate Docs</button>

    <h2>Output</h2>
    <pre id="output"></pre>

    <script type="module">
        import init, { Jcl } from './pkg/jcl.js';

        let jcl;

        async function initWasm() {
            await init();
            jcl = new Jcl();
            document.getElementById('output').textContent =
                'JCL WebAssembly module loaded successfully!';
        }

        window.parseCode = function() {
            const input = document.getElementById('input').value;
            const result = jcl.parse(input);

            if (result.is_success()) {
                document.getElementById('output').textContent =
                    'Parse successful!\n\n' + result.value();
            } else {
                document.getElementById('output').textContent =
                    'Parse error:\n\n' + result.error();
            }
        };

        window.formatCode = function() {
            const input = document.getElementById('input').value;
            const result = jcl.format(input);

            if (result.is_success()) {
                document.getElementById('output').textContent = result.value();
            } else {
                document.getElementById('output').textContent =
                    'Error:\n\n' + result.error();
            }
        };

        window.lintCode = function() {
            const input = document.getElementById('input').value;
            const result = jcl.lint(input);

            if (result.is_success()) {
                document.getElementById('output').textContent = result.value();
            } else {
                document.getElementById('output').textContent =
                    'Error:\n\n' + result.error();
            }
        };

        window.generateDocs = function() {
            const input = document.getElementById('input').value;
            const result = jcl.generate_docs(input, 'example');

            if (result.is_success()) {
                document.getElementById('output').textContent = result.value();
            } else {
                document.getElementById('output').textContent =
                    'Error:\n\n' + result.error();
            }
        };

        initWasm();
    </script>
</body>
</html>
```

### Convenience Functions

The module also provides standalone convenience functions:

```javascript
import init, { parse_jcl, format_jcl, lint_jcl, generate_jcl_docs } from './pkg/jcl.js';

await init();

// Parse JCL code
const parseResult = parse_jcl('x = 42');

// Format JCL code
const formatResult = format_jcl('x=42');

// Lint JCL code
const lintResult = lint_jcl('CONSTANT = 42');

// Generate documentation
const docsResult = generate_jcl_docs('fn add(x: int, y: int): int = x + y', 'mymodule');
```

## API Reference

### `Jcl` Class

#### `constructor()`
Creates a new JCL instance.

#### `parse(source: string): JclResult`
Parses JCL source code.

#### `format(source: string): JclResult`
Formats JCL source code.

#### `lint(source: string): JclResult`
Runs the linter on JCL source code and returns issues as JSON.

#### `generate_docs(source: string, module_name: string): JclResult`
Generates Markdown documentation from JCL source code.

#### `version(): string` (static)
Returns the JCL version.

### `JclResult` Class

#### `is_success(): boolean`
Returns true if the operation succeeded.

#### `value(): string`
Returns the result value (empty string if error).

#### `error(): string`
Returns the error message (empty string if success).

## Features Available in WASM

✅ **Available:**
- Parser
- Formatter
- Linter
- Documentation generator
- String functions
- Encoding/decoding functions
- Collection functions
- Math functions
- Hash functions
- Date/time functions
- Template functions

❌ **Not Available (require filesystem access):**
- `file()` - Read file contents
- `fileexists()` - Check if file exists
- `dirname()` - Get directory name
- `basename()` - Get base name
- `abspath()` - Get absolute path
- `templatefile()` - Load template from file

## Bundle Size

The release build produces a `.wasm` file of approximately:
- **Uncompressed:** ~2-3 MB
- **Gzip compressed:** ~600-800 KB

To minimize bundle size:
1. Use release builds (`--release`)
2. Enable LTO in Cargo.toml (already configured)
3. Serve with gzip or brotli compression
4. Consider using `wasm-opt` for further optimization:
   ```bash
   wasm-opt -Oz -o output.wasm input.wasm
   ```

## Development

### Running Tests

```bash
# Run all tests including WASM tests
cargo test --target wasm32-unknown-unknown --features wasm

# Run WASM tests in browser (requires wasm-pack)
wasm-pack test --headless --chrome
```

### Building with wasm-pack

Alternatively, you can use `wasm-pack` for a more integrated build experience:

```bash
# Install wasm-pack
cargo install wasm-pack

# Build for web
wasm-pack build --target web --features wasm --no-default-features

# Build for Node.js
wasm-pack build --target nodejs --features wasm --no-default-features

# Build for bundlers (webpack, rollup, etc.)
wasm-pack build --target bundler --features wasm --no-default-features
```

## Limitations

1. **No File I/O:** Filesystem operations are not available in the browser
2. **No Async/Await:** The current API is synchronous (future versions may add async support)
3. **Memory Management:** Be mindful of memory usage when processing large files
4. **Error Handling:** Errors are returned as strings, not structured objects

## Future Enhancements

- [ ] Add async API for long-running operations
- [ ] Support for custom function registration from JavaScript
- [ ] Stream processing API for large files
- [ ] Worker thread support for background processing
- [ ] Source map generation for debugging

## License

Same as the main JCL project (MIT OR Apache-2.0)
