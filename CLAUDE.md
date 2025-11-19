# CLAUDE.md - AI Assistant Development Guide

This document provides comprehensive guidance for AI assistants (primarily Claude) working on the JCL project. It covers the current state of the language, design decisions, development workflow, and coding standards.

## Table of Contents

1. [Project Overview](#project-overview)
2. [Language Design & Current State](#language-design--current-state)
3. [Architecture & Components](#architecture--components)
4. [Development Workflow](#development-workflow)
5. [Testing Requirements](#testing-requirements)
6. [Code Style & Standards](#code-style--standards)
7. [Issue & PR Templates](#issue--pr-templates)
8. [Common Pitfalls](#common-pitfalls)

---

## Project Overview

**JCL (Jack-of-All Configuration Language)** is a general-purpose configuration language with:
- **Version**: 1.0.0 (production-ready)
- **Language**: Rust (edition 2021)
- **Status**: 144 tests passing, zero warnings
- **License**: MIT OR Apache-2.0

### Project Goals

1. **Human-readable**: Clean syntax with minimal punctuation
2. **Type-safe**: Advanced static type inference + runtime validation
3. **Powerful**: 70+ built-in functions for common operations
4. **Embeddable**: Multi-language bindings (Python, Node.js, Go, Java, Ruby)
5. **Tooling**: Complete ecosystem (LSP, formatter, linter, validator, migrator)

### What JCL Is NOT

- **NOT** a full programming language (intentionally constrained)
- **NOT** infrastructure-specific (removed Terraform-like features in favor of general-purpose design)
- **NOT** dynamically typed (has advanced static type inference)

---

## Language Design & Current State

### Syntax Overview

#### Collections

```jcl
# Lists use square brackets
numbers = [1, 2, 3, 4, 5]
names = ["Alice", "Bob", "Carol"]

# Maps use parentheses with commas separating entries
config = (
    host = "localhost",
    port = 8080,
    enabled = true
)

# IMPORTANT: Multi-line maps REQUIRE commas
# This is WRONG:
bad_map = (
    host = "localhost"
    port = 8080
)

# This is CORRECT:
good_map = (
    host = "localhost",
    port = 8080
)
```

#### String Interpolation

```jcl
name = "World"
greeting = "Hello, ${name}!"
expression = "2 + 2 = ${2 + 2}"
```

#### Functions

```jcl
# Lambda functions
double = x => x * 2
add = (x, y) => x + y

# Named functions
fn triple(n) = n * 3
fn greet(name) = "Hello, ${name}!"

# Calling functions
result = double(21)  # 42
```

#### List Comprehensions

```jcl
# Python-style list comprehensions
doubled = [x * 2 for x in numbers]
filtered = [x for x in numbers if x > 2]

# IMPORTANT: JCL does NOT support standalone for loops
# This is WRONG:
# for x in list do something

# Use list comprehensions instead:
# results = [something(x) for x in list]
```

#### Null Safety

```jcl
# Optional chaining
value = config?.database?.host  # Returns null if any part is null

# Null coalescing
port = config?.port ?? 8080  # Use 8080 if port is null
```

#### Control Flow

```jcl
# Ternary operator
status = x > 0 ? "positive" : "negative"

# If/then/else expression
result = if x > 0 then "positive" else "negative"

# When expression (pattern matching)
grade = when score {
    >= 90 => "A"
    >= 80 => "B"
    >= 70 => "C"
    else => "F"
}
```

#### Error Handling

```jcl
# Try function for graceful error recovery
result = try(risky_operation(), "default_value")

# Error handling is expression-level, not statement-level
```

### Type System

#### Static Type Inference

JCL has **Hindley-Milner style type inference** that catches errors before runtime:

```rust
// In src/types.rs
pub struct TypeChecker {
    env: TypeEnvironment,
    errors: Vec<TypeError>,
}

impl TypeChecker {
    pub fn check_module(&mut self, module: &Module) -> Result<(), Vec<TypeError>>
    pub fn infer_expression(&self, expr: &Expression) -> Result<Type, TypeError>
}
```

**Key Points**:
- Type checking happens **before** evaluation
- Expression-level type checking (every expression has an inferred type)
- Type errors include precise source spans (line, column, length)
- 70+ built-in functions have type signatures pre-registered

#### Type Annotations

```jcl
# Optional type annotations for documentation/validation
x: Int = 42
name: String = "Alice"
config: Map = (host = "localhost", port = 8080)
```

### Built-in Functions (70+)

Organized in categories in `src/functions.rs`:

1. **String**: `upper`, `lower`, `trim`, `replace`, `split`, `join`, `format`, `substr`, `startswith`, `endswith`, `contains`
2. **Encoding**: `jsonencode`, `jsondecode`, `yamlencode`, `yamldecode`, `toml encode`, `tomldecode`, `base64encode`, `base64decode`, `urlencode`, `urldecode`
3. **Collections**: `merge`, `lookup`, `keys`, `values`, `length`, `sort`, `reverse`, `distinct`, `flatten`, `zip`, `contains`, `concat`
4. **Higher-Order**: `map`, `filter`, `reduce`
5. **Numeric**: `min`, `max`, `sum`, `avg`, `abs`, `ceil`, `floor`, `round`, `pow`, `sqrt`
6. **Hashing**: `md5`, `sha1`, `sha256`, `sha512`
7. **Filesystem**: `file`, `fileexists`, `dirname`, `basename`, `pathexpand`
8. **Templating**: `template`, `templatefile` (uses Handlebars)
9. **Type Conversion**: `tostring`, `tonumber`, `tobool`, `totype`
10. **Time**: `timestamp`, `formatdate`, `timeadd`
11. **Validation**: `regex`, `regexreplace`
12. **Error Handling**: `try`

---

## Architecture & Components

### File Structure

```
src/
├── lib.rs              # Public API exports
├── main.rs             # CLI entry point (jcl binary)
├── lexer.pest          # PEG grammar for tokenization
├── lexer.rs            # Lexer implementation
├── parser.rs           # Original parser (being phased out)
├── token_parser.rs     # New Pratt parser for expressions
├── ast.rs              # Abstract Syntax Tree definitions
├── types.rs            # Type system (inference, checking, environment)
├── evaluator.rs        # Expression evaluation engine
├── functions.rs        # All built-in functions
├── formatter.rs        # Code formatter (jcl-fmt)
├── linter.rs           # Linting rules and diagnostics
├── lsp.rs              # Language Server Protocol implementation
├── repl.rs             # Interactive REPL
├── schema.rs           # Schema validation
├── migration.rs        # Format migration (JSON/YAML/TOML -> JCL)
├── symbol_table.rs     # Symbol tracking for LSP (go-to-def, references)
├── docgen.rs           # Documentation generator
├── bin/                # CLI tools
│   ├── jcl-lsp.rs
│   ├── jcl-fmt.rs
│   ├── jcl-validate.rs
│   ├── jcl-migrate.rs
│   ├── jcl-watch.rs
│   └── jcl-bench.rs
└── bindings/           # Multi-language bindings
    ├── ffi.rs          # C FFI
    ├── wasm.rs         # WebAssembly
    ├── python.rs       # PyO3
    ├── nodejs.rs       # Neon
    ├── java.rs         # JNI
    └── ruby.rs         # Magnus
```

### Key Data Structures

#### AST (src/ast.rs)

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Module {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Statement {
    Assignment { name: String, value: Expression, type_annotation: Option<Type>, span: Option<SourceSpan>, doc_comments: Vec<String> },
    FunctionDef { name: String, params: Vec<Parameter>, return_type: Option<Type>, body: Expression, span: Option<SourceSpan>, doc_comments: Vec<String> },
    Import { path: String, alias: Option<String>, span: Option<SourceSpan> },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    Literal(Value),
    Identifier(String),
    Binary { left: Box<Expression>, op: BinaryOperator, right: Box<Expression>, span: Option<SourceSpan> },
    Unary { op: UnaryOperator, operand: Box<Expression>, span: Option<SourceSpan> },
    FunctionCall { name: String, args: Vec<Expression>, span: Option<SourceSpan> },
    Lambda { params: Vec<String>, body: Box<Expression>, span: Option<SourceSpan> },
    // ... many more variants
}
```

#### Type Environment (src/types.rs)

```rust
#[derive(Debug, Clone)]
pub struct TypeEnvironment {
    variables: HashMap<String, Type>,
    functions: HashMap<String, Type>,
    parent: Option<Box<TypeEnvironment>>,
}

impl TypeEnvironment {
    pub fn new() -> Self { /* ... */ }
    pub fn child(&self) -> Self { /* Creates nested scope */ }
    pub fn register_builtins(&mut self) { /* Registers 70+ function signatures */ }
    pub fn define_variable(&mut self, name: String, ty: Type) { /* ... */ }
    pub fn lookup_variable(&self, name: &str) -> Option<Type> { /* Searches up parent chain */ }
}
```

---

## Development Workflow

### 1. NEVER Push Directly to Main

**CRITICAL RULE**: NEVER push commits directly to the `main` branch.

**ALWAYS follow this workflow**:

```bash
# ❌ NEVER DO THIS:
git push origin main

# ✅ ALWAYS DO THIS:
# 1. Create a feature branch
git checkout -b feature/your-feature-name

# 2. Make changes and commit
git add .
git commit -m "Your commit message"

# 3. Push to feature branch
git push origin feature/your-feature-name

# 4. Create an issue first (if not exists)
gh issue create --title "..." --body "..."

# 5. Create a pull request referencing the issue
gh pr create --title "..." --body "Closes #XX ..."
```

**Why this matters**:
- Maintains code review process
- Enables CI/CD checks before merging
- Creates audit trail via issues and PRs
- Prevents accidental breaking changes
- Allows for discussion and collaboration

**Exception**: There are NO exceptions. Even trivial changes must go through PR process.

### 2. Always Read Files First

**CRITICAL**: Always use the `Read` tool before `Edit` or `Write`:

```
❌ WRONG:
Edit tool without reading first

✅ CORRECT:
1. Read the file
2. Analyze the content
3. Edit with precise old_string/new_string
```

### 3. Test Before Committing

**MANDATORY**: All changes must pass tests AND formatting checks before committing:

```bash
# REQUIRED BEFORE EVERY COMMIT (in this order):

# 0. Check git status for unwanted files
git status
# Ensure .venv/, __pycache__/, target/, etc. are NOT in "Changes to be committed"
# Update .gitignore if needed before committing

# 1. Format code
cargo fmt --all

# 2. Check formatting is correct
cargo fmt --all -- --check

# 3. Run all tests
cargo test

# 4. Check for clippy warnings
cargo clippy --lib --tests --bins --all-features

# CRITICAL: If any of these fail, DO NOT commit. Fix the issues first.
```

**Pre-commit checklist**:
- [ ] `git status` - Verify no unwanted files (venv, build artifacts, etc.)
- [ ] `cargo fmt --all` - Format all code
- [ ] `cargo fmt --all -- --check` - Verify formatting
- [ ] `cargo test` - All 144 tests pass
- [ ] `cargo clippy --lib --tests --bins --all-features` - Zero warnings

**Test counts (as of v1.0.0)**:
- Unit tests: 117
- CLI integration tests: 18
- Integration tests: 9
- **Total: 144 tests**

### 3. Incremental Development

```
1. Read relevant files
2. Make small, focused changes
3. Test immediately
4. Fix any errors
5. Repeat
```

### 4. Preserve Exact Indentation

When using `Edit` tool:
- **DO NOT** include line number prefixes in `old_string` or `new_string`
- **PRESERVE** exact indentation (tabs vs spaces)
- Match existing code style

Example:
```rust
// CORRECT old_string (no line numbers, exact indentation):
    pub fn check_module(&mut self, module: &Module) -> Result<(), Vec<TypeError>> {
        for statement in &module.statements {
            self.check_statement(statement)?;
        }
        Ok(())
    }
```

### 5. Handle Pattern Matching Carefully

Rust pattern matching requires all fields unless using `..`:

```rust
// If struct has more fields than you're matching:
Statement::FunctionDef {
    name,
    params,
    body,
    ..  // Ignore remaining fields
} => { /* ... */ }
```

---

## Testing Requirements

### Unit Tests

Located in each source file (e.g., `src/types.rs`):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_inference() {
        // Arrange
        let expr = Expression::Literal(Value::Int(42));
        let checker = TypeChecker::new();

        // Act
        let result = checker.infer_expression(&expr);

        // Assert
        assert_eq!(result.unwrap(), Type::Int);
    }
}
```

### CLI Integration Tests

Located in `tests/cli_tests.rs`:

```rust
#[test]
fn test_jcl_eval_basic() {
    let jcl_path = get_binary_path("jcl");
    let test_file = create_temp_file(
        "test.jcl",
        r#"
x = 42
y = "hello"
"#,
    );

    let output = Command::new(&jcl_path)
        .arg("eval")
        .arg(&test_file)
        .output()
        .expect("Failed to execute jcl eval");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("x = 42"));
}
```

**Key Helpers**:
```rust
fn get_binary_path(name: &str) -> String { /* ... */ }
fn create_temp_file(name: &str, content: &str) -> String { /* ... */ }
```

### Test Syntax Correctness

When writing test JCL code:

✅ **Correct**:
```jcl
# Maps with commas
config = (host = "localhost", port = 8080)

# List comprehensions (not for loops)
doubled = [x * 2 for x in numbers]

# Lists for indexing
numbers = [1, 2, 3]
first = numbers[0]
```

❌ **Incorrect**:
```jcl
# Maps without commas (FAILS)
config = (
    host = "localhost"
    port = 8080
)

# Standalone for loops (NOT SUPPORTED)
for x in numbers do print(x)
```

---

## Code Style & Standards

### Rust Style

```rust
// Use cargo fmt default settings
// Run before committing:
cargo fmt

// Check with clippy:
cargo clippy
```

### Error Messages

Provide helpful, contextual error messages:

```rust
// GOOD:
return Err(TypeError {
    message: format!(
        "Type mismatch: expected {}, found {} at {}:{}",
        expected, found, span.line, span.column
    ),
    span: Some(span.clone()),
});

// BAD:
return Err(TypeError {
    message: "Type error".to_string(),
    span: None,
});
```

### Span Information

**ALWAYS** include source spans for diagnostics:

```rust
#[derive(Debug, Clone)]
pub struct SourceSpan {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
    pub length: usize,
}
```

This enables:
- Precise error messages
- LSP diagnostics with exact locations
- Better developer experience

### Documentation

Use rustdoc comments:

```rust
/// Checks the type of an expression and returns the inferred type.
///
/// # Arguments
///
/// * `expr` - The expression to type check
///
/// # Returns
///
/// Returns `Ok(Type)` with the inferred type, or `Err(TypeError)` if type checking fails.
///
/// # Examples
///
/// ```
/// let expr = Expression::Literal(Value::Int(42));
/// let ty = checker.infer_expression(&expr)?;
/// assert_eq!(ty, Type::Int);
/// ```
pub fn infer_expression(&self, expr: &Expression) -> Result<Type, TypeError> {
    // Implementation
}
```

---

## Issue & PR Templates

### When Creating Issues

Use the appropriate template:

1. **Bug Report** (`.github/ISSUE_TEMPLATE/bug_report.md`):
   - Clear description
   - Steps to reproduce
   - Minimal reproducible example
   - Expected vs actual behavior
   - Environment details

2. **Feature Request** (`.github/ISSUE_TEMPLATE/feature_request.md`):
   - Problem description
   - Proposed solution
   - Alternatives considered
   - Use cases

3. **Documentation** (`.github/ISSUE_TEMPLATE/documentation.md`):
   - What's missing/incorrect
   - Where it should be documented
   - Suggested improvements

### When Creating PRs

Follow the template (`.github/PULL_REQUEST_TEMPLATE.md`):

**Required Sections**:
```markdown
## Description
[Clear description of changes]

## Type of Change
- [x] Bug fix / New feature / etc.

## Testing
- [x] All existing tests pass
- [x] Added new tests
- [x] Tested manually

## Checklist
- [x] Ran cargo fmt
- [x] Ran cargo clippy
- [x] Updated documentation
- [x] No new warnings
```

**CRITICAL Checklist Items**:
1. ✅ All existing tests pass (`cargo test`)
2. ✅ Code formatted (`cargo fmt`)
3. ✅ No clippy warnings (`cargo clippy`)
4. ✅ Documentation updated (if needed)
5. ✅ No new compiler warnings

---

## Common Pitfalls

### 1. Map Syntax Without Commas

❌ **Wrong**:
```jcl
config = (
    host = "localhost"
    port = 8080
)
```

✅ **Correct**:
```jcl
config = (host = "localhost", port = 8080)
```

### 2. Using For Loops Instead of List Comprehensions

❌ **Wrong**:
```jcl
for x in numbers do print(x)
```

✅ **Correct**:
```jcl
results = [process(x) for x in numbers]
```

### 3. Forgetting to Register Built-in Function Types

When adding a new built-in function in `src/functions.rs`, also add its type signature in `src/types.rs`:

```rust
// In TypeEnvironment::register_builtins()
self.define_function("new_function".to_string(), Type::Function {
    params: vec![Type::String],
    return_type: Box::new(Type::String),
});
```

### 4. Not Including Spans in Errors

Always include source spans:

```rust
// GOOD:
LintIssue {
    severity: Severity::Error,
    message: "Undefined variable".to_string(),
    rule: "no-undefined-vars".to_string(),
    suggestion: Some("Did you mean 'count'?".to_string()),
    span: Some(expr.span.clone()),  // ✅ Includes span
}

// BAD:
LintIssue {
    /* ... */
    span: None,  // ❌ No span
}
```

### 5. Editing Files Without Reading Them First

The `Edit` tool will fail if you haven't read the file in the current conversation:

```
✅ CORRECT workflow:
1. Read(file_path)
2. Analyze content
3. Edit(file_path, old_string, new_string)

❌ WRONG:
1. Edit(file_path, ...) ← FAILS
```

### 6. Not Testing After Changes

**ALWAYS** run tests after making changes:

```bash
# After editing src/types.rs:
cargo test --lib types

# After editing CLI tools:
cargo test --test cli_tests

# Before committing:
cargo test && cargo clippy && cargo fmt --check
```

### 7. Assuming Infrastructure Features Exist

JCL used to be infrastructure-focused but is now **general-purpose**. The following NO LONGER EXIST:

❌ Removed/Never Existed:
- Provider system
- State management
- Resource definitions
- Plan/apply workflow (beyond REPL)
- Hemmer-specific features

✅ What Exists:
- General-purpose configuration
- Built-in functions
- Type system
- Templating
- Import system

---

## Design Decisions & Rationale

### Why Parentheses for Maps?

**Decision**: Use `()` for maps, `[]` for lists

**Rationale**:
- Differentiates from JSON/YAML syntax
- More human-readable (less visual noise)
- Parentheses group related configuration naturally
- Square brackets indicate ordered collections

### Why No Standalone For Loops?

**Decision**: List comprehensions only, no `for...do` syntax

**Rationale**:
- JCL is a configuration language, not a general programming language
- List comprehensions are declarative (what to compute)
- For loops are imperative (how to compute)
- Simpler mental model for configuration

### Why Static Type Inference?

**Decision**: Hindley-Milner style type inference before evaluation

**Rationale**:
- Catch errors before runtime
- Better developer experience with LSP
- Type-safe without verbose annotations
- Safer for production configuration

### Why Multi-Language Bindings?

**Decision**: Support Python, Node.js, Go, Java, Ruby, WASM, C FFI

**Rationale**:
- JCL is designed to be embeddable
- Different ecosystems have different needs
- Increases adoption potential
- Rust makes FFI relatively straightforward

---

## When Working on JCL

### General Guidelines

1. **Read CHANGELOG.md** to understand recent changes
2. **Check existing tests** to understand expected behavior
3. **Follow existing patterns** in the codebase
4. **Test thoroughly** before proposing changes
5. **Document design decisions** for future reference

### For Language Changes

1. Update `src/lexer.pest` if changing syntax
2. Update `src/ast.rs` if changing AST
3. Update `src/parser.rs` or `src/token_parser.rs` for parsing
4. Update `src/types.rs` for type inference
5. Update `src/evaluator.rs` for evaluation
6. Add tests in relevant `mod tests` blocks
7. Update documentation in `docs/`

### For Built-in Functions

1. Add function to `src/functions.rs`
2. Add type signature to `TypeEnvironment::register_builtins()` in `src/types.rs`
3. Add tests in `src/functions.rs`
4. Document in `docs/reference/functions.md`
5. Add docstring comments for `docgen`

### For Tooling (LSP, Formatter, etc.)

1. Understand existing implementation first
2. Make incremental changes
3. Test with real editor integration
4. Update relevant documentation

---

## Summary

**Key Takeaways**:
1. ✅ **Always test** before committing
2. ✅ **Always read** files before editing
3. ✅ Use **correct JCL syntax** (commas in maps, list comprehensions not for loops)
4. ✅ Include **source spans** in all diagnostics
5. ✅ Follow **issue/PR templates** exactly
6. ✅ **Zero warnings** policy (cargo clippy)
7. ✅ Document **design decisions** in code comments
8. ✅ JCL is **general-purpose**, not infrastructure-specific

**Current State**:
- Version: 1.0.0
- Tests: 144 passing
- Warnings: 0
- Ready for publication

**Resources**:
- Full docs: https://turner-hemmer.github.io/jcl/
- Language spec: `docs/reference/language-spec.md`
- Function reference: `docs/reference/functions.md`
- Contributing: `CONTRIBUTING.md`

---

*Last Updated: 2025-01-18 for JCL v1.0.0*
