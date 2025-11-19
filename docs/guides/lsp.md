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

- **Diagnostics**: Real-time syntax checking and linting
  - Parse errors
  - Unused variables and functions
  - Naming convention violations
  - Type annotation suggestions
  - Redundant operations

- **Autocomplete**: Intelligent code completion for:
  - All 56+ built-in functions
  - Keywords (fn, if, else, for, etc.)
  - Type names
  - Constants (true, false, null)

- **Hover**: Documentation on hover (basic)

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
if not configs.jcl then
  configs.jcl = {
    default_config = {
      cmd = {'/path/to/jcl-lsp'},
      filetypes = {'jcl'},
      root_dir = lspconfig.util.root_pattern('.git', 'jcl.toml'),
      settings = {},
    },
  }
end

-- Setup JCL LSP
lspconfig.jcl.setup{}
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
      "selector": "source.jcl"
    }
  }
}
```

## Testing the LSP Server

Create a test file `test.jcl`:

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
- Go to definition
- Find references
- Rename symbol
- Code actions (quick fixes)
- Document symbols
- Workspace symbols
- Semantic highlighting
- Inlay hints for inferred types
- Position-aware diagnostics (currently whole-file)

## Troubleshooting

### LSP not starting

1. Check that `jcl-lsp` is executable: `chmod +x /path/to/jcl-lsp`
2. Verify it runs: `/path/to/jcl-lsp` (should wait for stdin)
3. Check editor LSP logs for errors

### No diagnostics appearing

1. Ensure file has `.jcl` extension
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
