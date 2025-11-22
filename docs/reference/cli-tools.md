---
layout: default
title: CLI Tools Reference
parent: Reference
nav_order: 3
permalink: /reference/cli-tools/
---


JCL provides a comprehensive suite of command-line tools for working with JCL configurations.

## jcl - Main CLI

The primary JCL command-line interface.

### Commands

#### eval

Evaluate a JCL file and print the results.

```bash
jcl eval <file>
```

**Example:**
```bash
$ jcl eval config.jcf
name: "my-app"
version: "1.0.0"
port: 8080
database: {host: "localhost", port: 5432}
```

#### repl

Start an interactive REPL (Read-Eval-Print Loop).

```bash
jcl repl
```

**REPL Commands:**
- `:help`, `:h` - Show help
- `:quit`, `:q`, `:exit` - Exit REPL
- `:clear`, `:c` - Clear all variables
- `:vars`, `:v` - Show all variables

**Features:**
- Persistent history (`~/.jcf_history`)
- Multi-line input (use `\` at end of line)
- Tab completion
- History search (Ctrl-R)

**Example:**
```bash
$ jcl repl
JCL REPL v0.1.0

jcl:1 x = 42
✓
jcl:2 x * 2
84
jcl:3 :quit
Goodbye!
```

---

## jcl-validate - Schema Validation

Validate JCL configuration files against schemas.

### Usage

```bash
jcl-validate <config> --schema <schema>
```

### Options

- `-s, --schema <SCHEMA>` - Schema file (JSON or YAML)
- `-f, --schema-format <FORMAT>` - Schema format (json, yaml) - auto-detects if not specified
- `-v, --verbose` - Verbose output
- `--no-fail` - Exit with status 0 even if validation fails

### Examples

**Basic validation:**
```bash
$ jcl-validate config.jcf --schema schema.yaml
✅ Validation passed!
```

**With errors:**
```bash
$ jcl-validate invalid.jcf --schema schema.yaml
❌ 3 validation error(s) found:

  • version
    String '1.2' does not match pattern '^\d+\.\d+\.\d+$'

  • port
    Number 99999 exceeds maximum 65535

  • database.port
    Required property 'port' is missing
```

**Verbose mode:**
```bash
$ jcl-validate config.jcf --schema schema.yaml --verbose
📋 Schema loaded from schema.yaml
📄 Configuration loaded from config.jcf
🔍 Validating...

✅ Validation passed!
```

### Schema Format

Schemas can be defined in JSON or YAML:

```yaml
version: "1.0"
title: "Application Configuration Schema"
parent: Reference
nav_order: 3
description: "Schema for validating app configs"

type:
  kind: map
  required:
    - name
    - version
    - port
  properties:
    name:
      type:
        kind: string
        min_length: 1
        max_length: 50

    port:
      type:
        kind: number
        minimum: 1
        maximum: 65535
        integer_only: true
```

---

## jcl-migrate - Format Migration

Convert configuration files from JSON, YAML, or TOML to JCL.

### Usage

```bash
jcl-migrate <input> [options]
```

### Options

- `-o, --output <OUTPUT>` - Output file (prints to stdout if not specified)
- `-f, --from <FORMAT>` - Input format (json, yaml, toml) - auto-detects if not specified
- `-v, --verbose` - Verbose output

### Examples

**Convert JSON to JCL:**
```bash
$ jcl-migrate config.json
name = "my-app"
version = "1.0.0"
port = 8080
database = (
    host = "localhost",
    port = 5432
)
```

**Save to file:**
```bash
$ jcl-migrate config.json -o config.jcf
$ cat config.jcf
name = "my-app"
version = "1.0.0"
...
```

**Convert YAML:**
```bash
$ jcl-migrate config.yaml --verbose
📄 Converting from Yaml to JCL...
name = "my-app"
...
```

**Convert TOML:**
```bash
$ jcl-migrate config.toml
name = "my-app"
enabled = true
...
```

---

## jcl-fmt - Code Formatter

Format JCL files according to standard style.

### Usage

```bash
jcl-fmt <files>... [options]
```

### Options

- `-c, --check` - Check only (don't modify files)
- `-v, --verbose` - Verbose output

### Examples

**Format a single file:**
```bash
$ jcl-fmt config.jcf
✓ config.jcf - Formatted

✅ Formatted 1 file(s)
```

**Format multiple files:**
```bash
$ jcl-fmt *.jcf
✓ config.jcf - Formatted
✓ app.jcf - Already formatted
✓ db.jcf - Formatted

✅ Formatted 3 file(s)
```

**Check formatting (CI mode):**
```bash
$ jcl-fmt --check config.jcf
✅ All files are properly formatted!

$ jcl-fmt --check *.jcf
! config.jcf - Needs formatting
! app.jcf - Needs formatting

⚠️ 2 file(s) need formatting:
  - config.jcf
  - app.jcf
```

**Verbose mode:**
```bash
$ jcl-fmt config.jcf --verbose
📝 Processing: config.jcf
✓ config.jcf - Formatted
```

### Formatting Rules

- Consistent spacing around operators (`=`, `+`, etc.)
- Proper indentation for nested structures
- Consistent comma placement in maps and lists
- Normalized quote usage

---

## jcl-watch - Auto-format on Save

Watch JCL files for changes and automatically format them.

### Usage

```bash
jcl-watch <paths>... [options]
```

### Options

- `-r, --recursive` - Recursive watch for directories
- `-c, --check` - Check only (don't modify files)
- `-v, --verbose` - Verbose output

### Examples

**Watch a single file:**
```bash
$ jcl-watch config.jcf
🔍 JCL Watch Mode
Watching for changes... (Press Ctrl+C to stop)

✓ config.jcf - Formatted
```

**Watch a directory:**
```bash
$ jcl-watch ./configs --recursive
🔍 JCL Watch Mode
Watching for changes... (Press Ctrl+C to stop)

👁️  Watching: ./configs
```

**Check mode (no modifications):**
```bash
$ jcl-watch config.jcf --check
🔍 JCL Watch Mode
Watching for changes... (Press Ctrl+C to stop)

! config.jcf - Needs formatting
```

---

## jcl-bench - Performance Benchmarking

Benchmark JCL parsing and evaluation performance.

### Usage

```bash
jcl-bench [file] [options]
```

### Options

- `-n, --iterations <N>` - Number of iterations (default: 1000)
- `-v, --verbose` - Show detailed timing for each iteration
- `--builtin` - Run built-in benchmarks

### Examples

**Benchmark a file:**
```bash
$ jcl-bench config.jcf
JCL Benchmarking Tool

Benchmarking: config.jcf
Iterations: 1000

📊 Parsing Benchmark
  Average: 559.041µs
  Min:     521.958µs
  Max:     1.311041ms

📊 Evaluation Benchmark
  Average: 22.307µs
  Min:     19.125µs
  Max:     140.042µs

Summary
──────────────────────────────────────────────────
Total parsing time:    55.904162ms (96.2%)
Total evaluation time: 2.230706ms (3.8%)
Total time:            58.134868ms

Operations per second:
  Parsing:    1788 ops/sec
  Evaluation: 44828 ops/sec
  Combined:   1720 ops/sec
```

**Custom iteration count:**
```bash
$ jcl-bench config.jcf -n 10000
...
```

**Built-in benchmarks:**
```bash
$ jcl-bench --builtin
JCL Benchmarking Tool

Running Built-in Benchmarks

Testing: Simple arithmetic
📊 Parsing Benchmark
  Average: 38.161µs
  ...

Total: 4.095333ms (24418 ops/sec)

Testing: String operations
...
```

**Verbose mode:**
```bash
$ jcl-bench config.jcf -n 100 --verbose
...
  Iteration 1: 545.125µs
  Iteration 2: 532.792µs
  ...
```

---

## jcl-lsp - Language Server

Start a Language Server Protocol server for editor integration.

### Usage

```bash
jcl-lsp
```

### Features

The LSP server provides:
- **Syntax highlighting** - Semantic token highlighting
- **Diagnostics** - Parse errors, linting warnings, and schema validation errors
- **Schema validation** - Automatic schema discovery and real-time validation (see [LSP Guide](../guides/lsp.md#schema-validation))
- **Go to definition** - Jump to variable/function definitions
- **Find references** - Find all uses of a symbol
- **Rename symbol** - Rename variables/functions
- **Hover information** - View type and documentation
- **Code completion** - Auto-complete suggestions

### Editor Integration

**VS Code:**

The LSP server can be integrated with VS Code using the provided extension in `editors/vscode/`.

**Vim/Neovim:**

Use with any LSP client plugin (e.g., coc.nvim, nvim-lspconfig):

```vim
" Configure JCL LSP
call coc#config('languageserver', {
  \ 'jcl': {
  \   'command': 'jcl-lsp',
  \   'filetypes': ['jcl']
  \ }
\ })
```

**Other Editors:**

Any editor with LSP support can use `jcl-lsp`. Configure it to start the `jcl-lsp` command for `.jcf` files.

---

## Common Workflows

### CI/CD Pipeline

```bash
#!/bin/bash
# Validate and format JCL configs in CI

