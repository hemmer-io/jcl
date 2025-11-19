# JCL - Jack-of-All Configuration Language

A modern, safe, and flexible general-purpose configuration language with powerful built-in functions, written in Rust.

## Vision

JCL is a general-purpose configuration language designed to be human-readable, type-safe, and powerful. It provides a rich standard library of functions for data manipulation, encoding/decoding (YAML, JSON, Base64), templating, string operations, and more. Built in Rust for performance and safety, JCL can be embedded in other tools (like Hemmer for IaC) or used standalone for configuration management.

## Installation

### Via Cargo (Rust)

```bash
cargo install jcl
```

### Via Binary Download

Download pre-built binaries from the [releases page](https://github.com/hemmer-io/jcl/releases).

### From Source

```bash
git clone https://github.com/hemmer-io/jcl.git
cd jcl
cargo build --release
```

## Quick Start

```bash
# Run the interactive REPL
jcl repl

# Evaluate a JCL file
jcl eval config.jcl

# Format JCL files
jcl-fmt config.jcl

# Validate against a schema
jcl-validate --schema schema.jcl config.jcl

# Migrate from other formats
jcl-migrate config.json --from json
```

## Documentation

- **[Getting Started Guide](https://hemmer-io.github.io/jcl/getting-started/)** - Learn JCL basics
- **[Language Specification](https://hemmer-io.github.io/jcl/reference/language-spec)** - Complete syntax reference
- **[Built-in Functions](https://hemmer-io.github.io/jcl/reference/functions)** - 70+ functions documented
- **[CLI Tools](https://hemmer-io.github.io/jcl/reference/cli-tools)** - Command-line utilities
- **[Comparison Guide](https://hemmer-io.github.io/jcl/guides/comparison)** - JCL vs other formats

## Key Features

🎯 **General-Purpose Configuration**
- Clean, human-readable syntax with minimal punctuation
- Rich standard library of 70+ built-in functions
- Can be embedded or used standalone

🔒 **Type Safety**
- Advanced static type inference catches errors before runtime
- Expression-level type checking with Hindley-Milner style inference
- Runtime type validation with annotations
- Immutability by default

🚀 **Powerful Built-in Functions**
- **String operations**: upper, lower, trim, replace, split, join, format
- **Encoding/Decoding**: JSON, YAML, TOML, Base64, URL encoding
- **Collections**: merge, lookup, keys, values, sort, distinct, flatten
- **Numeric**: min, max, sum, avg, abs, ceil, floor, round
- **Hashing**: MD5, SHA1, SHA256, SHA512
- **Templating**: String interpolation, conditional content, loops in templates
- **Filesystem**: file, fileexists, dirname, basename
- **Type conversion**: tostring, tonumber, tobool
- And more...

🏗️ **Flexible Syntax**
- Parentheses-based grouping (not braces)
- Dot notation for namespacing
- No quotes needed for simple values
- String interpolation: `"Hello, ${name}!"`
- Progressive disclosure: can be concise or explicit

## Example

```jcl
# Simple, readable configuration
environments = ["prod", "dev", "staging"]

env_prod = (
  region = "us-west-2",
  vars = (
    app_name = "myapp",
    version = "1.2.3",
    replicas = 3
  ),
  tags = (
    team = "platform",
    cost_center = "engineering"
  )
)

# String interpolation
greeting = "Hello, ${env_prod.vars.app_name}!"

# Built-in functions
uppercased = upper(env_prod.vars.app_name)
config_json = jsonencode(env_prod.vars)
config_yaml = yamlencode(env_prod.vars)

# Collections and data manipulation
regions = ["us-west-2", "us-east-1", "eu-west-1"]
region_count = length(regions)
merged_tags = merge(env_prod.tags, (environment = "prod", managed_by = "jcl"))

# List comprehensions
formatted_regions = [upper(r) for r in regions]
sorted_regions = sort(formatted_regions)
joined_regions = join(sorted_regions, ", ")
```

## Features

### Core Language
- **Type System**: Advanced static type inference with expression-level checking
- **Collections**: Lists `[]` and maps `()` with comprehensive manipulation functions
- **String Interpolation**: `"Hello, ${name}!"` syntax for dynamic strings
- **Null Safety**: `?.` optional chaining and `??` null coalescing operators
- **Functions**: Lambda expressions (`x => x * 2`) and named functions (`fn double(x) = x * 2`)
- **Control Flow**: Ternary operators, if/then/else, when expressions
- **List Comprehensions**: `[x * 2 for x in numbers if x > 0]`
- **Error Handling**: `try()` for graceful error recovery
- **Import System**: Modular configuration files

### Tooling Ecosystem
- **REPL**: Interactive shell with history and state management
- **Formatter** (`jcl-fmt`): Automatic code formatting with configurable style
- **Linter**: 9 comprehensive lint rules for code quality
- **LSP**: Full Language Server Protocol support (diagnostics, autocomplete, go-to-definition, rename)
- **Validator** (`jcl-validate`): Schema-based validation
- **Migrator** (`jcl-migrate`): Convert from JSON, YAML, TOML to JCL
- **Watcher** (`jcl-watch`): Auto-format on file changes
- **Benchmarking** (`jcl-bench`): Performance measurement tools

### Multi-Language Support
- **Rust**: `cargo install jcl` - Native implementation
- **Python**: `pip install jcl-lang` - PyO3 bindings
- **Node.js**: `npm install @hemmer-io/jcl` - Neon bindings
- **Ruby**: `gem install jcl` - Magnus bindings
- **Go**: cgo bindings for Go projects
- **Java**: JNI bindings for Java applications
- **WebAssembly**: Browser and serverless support
- **C FFI**: Embed in any language with C interop

### Editor Support
- **VSCode**: Full syntax highlighting and LSP integration
- **Vim/Neovim**: Syntax files and LSP support
- **Any LSP-compatible editor**: Diagnostics, autocomplete, formatting

## Architecture

```
Parser → Type Checker → Evaluator (with Functions) → Output
```

Built in Rust for:
- Memory safety and performance
- Strong type system
- Fast parsing and evaluation
- Easy embedding in other tools
- Cross-platform support

## Project Status

JCL v1.0.0 is production-ready! ✅

- **144 tests passing** (117 unit + 18 CLI + 9 integration)
- **Zero compiler warnings**
- **Complete documentation** with interactive examples
- **Multi-language bindings** for Python, Node.js, Go, Java, Ruby
- **Full LSP support** for modern editors
- **CI/CD pipeline** with automated testing and releases

See [CHANGELOG.md](CHANGELOG.md) for version history.

## Why JCL?

**vs. HCL (HashiCorp Configuration Language):**
- More human-readable syntax (less verbose, no braces)
- Richer built-in function library (70+ functions including higher-order functions)
- Advanced static type inference catches errors before runtime
- Runtime type validation with annotations
- Cleaner string interpolation
- Built-in code formatter and linter

**vs. YAML:**
- Type-safe with validation
- Powerful built-in functions
- String interpolation and templates
- Better error messages

**vs. JSON:**
- Human-readable (comments, no quotes required)
- Computed values and expressions
- Functions and data transformation
- Variables and references

**vs. Full Programming Languages (Python, TypeScript):**
- Purpose-built for configuration
- Simpler and more constrained
- Easier to learn and audit
- Can't execute arbitrary code (safer)

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for:
- Code of conduct
- Development setup
- Testing requirements
- Pull request process
- Coding standards

For bugs, feature requests, or questions:
- **Bug Reports**: Use our [bug report template](.github/ISSUE_TEMPLATE/bug_report.md)
- **Feature Requests**: Use our [feature request template](.github/ISSUE_TEMPLATE/feature_request.md)
- **Documentation**: Use our [documentation template](.github/ISSUE_TEMPLATE/documentation.md)

## Security

Found a security vulnerability? Please see [SECURITY.md](SECURITY.md) for responsible disclosure guidelines.

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Community

- **Documentation**: [https://hemmer-io.github.io/jcl/](https://hemmer-io.github.io/jcl/)
- **Repository**: [https://github.com/hemmer-io/jcl](https://github.com/hemmer-io/jcl)
- **Issues**: [https://github.com/hemmer-io/jcl/issues](https://github.com/hemmer-io/jcl/issues)

Built with ❤️ in Rust
