# JCL - Jack-of-All Configuration Language

A modern, safe, and flexible general-purpose configuration language with powerful built-in functions, written in Rust.

## Vision

JCL is a general-purpose configuration language designed to be human-readable, type-safe, and powerful. It provides a rich standard library of functions for data manipulation, encoding/decoding (YAML, JSON, Base64), templating, string operations, and more. Built in Rust for performance and safety, JCL can be embedded in other tools (like Hemmer for IaC) or used standalone for configuration management.

## Status

**Phase 4 Complete! 🚀**

JCL v1.0 is production-ready with comprehensive tooling, multi-language bindings, and complete documentation:

**Phase 1 (Complete):**
- ✅ Complete Pratt parser with proper operator precedence
- ✅ Full expression evaluator (arithmetic, logical, comparison, null-safety)
- ✅ String interpolation with `${...}` syntax
- ✅ List comprehensions, pipelines, pattern matching
- ✅ Lambda functions and user-defined functions
- ✅ 56+ built-in functions (string, encoding, collections, numeric, hashing, time)
- ✅ Interactive REPL with history and state management
- ✅ Comprehensive error messages with context and hints

**Phase 2 (Complete):**
- ✅ Higher-order functions: `map()`, `filter()`, `reduce()` with lambda support
- ✅ Runtime type validation with annotations
- ✅ Advanced static type inference with expression-level checking
- ✅ Code formatter (`jcl fmt`) with style rules
- ✅ Template rendering: `template()` and `templatefile()` with Handlebars
- ✅ Lambda variable calls (call lambdas stored in variables)
- ✅ Language Server Protocol (LSP) with diagnostics and autocomplete
- ✅ Comprehensive linter with 9 lint rules
- ✅ VSCode and Vim/Neovim syntax highlighting
- ✅ Documentation generator from function definitions
- ✅ WebAssembly compilation and online playground
- ✅ C Foreign Function Interface (FFI) for embedding
- ✅ 117 unit tests + 9 integration tests (100% passing)
- ✅ Zero compiler warnings

**Phase 3 (Complete):**
- ✅ Printf-style `format()` function with full specifier support
- ✅ Expression-level try/catch error handling
- ✅ Schema validation tool (`jcl-validate`)
- ✅ Format migration tool (`jcl-migrate`) for JSON/YAML/TOML
- ✅ Auto-format watcher (`jcl-watch`)
- ✅ Performance benchmarking tool (`jcl-bench`)
- ✅ LSP: Go to Definition with symbol table
- ✅ LSP: Find References
- ✅ LSP: Rename Symbol
- ✅ LSP: Position-aware diagnostics with line/column precision
- ✅ Multi-language bindings: Python, Node.js, Go, Java, Ruby

**Phase 4 (Complete):**
- ✅ Comprehensive documentation site with Jekyll
- ✅ Getting started guide with tutorials
- ✅ Complete CLI tools reference
- ✅ 70+ built-in functions documented
- ✅ Comparison guide (vs JSON/YAML/TOML/HCL)
- ✅ GitHub Pages deployment workflow

**Next:** Testing (integration tests for CLI, LSP, and language bindings) and publishing to package registries (crates.io, PyPI, npm).

## Key Features

🎯 **General-Purpose Configuration**
- Clean, human-readable syntax with minimal punctuation
- Rich standard library of 56+ built-in functions
- Can be embedded or used standalone

🔒 **Safety First**
- Advanced static type inference catches errors before runtime
- Strong type system with expression-level checking
- Immutability by default
- Validation at every stage
- Dry-run and plan before apply

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

```
# Simple, readable configuration
environments = (prod, dev, staging)

env.prod = (
  region = us-west-2

  vars (
    app_name = myapp
    version = 1.2.3
    replicas = 3
  )

  tags (
    team = platform
    cost_center = engineering
  )
)

# String interpolation
greeting = "Hello, ${env.prod.vars.app_name}!"

# Built-in functions
uppercased = upper(env.prod.vars.app_name)
config_json = jsonencode(env.prod.vars)
config_yaml = yamlencode(env.prod.vars)

# Collections and data manipulation
regions = (us-west-2, us-east-1, eu-west-1)
region_count = length(regions)
merged_tags = merge(env.prod.tags, (environment=prod managed_by=jcl))

# List comprehensions and pipelines
formatted_regions = regions
  | map r => upper(r)
  | sort
  | join ", "

# Template rendering
nginx_config = templatefile(nginx.conf.tpl, (
  port = 8080
  server_name = env.prod.vars.app_name
))
```

