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
$ jcl eval config.jcl
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
- Persistent history (`~/.jcl_history`)
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
$ jcl-validate config.jcl --schema schema.yaml
✅ Validation passed!
```

**With errors:**
```bash
$ jcl-validate invalid.jcl --schema schema.yaml
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
$ jcl-validate config.jcl --schema schema.yaml --verbose
📋 Schema loaded from schema.yaml
📄 Configuration loaded from config.jcl
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
$ jcl-migrate config.json -o config.jcl
$ cat config.jcl
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
$ jcl-fmt config.jcl
✓ config.jcl - Formatted

✅ Formatted 1 file(s)
```

**Format multiple files:**
```bash
$ jcl-fmt *.jcl
✓ config.jcl - Formatted
✓ app.jcl - Already formatted
✓ db.jcl - Formatted

✅ Formatted 3 file(s)
```

**Check formatting (CI mode):**
```bash
$ jcl-fmt --check config.jcl
✅ All files are properly formatted!

$ jcl-fmt --check *.jcl
! config.jcl - Needs formatting
! app.jcl - Needs formatting

⚠️ 2 file(s) need formatting:
  - config.jcl
  - app.jcl
```

**Verbose mode:**
```bash
$ jcl-fmt config.jcl --verbose
📝 Processing: config.jcl
✓ config.jcl - Formatted
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
$ jcl-watch config.jcl
🔍 JCL Watch Mode
Watching for changes... (Press Ctrl+C to stop)

✓ config.jcl - Formatted
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
$ jcl-watch config.jcl --check
🔍 JCL Watch Mode
Watching for changes... (Press Ctrl+C to stop)

! config.jcl - Needs formatting
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
$ jcl-bench config.jcl
JCL Benchmarking Tool

Benchmarking: config.jcl
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
$ jcl-bench config.jcl -n 10000
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
$ jcl-bench config.jcl -n 100 --verbose
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
- **Diagnostics** - Parse and evaluation errors
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

Any editor with LSP support can use `jcl-lsp`. Configure it to start the `jcl-lsp` command for `.jcl` files.

---

## Common Workflows

### CI/CD Pipeline

```bash
#!/bin/bash
# Validate and format JCL configs in CI

# Check formatting
jcl-fmt --check *.jcl || exit 1

# Validate against schema
jcl-validate config.jcl --schema schema.yaml || exit 1

# Run tests
jcl eval test.jcl || exit 1

echo "✅ All checks passed!"
```

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Format all staged .jcl files
git diff --cached --name-only --diff-filter=ACM | grep '\.jcl$' | while read file; do
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
jcl eval configs/app.jcl
```

### Migration from JSON

```bash
# Migrate all JSON files to JCL
for file in *.json; do
  jcl-migrate "$file" -o "${file%.json}.jcl"
done

# Format the new files
jcl-fmt *.jcl

# Validate them
for file in *.jcl; do
  jcl-validate "$file" --schema schema.yaml
done
```

---

## Environment Variables

- `JCL_LSP_LOG_LEVEL` - Set LSP logging level (trace, debug, info, warn, error)
- `HOME` - Used for REPL history file location (`~/.jcl_history`)

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
