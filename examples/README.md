# JCL Examples

This directory contains example JCL configuration files demonstrating various language features.

## Running Examples

You can run any example using the JCL CLI:

```bash
# View parsed output
cargo run --bin jcl eval examples/basic.jcl

# Output as JSON
cargo run --bin jcl eval examples/basic.jcf --format json

# Output as YAML
cargo run --bin jcl eval examples/basic.jcf --format yaml
```

## Real-World Examples

### `web-app-config.jcf`
**Web Application Configuration**
- Environment-specific settings with conditionals
- Database and cache configuration
- Feature flags and CORS settings
- Logging and monitoring setup
- Connection string generation

**Learn:** How to structure a complete web application configuration.

### `data-processing.jcf`
**Data Processing Pipeline**
- Filtering and transforming data
- Aggregations and statistics
- Grouping and sorting operations
- CSV/JSON/YAML export
- Report generation

**Learn:** Advanced list processing and data transformation techniques.

### `api-config.jcf`
**REST API Configuration**
- Endpoint definitions with rate limiting
- Authentication (JWT, OAuth providers)
- Security headers and middleware
- OpenAPI documentation setup
- Health check configuration

**Learn:** How to configure a production-ready REST API.

## Examples Overview

### `basic.jcf`
**Core Language Basics**
- Variable assignments with type inference
- Arithmetic operations (`+`, `-`, `*`, `/`, `%`)
- Boolean logic (`and`, `or`, `not`)
- Comparisons (`>`, `<`, `>=`, `<=`, `==`, `!=`)
- Null coalescing operator (`??`)
- If/then/else expressions

**Learn:** Start here to understand JCL syntax and basic expressions.

### `functions.jcf`
**Functions and Lambdas**
- Function definitions with `fn` keyword
- Type annotations for parameters and return types
- Lambda expressions (`x => x * 2`)
- Multi-parameter lambdas
- Higher-order functions with `map`

**Learn:** How to define and use functions in JCL.

### `collections.jcf`
**Lists and Maps**
- List literals with `[...]`
- Map literals with `(...)`
- List comprehensions with filters
- Member access with `.`
- Index access with `[n]`
- Nested data structures

**Learn:** Working with collections and data structures.

### `strings.jcf`
**String Operations**
- String literals with `"..."`
- String interpolation with `${...}`
- Multiline strings with `"""..."""`
- Nested interpolation
- Built-in string functions (`upper`, `lower`, `trim`, `replace`, `join`)

**Learn:** String manipulation and formatting.

### `conditionals.jcf`
**Control Flow**
- If/then/else expressions
- Ternary operator (`? :`)
- When expressions for pattern matching
- Pattern matching with guards
- Nested conditionals

**Learn:** Different ways to express conditional logic.

### `pipelines.jcf`
**Data Transformation**
- Pipe operator (`|`) for chaining operations
- Multi-stage data pipelines
- Combining `filter` and `map`
- Custom functions in pipelines
- String transformation pipelines

**Learn:** Functional data transformation patterns.

### `builtin.jcf`
**Built-in Functions**
- String functions: `upper`, `lower`, `trim`, `split`, `join`, `replace`
- Collection functions: `sort`, `reverse`, `filter`, `map`, `contains`, `slice`
- Numeric functions: `abs`, `round`, `floor`, `ceil`, `min`, `max`, `sum`
- Type conversion: `str`, `int`, `float`
- Encoding: `base64encode`, `urlencode`, `json`
- Hashing: `hash`, `md5`, `sha256`
- Object functions: `keys`, `values`, `merge`

**Learn:** The full standard library of built-in functions.

### `web-server.jcf`
**Real-World Configuration**
- Complex nested configuration structures
- Environment-based conditional logic
- String interpolation for URLs and connection strings
- Feature flags and environment detection
- Comprehensive application setup

**Learn:** Practical patterns for real-world configurations.

## Language Features Demonstrated

| Feature | Examples |
|---------|----------|
| Variables | `basic.jcf`, all examples |
| Arithmetic | `basic.jcf`, `functions.jcf` |
| Strings | `strings.jcf`, `web-server.jcf` |
| Interpolation | `strings.jcf`, `web-server.jcf` |
| Functions | `functions.jcf`, `pipelines.jcf` |
| Lambdas | `functions.jcf`, `pipelines.jcf` |
| Lists | `collections.jcf`, `builtin.jcf` |
| Maps | `collections.jcf`, `web-server.jcf` |
| Comprehensions | `collections.jcf` |
| Conditionals | `conditionals.jcf`, `web-server.jcf` |
| Pattern Matching | `conditionals.jcf` |
| Pipelines | `pipelines.jcf` |
| Built-ins | `builtin.jcf`, `strings.jcf` |

## Tips

1. **Start Simple**: Begin with `basic.jcf` to understand core syntax
2. **Explore Features**: Try `functions.jcf` and `collections.jcf` for more advanced patterns
3. **Real-World Usage**: Study `web-server.jcf` for production configuration patterns
4. **Experiment**: Modify examples and re-run to see how changes affect output
5. **Check Output**: Use `--format json` to see structured output

## Next Steps

After exploring these examples:
1. Try creating your own configuration file
2. Experiment with combining different features
3. Check the [Language Specification](../docs/spec.md) for complete syntax reference
4. Review [Built-in Functions](../docs/builtins.md) documentation
