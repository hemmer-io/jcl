# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Module System - Phase 1** (#95)
  - **Module interfaces**: Declare module inputs and outputs with type contracts
  - **Module instantiation**: `module.<type>.<instance> = (source = "...", inputs...)`
  - **Module inputs access**: Special `module.inputs` variable within modules
  - **Module outputs**: Define outputs with `module.outputs = (...)`
  - **Input validation**: Required field checking, unknown field detection
  - **Output validation**: Ensure all declared outputs are provided
  - **Variable isolation**: Module evaluations don't pollute parent scope
  - **Multiple instances**: Same module can be instantiated multiple times with different inputs
  - **Nested access**: `module.<type>.<instance>.<output>` for accessing outputs
  - Syntax:
    ```jcl
    # Module definition
    module.interface = (
        inputs = (name = (type = string, required = true)),
        outputs = (result = (type = string))
    )
    module.outputs = (result = "Hello, ${module.inputs.name}!")

    # Module usage
    module.greeter.alice = (source = "./greeter.jcl", name = "Alice")
    message = module.greeter.alice.result  # "Hello, Alice!"
    ```
- **Schema Generation from Examples** (#102)
  - `jcl-schema-gen` CLI tool to generate schemas from example JCL files
  - Automatic type inference from values (String, Number, Boolean, List, Map)
  - Pattern detection for common formats (email, URL, file paths)
  - Constraint inference (min/max lengths, ranges)
  - Required field detection (fields present in all examples)
  - Supports multiple input files for comprehensive schema generation
  - Multiple output formats (JSON Schema, YAML, OpenAPI)
  - Programmatic API: `generate_from_examples()` function
  - Configurable generation options (infer types, infer constraints, all optional)
- **LSP Schema Validation Integration** (#101)
  - Real-time schema validation in editors (VSCode, Vim, etc.)
  - Automatic schema discovery from workspace (`.jcl-schema.json`, `.jcl-schema.yaml`)
  - Schema validation diagnostics alongside linting errors
  - Detailed error messages with suggestions in editor
  - Hot-reloading of schema files on workspace initialization
- **Enhanced Import System**: Complete multi-file module system with two import patterns (#94)
  - **Path-based imports**: `import "./config.jcl"` or `import "./config.jcl" as alias`
  - **Selective imports**: `import (item1, item2) from "./path.jcl"` with per-item aliasing
  - **Wildcard imports**: `import * from "./path.jcl"`
  - **Path resolution**: Relative to importing file (not cwd)
  - **Circular dependency detection**: Automatic cycle prevention with clear error messages
  - **Import caching**: Parse and evaluate each module only once for performance
  - **Nested imports**: Full support for import chains with proper context tracking
- **Import Debugging & Analysis Tools** (#94)
  - **Import tracing**: Real-time logging of import activity (`enable_import_tracing()`)
  - **Performance metrics**: Track cache hits, timing, and efficiency
  - **Import graph visualization**: Generate DOT format graphs for GraphViz
  - **Detailed trace reports**: Human-readable import chain analysis
- **Import Documentation**: Comprehensive guides and examples in language spec and getting started
- **Enhanced Schema Validation API - Phases 1-5 (Complete)** (#93)
  - **Phase 1: Better Error Messages & Builder Pattern**
    - Rich error types with suggestions and precise source locations
    - Fluent `SchemaBuilder` and `PropertyBuilder` APIs
    - Type-safe schema construction with minimal boilerplate
  - **Phase 2: Custom Validators & Conditional Rules**
    - Thread-safe custom validator functions (`ValidatorFn`)
    - Field dependency tracking (`requires()`, `requires_absence_of()`)
    - Mutually exclusive field groups
  - **Phase 3: Complex Types**
    - Discriminated unions (tagged unions) with variant validation
    - Recursive type references via `TypeDef::Ref`
    - Explicit type system (no surprising coercions per JCL design)
  - **Phase 4: Schema Composition**
    - Schema versioning with `version()` method
    - Schema inheritance via `extends()` with property merging
    - Schema composition via `merge()` for combining multiple schemas
    - Markdown documentation generation with `generate_docs()`
    - Round-trip support via `SchemaBuilder::from_schema()`
  - **Phase 5: Schema Export**
    - JSON Schema Draft 7 export via `to_json_schema()`
    - OpenAPI 3.0 schema export via `to_openapi()`
    - Full TypeDef coverage including unions, discriminated unions, refs
    - Standard-compliant output for integration with external tools

## [1.1.0] - 2025-01-19

### Added
- **Performance Benchmarks**: Comprehensive suite demonstrating 50-100x speedup for library bindings (#50, #78)
- **Test Build Workflow**: Automated package build testing without publishing (#66, #72)
- **Pre-commit Hooks**: Automated code quality checks (#53, #54)
- **Dependabot**: Intelligent dependency grouping and auto-merge (#55, #56)
- **Enhanced AST Caching**: Increased capacity, metrics, CLI stats command (#51, #61)
- **Lazy Variables (Phase 2)**: On-demand evaluation with cycle detection (#5, #43)
- **List Comprehension Optimization (Phase 3A)**: Index-based early termination (#44, #47)
- **Parallel Parsing**: Multi-core file parsing with Rayon (#6, #36)
- **Multi-file Validation**: Directory and glob pattern support (#37, #38)
- **CODEOWNERS**: Repository maintainer definitions (#48, #52)

### Changed
- **Documentation**: Restructured to promote library bindings (#49, #73)
- **WASM Playground**: Fixed deployment (#69, #70, #71)
- **CI/CD**: Workflows run only on PR creation (#39, #41)
- **Examples**: Fixed invalid JCL syntax (#40, #42)

### Fixed
- **Dependabot Auto-merge**: Use PAT token for approvals (#67, #68)
- **Test Build Workflow**: Fixed cache action and gem install (#74-#77)
- **.gitignore**: Added bindings build artifact patterns (#78)
- **PyO3 0.27.1**: Updated Python bindings API (#58, #62)
- **Magnus 0.8.2**: Fixed Ruby deprecation warnings (#58, #63)

### Dependencies
- Multiple Dependabot updates including PyO3, Magnus, criterion, and GitHub Actions

## [1.0.0] - 2025-01-18

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
