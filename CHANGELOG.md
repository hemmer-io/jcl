# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Advanced static type inference system with expression-level checking
- LSP: Position-aware diagnostics with precise line/column information
- GitHub Actions CI/CD workflows for testing, linting, and multi-platform builds
- Automated release workflow with pre-built binaries
- CONTRIBUTING.md with contribution guidelines
- CODE_OF_CONDUCT.md with community standards
- SECURITY.md with security policy and reporting
- TODO.md tracking remaining work
- 13 new test cases for advanced type inference (117 total tests)

### Changed
- Type system now performs static analysis before evaluation
- Type errors include precise source location spans
- Improved type compatibility checking for arithmetic operations
- Enhanced function type checking with parameter validation

## [1.0.0] - 2025-11-18

### Added

#### Core Language (Phase 1)
- Complete Pratt parser with proper operator precedence
- Full expression evaluator (arithmetic, logical, comparison, null-safety)
- String interpolation with `${...}` syntax
- List comprehensions and for loops
- Lambda functions (`x => x * 2`) and user-defined functions (`fn name(x) = ...`)
- 70+ built-in functions across 12 categories
- Interactive REPL with history and state management
- Comprehensive error messages with context and hints

#### Tooling & Integration (Phase 2)
- Higher-order functions: `map()`, `filter()`, `reduce()` with lambda support
- Runtime type validation with annotations
- Advanced static type inference with expression-level checking
- Code formatter (`jcl fmt`) with style rules
- Template rendering: `template()` and `templatefile()` with Handlebars
- Language Server Protocol (LSP) with diagnostics and autocomplete
- Comprehensive linter with 9 lint rules
- VSCode and Vim/Neovim syntax highlighting
- Documentation generator from function definitions
- C Foreign Function Interface (FFI) for embedding
- WebAssembly compilation and online playground

#### Advanced Features (Phase 3)
- Printf-style `format()` function with full specifier support (%s, %d, %f, %b, %v, %x, %X, %o)
- Expression-level try/catch error handling
- Schema validation tool (`jcl-validate`)
- Format migration tool (`jcl-migrate`) for JSON/YAML/TOML
- Auto-format watcher (`jcl-watch`)
- Performance benchmarking tool (`jcl-bench`)
- LSP: Position-aware diagnostics with line/column precision
- LSP: Go to Definition with symbol table
- LSP: Find References
- LSP: Rename Symbol
- Multi-language bindings: Python (PyO3), Node.js, Go, Java, Ruby

#### Documentation (Phase 4)
- Comprehensive documentation site with Jekyll
- Getting started guide with tutorials
- Complete CLI tools reference (7 tools documented)
- Built-in functions reference (70+ functions)
- Comparison guide (vs JSON/YAML/TOML/HCL)
- GitHub Pages deployment workflow

### Changed
- Refocused project as general-purpose configuration language
- Removed infrastructure-specific features (providers, state management, planners)
- Updated README to reflect current scope and status

### Fixed
- Parser keyword boundary checking
- Multi-parameter lambda support
- Token parser improvements

## [0.1.0] - 2025-11-15

### Added
- Initial release
- Basic parser and evaluator
- Core built-in functions
- REPL implementation

---

## Version History Summary

- **1.0.0** (2025-11-18): Production-ready release with complete tooling, multi-language bindings, and documentation
- **0.1.0** (2025-11-15): Initial experimental release

[Unreleased]: https://github.com/hemmer-io/jcl/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/hemmer-io/jcl/releases/tag/v1.0.0
[0.1.0]: https://github.com/hemmer-io/jcl/releases/tag/v0.1.0
