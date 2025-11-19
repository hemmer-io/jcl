---
layout: default
title: Home
nav_order: 0
permalink: /
---

# JCL - Jack-of-All Configuration Language
{: .fs-9 }

A powerful, flexible configuration language designed for modern infrastructure and application configuration.
{: .fs-6 .fw-300 }

## Features

- **Simple & Readable** - Clean syntax that's easy to learn and understand
- **Powerful Functions** - 70+ built-in functions for string manipulation, encoding, hashing, and more
- **Type Safe** - Optional type annotations and schema validation
- **Developer Friendly** - LSP support, syntax highlighting, formatting tools
- **Cross-Platform** - Bindings for Python, Node.js, Go, Java, and Ruby

## Quick Example

```jcl
name = "my-app"
version = "1.0.0"
port = 8080

database = (
    host = "localhost",
    port = 5432,
    name = "myapp_db"
)

features = ["auth", "api", "websockets"]

# Use powerful built-in functions
secret = base64_encode("my-secret-key")
config_hash = sha256(name + version)
```

## Getting Started

- [Installation](getting-started/index.html)
- [Language Basics](reference/language-spec.html)
- [Built-in Functions](reference/functions.html)
- [CLI Tools](reference/cli-tools.html)

## Tools

JCL comes with a comprehensive suite of CLI tools:

- **jcl** - Main CLI for evaluation and formatting
- **jcl-validate** - Schema validation
- **jcl-migrate** - Convert from JSON/YAML/TOML
- **jcl-fmt** - Code formatter
- **jcl-watch** - Auto-format on file changes
- **jcl-bench** - Performance benchmarking
- **jcl-lsp** - Language Server Protocol implementation

## Why JCL?

### vs JSON
- Comments and documentation
- No trailing comma issues
- More expressive syntax
- Built-in functions

### vs YAML
- No indentation nightmares
- Explicit syntax
- Type safety with schemas
- Faster parsing

### vs TOML
- More flexible data structures
- Function support
- Better for complex configurations
- Consistent syntax

[Learn more in our comparison guide →](guides/comparison.html)

## Community

- [GitHub Repository](https://github.com/hemmer-io/jcl)
- [Issue Tracker](https://github.com/hemmer-io/jcl/issues)
- [Contributing Guide](https://github.com/hemmer-io/jcl/blob/main/CONTRIBUTING.md)

## License

JCL is dual-licensed under MIT OR Apache-2.0.