# Check formatting
jcl-fmt --check *.jcf || exit 1

# Validate against schema
jcl-validate config.jcf --schema schema.yaml || exit 1

# Run tests
jcl eval test.jcf || exit 1

echo "✅ All checks passed!"
```

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Format all staged .jcf files
git diff --cached --name-only --diff-filter=ACM | grep '\.jcf$' | while read file; do
  jcl-fmt "$file"
  git add "$file"
done
```

### Development Workflow

```bash
# Start watch mode in one terminal
jcl-watch ./configs --recursive

# Edit configs in your editor
# Files are automatically formatted on save

# In another terminal, test configs
jcl eval configs/app.jcf
```

### Migration from JSON

```bash
# Migrate all JSON files to JCL
for file in *.json; do
  jcl-migrate "$file" -o "${file%.json}.jcf"
done

# Format the new files
jcl-fmt *.jcf

# Validate them
for file in *.jcf; do
  jcl-validate "$file" --schema schema.yaml
done
```

---

## jcl-module - Module Management

Manage JCL modules including creation, validation, dependency installation, and listing.

### Commands

#### init

Initialize a new JCL module with scaffolding.

```bash
jcl-module init <name> [OPTIONS]
```

**Options:**
- `-p, --path <PATH>` - Directory to create module in (defaults to module name)
- `-v, --version <VERSION>` - Module version (default: "0.1.0")
- `-d, --description <DESCRIPTION>` - Module description
- `-a, --author <AUTHOR>` - Module author
- `-l, --license <LICENSE>` - Module license (default: "MIT")

