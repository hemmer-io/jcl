# Contributing to JCL

Thank you for your interest in contributing to JCL! This document provides guidelines for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [How to Contribute](#how-to-contribute)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Submitting Changes](#submitting-changes)
- [Project Structure](#project-structure)

## Code of Conduct

By participating in this project, you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md). Please read it before contributing.

## Getting Started

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/YOUR-USERNAME/jcl.git
   cd jcl
   ```
3. Add the upstream repository:
   ```bash
   git remote add upstream https://github.com/hemmer-io/jcl.git
   ```

## Development Setup

### Prerequisites

- Rust 1.70 or later
- Cargo (comes with Rust)

### Building from Source

```bash
# Build the project
cargo build

# Build in release mode
cargo build --release

# Run tests
cargo test

# Run linter
cargo clippy

# Check formatting
cargo fmt --check
```

### Running the REPL

```bash
cargo run --bin jcl repl
```

### Running CLI Tools

```bash
# Run the main CLI
cargo run --bin jcl -- eval examples/hello.jcl

# Run formatter
cargo run --bin jcl-fmt -- config.jcl

# Run LSP
cargo run --bin jcl-lsp
```

## How to Contribute

### Reporting Bugs

Before submitting a bug report:
- Check the existing issues to avoid duplicates
- Collect information about your environment (OS, Rust version, JCL version)
- Create a minimal reproducible example

When submitting a bug report, include:
- A clear and descriptive title
- Steps to reproduce the issue
- Expected behavior
- Actual behavior
- Sample code that demonstrates the issue
- Your environment details

### Suggesting Features

Feature suggestions are welcome! Please:
- Check existing issues/PRs to avoid duplicates
- Provide a clear use case for the feature
- Explain how it fits with JCL's design philosophy
- Consider offering to implement it yourself

### Improving Documentation

Documentation improvements are always appreciated:
- Fix typos or clarify existing docs
- Add examples to the documentation
- Write guides or tutorials
- Improve code comments

## Coding Standards

### Rust Style Guide

- Follow the [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/)
- Use `cargo fmt` to format your code
- Run `cargo clippy` and address all warnings
- Write clear, descriptive variable and function names
- Add comments for complex logic
- Keep functions focused and reasonably sized

### Code Quality

- All code must pass `cargo test`
- All code must pass `cargo clippy` with no warnings
- Maintain or improve test coverage
- Add tests for new functionality
- Add documentation comments for public APIs

### Commit Messages

Write clear, meaningful commit messages:

```
Short (50 chars or less) summary

More detailed explanatory text, if necessary. Wrap it to about 72
characters. The blank line separating the summary from the body is
critical.

- Bullet points are okay
- Use imperative mood ("Add feature" not "Added feature")
- Reference issues and PRs when relevant (#123)
```

Good commit message examples:
- `Add support for null coalescing operator`
- `Fix parser crash on empty input`
- `Improve error messages for type mismatches`
- `Update documentation for string functions`

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test '*'
```

### Writing Tests

- Add unit tests in the same file as the code (in a `tests` module)
- Add integration tests in the `tests/` directory
- Test edge cases and error conditions
- Use descriptive test names that explain what is being tested

Example:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_integer() {
        let result = parse("42");
        assert_eq!(result.unwrap(), Value::Int(42));
    }

    #[test]
    fn test_parse_empty_string_fails() {
        let result = parse("");
        assert!(result.is_err());
    }
}
```

## Submitting Changes

### Pull Request Process

1. **Create a branch** for your changes:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** and commit them:
   ```bash
   git add .
   git commit -m "Add feature description"
   ```

3. **Keep your branch up to date**:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

4. **Run tests and linters**:
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ```

5. **Push your branch**:
   ```bash
   git push origin feature/your-feature-name
   ```

6. **Open a Pull Request** on GitHub with:
   - Clear title and description
   - Reference to any related issues
   - Screenshots/examples if applicable
   - Confirmation that tests pass

### PR Guidelines

- Keep PRs focused on a single feature or fix
- Include tests for new functionality
- Update documentation if needed
- Respond to review comments promptly
- Be patient - reviews may take time

### After Submitting

- CI will automatically run tests, linting, and builds
- Address any CI failures
- Respond to reviewer feedback
- Make requested changes in new commits (don't force-push until approved)
- Squash commits if requested before merging

## Project Structure

```
jcl/
├── src/
│   ├── ast.rs           # Abstract Syntax Tree definitions
│   ├── parser.rs        # Parser implementation
│   ├── evaluator.rs     # Expression evaluator
│   ├── functions.rs     # Built-in functions
│   ├── types.rs         # Type system
│   ├── linter.rs        # Linter implementation
│   ├── lsp.rs           # Language Server Protocol
│   ├── formatter.rs     # Code formatter
│   ├── symbol_table.rs  # Symbol tracking for LSP
│   ├── schema.rs        # Schema validation
│   ├── migration.rs     # Format migration
│   ├── bindings/        # Language bindings
│   │   ├── python.rs
│   │   ├── nodejs.rs
│   │   └── ...
│   └── bin/             # CLI binaries
│       ├── jcl-fmt.rs
│       ├── jcl-lsp.rs
│       └── ...
├── tests/               # Integration tests
├── examples/            # Example JCL files
├── docs/                # Documentation website
└── bindings/            # Language binding packages
```

## Questions?

If you have questions:
- Open an issue for discussion
- Check existing issues and documentation
- Reach out to maintainers

## License

By contributing to JCL, you agree that your contributions will be licensed under the same license as the project (MIT OR Apache-2.0).

Thank you for contributing to JCL! 🎉
