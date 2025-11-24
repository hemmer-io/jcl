# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.2.0] - 2025-11-24

### Added
- **Cross-Compilation Support** (#117, #122)
  - Pre-built binaries for 6 target platforms
  - Native builds: Linux x86_64, macOS ARM64, macOS x86_64 (Intel), Windows x86_64
  - Cross-compiled builds: Linux ARM64 (Raspberry Pi, AWS Graviton), Linux MUSL (Alpine, static linking)
  - Automated release workflow uploads all binaries on tagged releases
  - Build artifacts uploaded for every PR for testing

- **Heredoc String Syntax** (#85, #96)
  - Multi-line strings with `<<EOF ... EOF` syntax
  - Preserves formatting and whitespace
  - Supports custom delimiters
  - Example:
    ```jcl
    message = <<EOF
    Hello, World!
    This is a multi-line string.
    EOF
    ```

- **Splat Operator** (#86, #92)
  - Extract attributes from lists of maps with `[*]` syntax
  - `users[*].name` extracts all `name` fields from a list of user maps
  - Works with nested access: `data[*].items[*].value`

- **Range Syntax** (#89, #91)
  - Generate number sequences with `[start..end]` syntax
  - Inclusive ranges: `[1..5]` produces `[1, 2, 3, 4, 5]`
  - Step support: `[0..10..2]` produces `[0, 2, 4, 6, 8, 10]`
  - Reverse ranges: `[5..1]` produces `[5, 4, 3, 2, 1]`

- **Slice Syntax & Nested List Comprehensions** (#46, #84)
  - Python-style slicing: `list[start:end]`, `list[:end]`, `list[start:]`
  - Negative indices: `list[-1]` for last element
  - Nested comprehensions: `[[x * y for y in row] for x in matrix]`

- **New Built-in Functions** (#87, #90)
  - `indent(str, spaces)`: Indent each line of a string
  - `chomp(str)`: Remove trailing newlines
  - `title(str)`: Title case conversion
  - `compact(list)`: Remove null values from list
  - `chunklist(list, size)`: Split list into chunks
  - `coalesce(...)`: Return first non-null value
  - `try(expr, default)`: Return default on error
  - `range(start, end, step?)`: Generate number sequences
  - `element(list, index)`: Safe list access with wraparound
  - `index(list, value)`: Find index of value in list
  - `zipmap(keys, values)`: Create map from key/value lists
  - `setproduct(...)`: Cartesian product of lists
  - `setunion(...)`: Union of multiple lists
  - `setintersection(...)`: Intersection of lists
  - `setsubtract(a, b)`: Set difference

- **Let-Bindings for Local Variable Scoping** (#109, #110)
  - Local variable declarations with `let name = value in expr`
  - Proper lexical scoping
  - Shadowing support
  - Example:
    ```jcl
    result = let x = 10 in let y = 20 in x + y  # 30
    ```

- **LSP Schema Validation Enhancements** (#104, #111)
  - Precise error positioning with exact line/column locations
  - File watching for schema changes with automatic revalidation
  - Hover information for schema-validated fields
  - Completion suggestions based on schema definitions
  - Multi-file schema support

- **GitHub Linguist Support Files** (#112, #113)
  - TextMate grammar for syntax highlighting (`syntaxes/jcl.tmLanguage.json`)
  - Sample files for Linguist testing
  - Prepared for GitHub Linguist submission (pending separate grammar repo)

- **Streaming API & Transparent Lazy Evaluation** (#45)
  - **Higher-Order Functions**: `map`, `filter`, and `reduce` now work polymorphically with both lists (eager) and streams (lazy)
  - **Streaming Functions**: New `stream()`, `take()`, and `collect()` functions for explicit lazy evaluation
  - **Transparent Optimization**: Automatic lazy evaluation for `[expr for x in list][start:end]` pattern
    - Only processes elements actually needed for the slice
    - Memory: O(k) instead of O(n) where k = slice size
    - Speed: 10x-100x faster for small slices from large lists
    - Works with filters: `[expr for x in list if cond][start:end]`
  - **Pattern Detection**: Automatically detects and optimizes:
    - `[expr for x in list][0:10]` - bounded slice
    - `[expr for x in list][:10]` - from start
    - `[expr for x in list][5:]` - to end
    - `[expr for x in list if cond][0:10]` - with filter
  - **Backwards Compatible**: All existing code continues to work unchanged
  - **Example**:
    ```jcl
    # Automatically optimized! Only processes 10 elements, not 1000.
    result = [x * 2 for x in [0..1000]][0:10]
    # [0, 2, 4, 6, 8, 10, 12, 14, 16, 18]
    ```
  - **Test Coverage**: Added 20 comprehensive tests (297 total tests passing)

### Changed
- **File Extension Migration** (#106)
  - Migrated file extension from `.jcl` to `.jcf` (JCL Configuration Format)
  - Avoids conflict with IBM mainframe Job Control Language in GitHub Linguist
  - All 27 example/test files renamed
  - Updated all documentation and code references
  - VSCode extensions updated to recognize `.jcf` files
  - All CLI tools (jcl-validate, jcl-watch) now process `.jcf` files

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
    module.greeter.alice = (source = "./greeter.jcf", name = "Alice")
    message = module.greeter.alice.result  # "Hello, Alice!"
    ```
- **Module System - Phase 2: Module Composition** (#95)
  - **Nested module calls**: Modules can instantiate other modules
  - **Default value support**: Module inputs can have default values that are automatically applied
  - **Circular dependency detection**: Detects and prevents circular module dependencies with clear error messages
  - **Multi-level hierarchies**: Support for complex 3+ level module call chains
  - **Module context preservation**: Proper isolation of `module.inputs` context for nested modules
  - Example of nested modules:
    ```jcl
    # wrapper.jcf uses base.jcf
    module.interface = (
        inputs = (person = (type = string, required = true),
                  prefix = (type = string, required = false, default = "Welcome")),
        outputs = (full_message = (type = string))
    )
    module.base.instance1 = (source = "./base.jcf", name = module.inputs.person)
    module.outputs = (full_message = "${module.inputs.prefix}: ${module.base.instance1.message}")

    # main.jcf uses wrapper.jcf (which uses base.jcf)
    module.wrapper.alice = (source = "./wrapper.jcf", person = "Alice", prefix = "Greetings")
    result = module.wrapper.alice.full_message  # "Greetings: Hello, Alice!"
    ```
- **Module System - Phase 3: Advanced Features** (#95)
  - **Module metadata**: Declare module version, description, author, and license
    ```jcl
    module.metadata = (
        version = "1.0.0",
        description = "A simple greeting module",
        author = "JCL Team",
        license = "MIT"
    )
    ```
  - **Conditional module instantiation**: Use `condition` to conditionally create module instances
    ```jcl
    module.service.web = (
        source = "./service.jcf",
        condition = environment == "production",
        name = "web-server"
    )
    ```
  - **Count meta-argument**: Create N identical module instances stored as a list
    ```jcl
    module.server.cluster = (
        source = "./server.jcf",
        count = 3,
        name = "server-${count.index}"  # count.index available during evaluation
    )
    # Access: module.server.cluster = [instance0, instance1, instance2]
    ```
  - **For_each meta-argument**: Create module instances for each element in a list or map
    ```jcl
    # With list
    module.server.named = (
        source = "./server.jcf",
        for_each = ["web", "api", "db"],
        name = each.value  # each.key = index, each.value = element
    )
    # With map
    module.server.envs = (
        source = "./server.jcf",
        for_each = (dev = "dev-server", prod = "prod-server"),
        name = each.value  # each.key = map key, each.value = map value
    )
    # Access: module.server.envs = (dev = {...}, prod = {...})
    ```
  - **Module output aggregation helpers**: Built-in functions for extracting outputs
    - `module_outputs(list, "field")`: Extract field from list of module instances
    - `module_outputs_map(map, "field")`: Extract field from map of module instances
    - `module_all_outputs(list)`: Get all outputs from list of module instances
    ```jcl
    hostnames = module_outputs(module.server.cluster, "hostname")
    # hostnames = ["server-0", "server-1", "server-2"]
    ```
- **Module System - Phase 4: External Sources** (#95)
  - **Module source resolution API**: Abstraction for loading modules from various sources
  - **Git repository sources**: Clone and use modules from Git repositories
    ```jcl
    module.external.example = (
        source = "git::https://github.com/user/repo.git//modules/example.jcf?ref=v1.0.0",
        config_value = "production"
    )
    ```
  - **HTTP/HTTPS sources**: Download modules from web URLs
    ```jcl
    module.remote.config = (
        source = "https://example.com/modules/config.jcf",
        api_key = secrets.api_key
    )
    ```
  - **Tarball sources**: Extract and use modules from compressed archives
    ```jcl
    module.archived.legacy = (
        source = "https://example.com/modules/legacy.tar.gz//legacy/module.jcf",
        compatibility_mode = true
    )
    ```
  - **Module caching**: Automatic local caching of downloaded modules
    - Cache directory: `~/.cache/jcl/modules/` (configurable)
    - Cache key based on URL hash (MD5)
    - Git repos cached and updated with `git fetch`
    - HTTP/tarball sources downloaded once and reused
  - **Version resolution**: Support for Git refs (tags, branches, commits)
    - Specify version with `?ref=` query parameter
    - Example: `?ref=v1.2.3`, `?ref=main`, `?ref=abc123`
  - **Lock file format**: `.jcf.lock` for reproducible builds
    - JSON format with resolved URLs and checksums
    - Tracks exact versions of external modules
    - Ensures consistent builds across environments
- **Module System - Phase 5: Module Registry** (#95)
  - **Registry protocol**: RESTful API for module discovery, publishing, and downloads
    - GET `/api/v1/modules/search?q=<query>` - Search modules
    - GET `/api/v1/modules/<name>` - Get module metadata
    - GET `/api/v1/modules/<name>/versions/<version>` - Get specific version
    - POST `/api/v1/modules/publish` - Publish module (requires auth)
  - **Registry client**: Full-featured client for interacting with registries
    ```rust
    use jcl::module_registry::RegistryClient;

    let client = RegistryClient::default_registry();
    let results = client.search("aws", 10)?;
    let module = client.get_module("aws-ec2")?;
    ```
  - **Registry module sources**: Use modules from registry with semantic versioning
    ```jcl
    module.compute.ec2 = (
        source = "registry::aws-ec2@^1.0.0",
        region = "us-east-1"
    )

    module.database.rds = (
        source = "registry::aws-rds",  # Latest version
        engine = "postgres"
    )
    ```
  - **Semantic versioning**: Full semver support with version requirements
    - Caret requirements: `^1.2.3` (compatible with 1.x.x)
    - Tilde requirements: `~1.2.3` (compatible with 1.2.x)
    - Exact requirements: `=1.2.3`
    - Wildcard: `*` (latest version)
    - Version resolution finds highest matching version
  - **Module publishing workflow**: Publish modules to registry
    - Module manifest (`jcl.json`) with name, version, dependencies
    - Automatic tarball creation and checksum generation
    - Authentication via bearer tokens
    - Example: `client.publish("./my-module")`
  - **Module discovery and search**: Find modules in registry
    - Search by name, keywords, description
    - Pagination support
    - Download counts and popularity
  - **Module manifests** (`jcl.json`):
    ```json
    {
      "name": "aws-ec2",
      "version": "1.2.3",
      "description": "AWS EC2 instance configuration",
      "author": "JCL Community",
      "license": "MIT",
      "repository": "https://github.com/jcl-modules/aws-ec2",
      "keywords": ["aws", "ec2", "compute"],
      "dependencies": {
        "aws-base": "^2.0.0"
      },
      "main": "module.jcf"
    }
    ```
  - **Dependency resolution**: Automatically download and resolve dependencies
  - **Default registry**: `https://registry.jcf.io` (configurable)
  - **Multi-registry support**: Configure multiple registries (public, private, company-internal)
- **Module System - Phase 6: Tooling** (#95)
  - **CLI Commands**: `jcl-module` binary for module management
    - `jcl-module init <name>`: Scaffold a new module with manifest, interface template, README, and .gitignore
    - `jcl-module validate <path>`: Validate module structure, manifest, and interface
    - `jcl-module get <path>`: Download module dependencies from registry
    - `jcl-module list`: List installed modules with version information
  - **LSP Enhancements**: Language server support for module development
    - Symbol tracking for module instances in symbol table
    - Module instance metadata (source path) for navigation
  - **Module Documentation Generation**: Generate markdown docs from module interfaces
    - Extract module metadata (version, description, author, license)
    - Document module inputs with types, required status, defaults, and descriptions
    - Document module outputs with types and descriptions
    - Integrated with existing `docgen` module
    - Example:
      ```jcl
      module.metadata = (
          version = "1.0.0",
          description = "A simple greeting module",
          author = "JCL Team",
          license = "MIT"
      )
      module.interface = (
          inputs = (
              name = (type = string, required = true, description = "Person's name"),
              prefix = (type = string, required = false, default = "Hello", description = "Greeting prefix")
          ),
          outputs = (
              message = (type = string, description = "The greeting message")
          )
      )
      # Generated docs include all inputs, outputs, and metadata in markdown format
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
  - Automatic schema discovery from workspace (`.jcf-schema.json`, `.jcf-schema.yaml`)
  - Schema validation diagnostics alongside linting errors
  - Detailed error messages with suggestions in editor
  - Hot-reloading of schema files on workspace initialization
- **Enhanced Import System**: Complete multi-file module system with two import patterns (#94)
  - **Path-based imports**: `import "./config.jcf"` or `import "./config.jcf" as alias`
  - **Selective imports**: `import (item1, item2) from "./path.jcf"` with per-item aliasing
  - **Wildcard imports**: `import * from "./path.jcf"`
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

### Changed
- **File Extension**: Migrated from `.jcl` to `.jcf` (JCL Configuration Format) (#106)
  - Avoids conflict with IBM mainframe Job Control Language in GitHub Linguist
  - All example/test files renamed
  - Updated all documentation and code references

### Fixed
- **Python Bindings**: Handle `Value::Stream` variant in `value_to_python()` (#119)
- **Dependabot Auto-merge**: Allow skipped checks in workflow (#121)
- **Dependabot Auto-merge**: Use DEPENDABOT_PAT consistently (#118)
- **Parser Bugs**: Fixed critical issues including keyword boundary checking (#109)

### Dependencies
- Bumped `pest` from 2.8.3 to 2.8.4 (#115)
- Bumped `pest_derive` from 2.8.3 to 2.8.4 (#115)

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

- **1.2.0** (2025-11-24): Major feature release with cross-compilation, heredocs, splat operator, range syntax, and streaming API
- **1.1.0** (2025-11-19): Performance improvements with lazy evaluation and parallel parsing
- **1.0.0** (2025-11-18): Production-ready release with complete tooling, multi-language bindings, and documentation
- **0.1.0** (2025-11-15): Initial experimental release

[1.2.0]: https://github.com/hemmer-io/jcl/compare/v1.1.0...v1.2.0
[1.1.0]: https://github.com/hemmer-io/jcl/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/hemmer-io/jcl/releases/tag/v1.0.0
[0.1.0]: https://github.com/hemmer-io/jcl/releases/tag/v0.1.0