**Example:**
```bash
$ jcl-module init my-module \
    --version "0.1.0" \
    --description "My awesome module" \
    --author "Your Name" \
    --license "MIT"

Creating module 'my-module' in my-module
  ✓ Created jcl.json
  ✓ Created module.jcl
  ✓ Created README.md
  ✓ Created .gitignore

✓ Module 'my-module' initialized successfully!

Next steps:
  1. cd my-module
  2. Edit module.jcf to define your module
  3. Run 'jcl-module validate' to check your module
```

**Creates:**
- `jcl.json` - Module manifest with metadata and dependencies
- `module.jcf` - Module template with interface and outputs structure
- `README.md` - Documentation template with usage examples
- `.gitignore` - JCL cache and OS file exclusions

**Module Template:**
```jcl
# my-module Module

module.interface = (
    inputs = (
        # Define your module inputs here
        # Example:
        # name = (type = string, required = true, description = "Resource name")
    ),
    outputs = (
        # Define your module outputs here
        # Example:
        # id = (type = string, description = "Resource ID")
    )
)

module.outputs = (
    # Implement your module outputs here
    # Example:
    # id = "resource-${module.inputs.name}"
)
```

#### validate

Validate module structure, manifest, and interface.

```bash
jcl-module validate [PATH]
```