## Project Status

🎉 **Phase 1 Complete - Fully Functional!** 🎉

JCL v1.0 is now fully implemented and tested! All core features are working:
- ✅ Language syntax and grammar (see [LANGUAGE_SPEC.md](docs/LANGUAGE_SPEC.md))
- ✅ Collection syntax: `[]` for lists, `()` for maps
- ✅ String interpolation with `${...}`
- ✅ Null safety operators: `?.` and `??`
- ✅ Control flow: ternary, if/then/else, when expressions
- ✅ Functions: lambda (`x => x * 2`) and named (`fn name(x) = ...`)
- ✅ For loops and list comprehensions
- ✅ Import system
- ✅ Error handling with `try()` and fail-fast
- ✅ Templating patterns (see [TEMPLATING.md](docs/TEMPLATING.md))
- ✅ Interactive REPL with history
- ✅ Comprehensive error messages

Next step: Phase 2 - Higher-order functions, advanced type checking, LSP, and tooling ecosystem.

## Integration

JCL is designed as a general-purpose configuration language that can be embedded in other tools:

- **JCL**: The configuration language with syntax, parser, type system, and built-in functions
- **Embedding**: Use C FFI, WebAssembly, or future Python bindings to integrate JCL into your tools

This separation allows JCL to be a versatile configuration language that can be embedded in multiple applications.

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

## Roadmap

**Phase 1 - Core Language (Complete):**
- [x] Language specification and grammar (Pest PEG parser)
- [x] Built-in functions library (56+ functions)
- [x] Templating patterns documentation
- [x] Parser implementation (Pratt parser for expressions)
- [x] Expression evaluator with all operators
- [x] String interpolation engine
- [x] Basic type system with inference
- [x] Parser with error recovery and comprehensive error messages
- [x] REPL for interactive testing

**Phase 2 - Tooling & Integration (Complete):**
- [x] Higher-order functions (map, filter, reduce) with lambda support
- [x] Runtime type validation during evaluation
- [x] Advanced static type inference with expression-level checking
- [x] Template rendering (template, templatefile) with Handlebars
- [x] Code formatter with style rules (jcl fmt)
- [x] Language Server Protocol (LSP) with diagnostics and autocomplete
- [x] Comprehensive linter with 9 lint rules
- [x] Syntax highlighting (VSCode extension)
- [x] Vim/Neovim syntax files
- [x] Documentation generator from function definitions
- [x] C FFI for embedding in other languages
- [x] WebAssembly compilation
- [x] Online playground with WASM

**Phase 3 - Advanced Features (Complete):**
- [x] Printf-style `format()` function implementation
- [x] Expression-level try/catch error handling
- [x] LSP: Go to Definition
- [x] LSP: Find References
- [x] LSP: Rename Symbol
- [x] LSP: Position-aware diagnostics (line/column precision)
- [x] Advanced static type inference
- [x] Python bindings (PyO3)
- [x] Node.js bindings
- [x] Go bindings
- [x] Java bindings
- [x] Ruby bindings

**Phase 4 - Documentation & GitHub Pages (Complete):**
- [x] Jekyll-based documentation site setup
- [x] Comprehensive getting started guide
- [x] Complete CLI tools reference (7 tools documented)
- [x] Built-in functions reference (70+ functions)
- [x] Comparison guide (JCL vs JSON/YAML/TOML/HCL)
- [x] GitHub Actions workflow for automatic deployment

## Why JCL?

**vs. HCL (HashiCorp Configuration Language):**
- More human-readable syntax (less verbose, no braces)
- Richer built-in function library (56+ functions including higher-order functions)
- Runtime type validation with annotations
- Better type inference
- Cleaner string interpolation
- Built-in code formatter

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

This project is in early stages. We welcome:
- Feedback on the design
- Syntax suggestions
- Use case examples
- Architecture discussions

Please see [DESIGN.md](./DESIGN.md) for the current design thinking.

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Contact

Project is in early development. More information coming soon!
