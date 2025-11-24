# JCL Sample Files for GitHub Linguist

This directory contains representative `.jcf` (JCL Configuration Files) samples for submission to GitHub Linguist.

## Purpose

These files demonstrate the syntax and features of the JCL (Jack-of-All Configuration Language) and will be used as part of the PR to add JCL language support to GitHub Linguist.

## Files

### config.jcf
Demonstrates basic JCL configuration syntax including:
- Maps and nested structures
- Lists and arrays
- String interpolation
- Conditional expressions (`if`/`then`/`else`, `when`)
- List comprehensions
- Built-in functions
- Function definitions
- Heredoc strings
- Range syntax
- Null coalescing operators

### functions.jcf
Showcases JCL's functional programming features:
- Function definitions (`fn`)
- Lambda expressions (`=>`)
- Higher-order functions (`map`, `filter`, `reduce`)
- List comprehensions with filters
- Nested comprehensions
- String manipulation
- Collection operations
- Encoding/decoding (JSON, YAML, TOML, Base64)
- Hashing functions
- Template rendering
- Let bindings

### kubernetes.jcf
Real-world example of using JCL for Kubernetes-style configuration:
- Complex nested map structures
- Resource definitions
- Multi-line strings with heredoc
- List comprehensions for generating multiple resources
- Helper functions for DRY configuration
- Environment-specific configuration
- YAML generation

## Usage in Linguist PR

These files will be placed in the `samples/JCL/` directory of the github/linguist repository as part of the language addition PR.

## Language Characteristics

- **File extension**: `.jcf`
- **Type**: Configuration/Data
- **Key features**:
  - Human-readable configuration syntax
  - Support for complex data structures
  - Functional programming capabilities
  - Built-in functions for common operations
  - Template and encoding support

## Links

- **Repository**: https://github.com/hemmer-io/jcl
- **Documentation**: https://jcl.hemmer.io/
- **License**: MIT OR Apache-2.0
