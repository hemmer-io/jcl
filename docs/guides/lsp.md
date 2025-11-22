---
layout: default
title: Language Server Protocol (LSP)
parent: Guides
nav_order: 4
---

The JCL Language Server provides IDE features like diagnostics, autocomplete, and hover information.

## Installation

Build the LSP server:

```bash
cargo build --release --bin jcl-lsp
```

The binary will be available at `target/release/jcl-lsp`.

## Features

- **Diagnostics**: Real-time syntax checking, linting, and schema validation
  - Parse errors
  - Unused variables and functions
  - Naming convention violations
  - Type annotation suggestions
  - Redundant operations
  - **Schema validation errors** (if `.jcf-schema.json` or `.jcf-schema.yaml` present)

- **Autocomplete**: Intelligent code completion for:
  - All 70+ built-in functions
  - Keywords (fn, if, else, for, etc.)
  - Type names
  - Constants (true, false, null)

- **Hover**: Documentation on hover (basic)

- **Go to Definition**: Jump to variable and function definitions

- **Find References**: Find all usages of a symbol

- **Rename Symbol**: Rename variables and functions across files

## Editor Configuration

### VSCode

Add to your `settings.json`:

```json
{
  "jcl.languageServer": {
    "enabled": true,
    "path": "/path/to/jcl-lsp"
  }
}
```

Or use the generic LSP client extension and configure:

```json
{
  "languageServerExample.trace.server": "verbose",
  "languageServerExample.servers": {
    "jcl": {
      "command": "/path/to/jcl-lsp",
      "filetypes": ["jcl"],
      "rootPatterns": [".git/"]
    }
  }
}
```

### Neovim (with nvim-lspconfig)

Add to your `init.lua`:

```lua
local lspconfig = require('lspconfig')
local configs = require('lspconfig.configs')

-- Define JCL LSP if not already defined
if not configs.jcf then
  configs.jcf = {
    default_config = {
      cmd = {'/path/to/jcl-lsp'},
      filetypes = {'jcl'},
      root_dir = lspconfig.util.root_pattern('.git', 'jcl.toml'),
      settings = {},
    },
  }
end

-- Setup JCL LSP
lspconfig.jcf.setup{}
```

### Emacs (with lsp-mode)

Add to your `init.el`:

```elisp
(require 'lsp-mode)

(add-to-list 'lsp-language-id-configuration '(jcl-mode . "jcl"))

(lsp-register-client
 (make-lsp-client
  :new-connection (lsp-stdio-connection "/path/to/jcl-lsp")
  :major-modes '(jcl-mode)
  :server-id 'jcl))

(add-hook 'jcl-mode-hook #'lsp)
```

### Sublime Text (with LSP package)

Add to `LSP.sublime-settings`:

```json
{
  "clients": {
    "jcl": {
      "enabled": true,
      "command": ["/path/to/jcl-lsp"],
      "selector": "source.jcf"
    }
  }
}
```

## Schema Validation

The LSP server automatically discovers and loads schema files from your workspace root, providing real-time schema validation alongside linting errors.

### Automatic Schema Discovery

When the LSP server initializes, it searches for schema files in your workspace root in this order:

1. `.jcf-schema.json`
2. `.jcf-schema.yaml`
3. `.jcf-schema.yml`
4. `jcl-schema.json`
5. `jcl-schema.yaml`

The first file found is loaded and used for validation. Both JSON and YAML schema formats are supported.

### Example: Using Schema Validation

Create a schema file `.jcf-schema.json` in your workspace root:

```json
{
  "version": "1.0",
  "title": "Application Configuration",
  "type": {
    "kind": "map",
    "required": ["name", "port"],
    "properties": {
      "name": {
        "type": {"kind": "string", "min_length": 1}
      },
      "port": {
        "type": {"kind": "number", "minimum": 1, "maximum": 65535, "integer_only": true}
      }
    }
  }
}
```

Now edit a JCL file:

```jcl
# config.jcf
name = "my-app"
port = "8080"  # ← Error: Type mismatch: expected Int, found String (at config.port)
```

The LSP will show schema validation errors in real-time:
- Red squiggles under invalid values
- Error messages with field paths
- Suggestions for fixes (when available)

### Schema Validation Diagnostics

Schema validation errors appear alongside linting errors with:
- **Source**: `jcl-schema` (vs `jcl` for linter)
- **Code**: `schema-{ErrorType}` (e.g., `schema-TypeMismatch`)
- **Message**: Includes field path (e.g., "at config.database.port")

### Hot-Reloading

The schema is loaded when the LSP server initializes. To reload after changing the schema:
1. Restart your editor's LSP client, or
2. Reload the workspace/window in your editor

## Testing the LSP Server

Create a test file `test.jcf`:

```jcl
# This should show autocomplete for built-in functions
result = map(x => x * 2, [1, 2, 3])

# This should show a warning about unused variable
unusedVar = 42

# This should show naming convention warning
MyVariable = "test"

# This should show redundant operation warning
value = x + 0
```

Open the file in your configured editor and you should see:
- Diagnostics highlighting issues
- Autocomplete when typing function names
- Hover information on symbols

## Advanced Configuration

### Custom Lint Rules

The LSP uses the same linter as `jcl lint`. You can configure which rules to enable by setting environment variables before starting the LSP:

```bash
JCL_LINT_LEVEL=info jcl-lsp
```

### Logging

The LSP server logs to stderr. To see debug logs:

```bash
RUST_LOG=debug jcl-lsp 2> lsp.log
```

## Architecture

The JCL LSP implementation uses:
- **tower-lsp**: Async LSP framework for Rust
- **tokio**: Async runtime
- Integration with existing JCL parser and linter

The server maintains a document cache and re-parses/re-lints on every change, providing real-time feedback.

## Future Enhancements

Planned features:
- Code actions (quick fixes)
- Document symbols
- Workspace symbols
- Semantic highlighting
- Inlay hints for inferred types
- Precise error positioning for schema validation errors
- File watcher for schema hot-reload on edit
- Hover support showing schema requirements for fields
- Schema-based completion suggestions

## Troubleshooting

### LSP not starting

1. Check that `jcl-lsp` is executable: `chmod +x /path/to/jcl-lsp`
2. Verify it runs: `/path/to/jcl-lsp` (should wait for stdin)
3. Check editor LSP logs for errors

### No diagnostics appearing

1. Ensure file has `.jcf` extension
2. Check that file is syntactically valid
3. Look at LSP logs for errors

### Autocomplete not working

1. Verify LSP is connected (check editor status bar)
2. Try typing a known function name like `map`
3. Check that completion is triggered on `.` and `(`

## Contributing

To add new LSP features:

1. Modify `src/lsp.rs`
2. Add capabilities to `initialize()` method
3. Implement the corresponding trait method
4. Test with multiple editors
5. Update this documentation