**Arguments:**
- `PATH` - Path to module directory (defaults to current directory)

**Example:**
```bash
$ jcl-module validate ./my-module

Validating module in ./my-module
  ✓ Valid manifest (jcl.json)
    Name: my-module
    Version: 0.1.0
  ✓ Main module file exists (module.jcf)
  ✓ Module file parses successfully
  ✓ Module interface defined
  ✓ Module outputs defined

  Dependencies:
    aws-base @ ^2.0.0
    networking @ ~1.5.0

✓ Module validation successful!
```

**Checks:**
- `jcl.json` manifest exists and is valid JSON
- Manifest contains required fields (name, version, main)
- Main module file exists
- Module file parses without syntax errors
- `module.interface` statement is present
- `module.outputs` statement is present
- All dependencies are listed

**Exit Codes:**
- `0` - Validation passed
- `1` - Validation failed

#### get

Download and install module dependencies from the registry.

```bash
jcl-module get [PATH]
```

**Arguments:**
- `PATH` - Path to module directory (defaults to current directory)

**Example:**
```bash
$ jcl-module get ./my-module

Downloading dependencies for module in ./my-module

  Resolving aws-base @ ^2.0.0...
    → Resolved to v2.1.3
    Downloading...
    ✓ Downloaded to /Users/user/.cache/jcl/modules/registry/aws-base/2.1.3

  Resolving networking @ ~1.5.0...
    → Resolved to v1.5.2
    Downloading...
    ✓ Downloaded to /Users/user/.cache/jcl/modules/registry/networking/1.5.2

✓ All dependencies downloaded successfully!
```

**Features:**
- Reads dependencies from `jcl.json` manifest
- Resolves version requirements using semantic versioning
- Downloads modules from the default registry
- Caches modules locally for performance
- Shows progress and resolved versions

**Cache Location:**
- Default: `~/.cache/jcl/modules/registry/`
- Structure: `{cache_dir}/{module_name}/{version}/`

#### list

List all installed modules.

```bash
jcl-module list [OPTIONS]
```

**Options:**
- `-v, --verbose` - Show detailed information including versions and descriptions

**Example (basic):**
```bash
$ jcl-module list

Installed modules:
  aws-base
  aws-ec2
  networking

Total: 3 module(s)
```

**Example (verbose):**
```bash
$ jcl-module list --verbose

Installed modules:

  aws-base
    v2.0.0
    v2.1.3
      Base AWS configuration module

  aws-ec2
    v1.2.3
      AWS EC2 instance configuration

  networking
    v1.5.0
    v1.5.2
      Network configuration utilities

Total: 3 module(s)
```

**Module Cache:**
The list command shows modules cached in:
- `~/.cache/jcl/modules/registry/` - Registry modules
- Organized by module name, then version

---

## Environment Variables

- `JCL_LSP_LOG_LEVEL` - Set LSP logging level (trace, debug, info, warn, error)
- `HOME` - Used for REPL history file location (`~/.jcf_history`)

---

## Exit Codes

All tools follow standard exit code conventions:

- `0` - Success
- `1` - Error (parse error, validation failure, etc.)
- `2` - Invalid usage (wrong arguments, missing files, etc.)

For `jcl-fmt --check`:
- `0` - All files properly formatted
- `1` - Some files need formatting

For `jcl-validate`:
- `0` - Validation passed
- `1` - Validation failed (unless `--no-fail` is used)
