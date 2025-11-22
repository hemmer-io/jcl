# JCL for Visual Studio Code

Syntax highlighting and language support for JCL (Jack-of-All Configuration Language).

## Features

- **Syntax Highlighting**: Full syntax highlighting for JCL files
- **Bracket Matching**: Automatic bracket/parenthesis matching
- **Auto-closing Pairs**: Automatic closing of quotes, brackets, and parentheses
- **Comment Support**: Line comments with `#`
- **String Interpolation**: Highlighting for `${...}` expressions in strings

## Syntax Highlighting

The extension provides rich syntax highlighting for:

- **Keywords**: `fn`, `if`, `then`, `else`, `when`, `for`, `in`, `import`, `mut`
- **Types**: `string`, `int`, `float`, `bool`, `list`, `map`, `any`
- **Built-in Functions**: All 56+ JCL built-in functions including:
  - String functions: `upper`, `lower`, `trim`, `replace`, `split`, `join`
  - Encoding functions: `jsonencode`, `yamlencode`, `base64encode`
  - Collection functions: `map`, `filter`, `reduce`, `merge`, `keys`, `values`
  - Numeric functions: `min`, `max`, `sum`, `avg`
  - Hash functions: `md5`, `sha1`, `sha256`, `sha512`
  - Template functions: `template`, `templatefile`
- **Operators**: `=`, `==`, `!=`, `+`, `-`, `*`, `/`, `%`, `**`, `and`, `or`, `=>`, `??`, `?.`
- **Literals**: Strings, numbers, booleans (`true`/`false`), `null`
- **Lambda Expressions**: `x => x * 2`
- **String Interpolation**: `"Hello, ${name}!"`

## Example

```jcl
# Configuration with type annotations
name: string = "MyApp"
version: int = 2
active: bool = true

# Lambda functions
double = x => x * 2
add = (x, y) => x + y

# Higher-order functions
numbers = [1, 2, 3, 4, 5]
doubled = map(double, numbers)
evens = filter(x => x % 2 == 0, numbers)
sum = reduce((acc, x) => acc + x, numbers, 0)

# Template rendering
config = (port=8080, host="localhost")
message = template("Server: {{host}}:{{port}}", config)
```

## Installation

### From Marketplace

1. Open VS Code
2. Press `Ctrl+P` / `Cmd+P`
3. Type `ext install jcl`
4. Press Enter

### Manual Installation

1. Download the `.vsix` file from releases
2. Open VS Code
3. Press `Ctrl+Shift+P` / `Cmd+Shift+P`
4. Type "Install from VSIX"
5. Select the downloaded file

## File Association

The extension automatically activates for `.jcf` files.

## About JCL

JCL (Jack-of-All Configuration Language) is a modern, safe, and flexible general-purpose configuration language with powerful built-in functions, written in Rust.

**Key Features:**
- Clean, human-readable syntax
- 56+ built-in functions
- Runtime type validation
- Higher-order functions (map, filter, reduce)
- Template rendering with Handlebars
- Lambda expressions
- Interactive REPL

Learn more at: https://github.com/turner-hemmer/jcl

## License

MIT OR Apache-2.0

## Contributing

Contributions are welcome! Please visit the [GitHub repository](https://github.com/turner-hemmer/jcl) to report issues or submit pull requests.
