# JCL - Jack-of-All Configuration Language

A modern, safe, and flexible general-purpose configuration language with powerful built-in functions, written in Rust.

## Vision

JCL is a general-purpose configuration language designed to be human-readable, type-safe, and powerful. It provides a rich standard library of functions for data manipulation, encoding/decoding (YAML, JSON, Base64), templating, string operations, and more. Built in Rust for performance and safety, JCL can be embedded in other tools (like Hemmer for IaC) or used standalone for configuration management.

## Key Features

🎯 **General-Purpose Configuration**
- Clean, human-readable syntax with minimal punctuation
- Rich standard library of 50+ built-in functions
- Can be embedded or used standalone

🔒 **Safety First**
- Strong type system with inference
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

🚧 **Design Complete, Implementation Starting** 🚧

JCL v1.0 language specification is complete! We've finalized:
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

Next step: Phase 1 implementation (parser, evaluator, type checker).

## Integration with Hemmer

JCL is designed as a configuration language that can be used by other tools. **Hemmer** is a companion tool that uses JCL for infrastructure as code:

- **JCL**: The configuration language with syntax, parser, type system, and built-in functions
- **Hemmer**: Infrastructure provisioning tool that uses JCL for configuration (handles modules, registry, cloud providers, etc.)

This separation allows JCL to be a general-purpose configuration language that can be embedded in multiple tools, not just IaC.

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

**Core Language:**
- [x] Language specification and grammar (Pest PEG parser) - **v1.0 Complete!**
- [x] Built-in functions library (50+ functions)
- [x] Templating patterns documentation
- [ ] Parser implementation with error recovery
- [ ] Type system with inference
- [ ] Expression evaluator
- [ ] String interpolation engine
- [ ] REPL for interactive testing
- [ ] Language server protocol (LSP) support

**Tooling:**
- [ ] CLI for standalone use
- [ ] Syntax highlighting (VSCode, Vim, etc.)
- [ ] Formatter (jcl fmt)
- [ ] Linter
- [ ] Documentation generator

**Integration:**
- [ ] C FFI for embedding in other languages
- [ ] Python bindings
- [ ] WebAssembly compilation
- [ ] Integration examples with Hemmer

## Why JCL?

**vs. HCL (HashiCorp Configuration Language):**
- More human-readable syntax (less verbose, no braces)
- Richer built-in function library (50+ functions)
- Better type inference
- Cleaner string interpolation

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
