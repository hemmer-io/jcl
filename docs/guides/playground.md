---
layout: default
title: Online Playground
parent: Guides
nav_order: 5
---

An interactive, browser-based playground for trying JCL without installing anything.

## Features

- **Live Editing:** Write JCL code in a Monaco-style editor
- **Instant Feedback:** Parse, format, lint, and generate docs in real-time
- **Example Library:** Pre-loaded examples demonstrating JCL features
- **Tabbed Interface:** Switch between output, formatted code, lint results, and documentation
- **Keyboard Shortcuts:**
  - `Ctrl/Cmd + S` - Format code
  - `Ctrl/Cmd + Enter` - Parse code
- **Dark Theme:** Easy on the eyes with VSCode-inspired styling

## Quick Start

### 1. Build the WASM Module

```bash
# Easy way: use the build script
./build-wasm.sh

# Or manual build:
cargo build --target wasm32-unknown-unknown --lib --no-default-features --features wasm --release
wasm-bindgen target/wasm32-unknown-unknown/release/jcl.wasm --out-dir pkg --target web
```

### 2. Start a Web Server

```bash
# Using Python 3
python -m http.server 8000

# Using Python 2
python -m SimpleHTTPServer 8000

# Using Node.js
npx http-server

# Using Rust
cargo install simple-http-server
simple-http-server -p 8000
```

### 3. Open in Browser

Navigate to http://localhost:8000/playground.html

## Usage

### Editor Panel (Left)
- Write or edit JCL code
- Select examples from the dropdown
- Use keyboard shortcuts for quick actions

### Output Panel (Right)
Four tabs show different information:

1. **Output** - General output and parse results
2. **Formatted** - Auto-formatted code
3. **Lint Results** - Code quality issues with severity levels
4. **Documentation** - Generated Markdown documentation

### Actions

- **✓ Parse** - Validates syntax and shows parse tree
- **✨ Format** - Auto-formats code and updates editor
- **🔍 Lint** - Checks for style and quality issues
- **📄 Docs** - Generates API documentation from code

## Examples

The playground includes several examples:

1. **Hello World** - Basic syntax and variables
2. **Functions** - Function definitions with type annotations
3. **Collections** - Lists, maps, and comprehensions
4. **Templates** - String interpolation and template rendering
5. **Conditionals** - If-then-else and pattern matching
6. **Loops** - For loops and list comprehensions
7. **Complex Example** - Real-world configuration scenario

## Architecture

```
playground.html (UI)
    ↓
pkg/jcl.js (Generated bindings)
    ↓
pkg/jcl_bg.wasm (JCL compiled to WASM)
```

The playground uses:
- **wasm-bindgen** for Rust ↔ JavaScript interop
- **WebAssembly** for running JCL in the browser
- Pure HTML/CSS/JS (no framework dependencies)

## Deployment

### GitHub Pages

```bash
# Build WASM
./build-wasm.sh

# Commit and push
git add pkg/ playground.html
git commit -m "Deploy playground"
git push

# Enable GitHub Pages in repository settings
# Point to the branch with playground.html
```

### Netlify / Vercel

1. Build WASM: `./build-wasm.sh`
2. Deploy the entire repository
3. Set `playground.html` as the entry point

### Static Hosting

Upload these files to any static host:
- `playground.html`
- `pkg/jcl.js`
- `pkg/jcl_bg.wasm`
- `pkg/jcl.d.ts` (optional, for TypeScript support)

## Customization

### Adding Examples

Edit the `examples` object in `playground.html`:

```javascript
const examples = {
    myexample: `/// My custom example
x = 42
y = "hello"`,
    // ... more examples
};
```

Then add an option to the dropdown:

```html
<option value="myexample">My Example</option>
```

### Changing Theme

Modify the CSS variables in `<style>`:

```css
body {
    background-color: #1e1e1e; /* Background */
    color: #d4d4d4;            /* Text */
}

button {
    background-color: #0e639c; /* Primary button */
}
```

### Adding Features

The WASM module exposes these APIs:

```javascript
// Parse code
const result = jcl.parse(sourceCode);

// Format code
const formatted = jcl.format(sourceCode);

// Lint code
const lintResults = jcl.lint(sourceCode);

// Generate docs
const docs = jcl.generate_docs(sourceCode, "moduleName");

// Check result
if (result.is_success()) {
    console.log(result.value());
} else {
    console.error(result.error());
}
```

## Browser Compatibility

- Chrome/Edge: ✅ Full support
- Firefox: ✅ Full support
- Safari: ✅ Full support (14+)
- Mobile browsers: ✅ Responsive design

WebAssembly is supported in all modern browsers.

## Performance

- **Initial Load:** ~800KB (gzipped)
- **Parse Time:** < 10ms for typical files
- **Format Time:** < 20ms for typical files
- **Memory Usage:** ~5-10MB

Performance is comparable to native execution for most operations.

## Limitations

The WASM playground has some limitations compared to the CLI:

❌ **Not Available:**
- File I/O operations (`file()`, `fileexists()`, etc.)
- External process execution
- Native filesystem access
- Template file loading (`templatefile()`)

✅ **Available:**
- All parsing and formatting features
- Linting with all rules
- Documentation generation
- Template rendering (inline only)
- All built-in functions except file I/O

## Troubleshooting

### "Failed to load WASM module"

1. Make sure you built the WASM module: `./build-wasm.sh`
2. Check that `pkg/` directory exists with `.wasm` and `.js` files
3. Verify you're using a web server (not `file://` protocol)
4. Check browser console for detailed errors

### "Module not found" error

The WASM module path is hardcoded as `./pkg/jcl.js`. If you move files:

1. Update the import path in `playground.html`:
   ```javascript
   const wasmModule = await import('./your-path/jcl.js');
   ```

### Slow load times

1. Enable compression on your web server (gzip/brotli)
2. Use CDN for faster delivery
3. Consider using `wasm-opt` to optimize the WASM binary:
   ```bash
   wasm-opt -Oz -o pkg/jcl_bg.opt.wasm pkg/jcl_bg.wasm
   ```

### Memory issues with large files

The playground runs entirely in the browser. For very large files (>1MB):
1. Consider chunking/streaming the content
2. Add a file size warning
3. Implement lazy evaluation

## Development

### Local Development

```bash
# Install dependencies
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli

# Build and watch for changes
./build-wasm.sh

# In another terminal, start server
python -m http.server 8000

# Open http://localhost:8000/playground.html
# Edit code, rebuild, refresh browser
```

### Testing

```bash
# Run WASM tests
cargo test --target wasm32-unknown-unknown --features wasm

# Run browser tests (requires wasm-pack)
wasm-pack test --headless --chrome
```

## Contributing

Contributions welcome! Ideas for improvements:

- [ ] Syntax highlighting in editor
- [ ] Auto-completion support
- [ ] Error highlighting with line numbers
- [ ] Share code via URL (encode in hash)
- [ ] Export generated documentation
- [ ] Multi-file support
- [ ] AST visualization
- [ ] Performance profiling
- [ ] Mobile-optimized layout
- [ ] Offline support (Service Worker)

## License

Same as JCL project (MIT OR Apache-2.0)

## Resources

- [JCL Documentation](https://github.com/turner-hemmer/jcl)
- [WASM README](./WASM_README.md)
- [WebAssembly](https://webassembly.org/)
- [wasm-bindgen](https://rustwasm.github.io/wasm-bindgen/)
