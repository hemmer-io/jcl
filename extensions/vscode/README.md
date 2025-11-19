# JCL Language Support for Visual Studio Code

Provides comprehensive language support for JCL (Jack-of-All Configuration Language) files in Visual Studio Code.

## Features

- **Syntax Highlighting**: Full syntax highlighting for JCL files
- **Language Server Protocol (LSP)**: Real-time diagnostics, code completion, and more
- **Code Formatting**: Format JCL files with the built-in formatter
- **Linting**: Detect common issues and receive suggestions
- **Auto-closing**: Automatic closing of brackets, quotes, and parentheses
- **Comment Toggling**: Quick comment/uncomment with `Cmd+/` or `Ctrl+/`

## Requirements

This extension requires the `jcl-lsp` binary to be installed and available in your PATH. You can install it via:

### Using Cargo (Rust)

```bash
cargo install jcl
```

### Using Pre-built Binaries

Download the latest release from [GitHub Releases](https://github.com/hemmer-io/jcl/releases) and add the binaries to your PATH.

## Extension Settings

This extension contributes the following settings:

- `jcl.lsp.enabled`: Enable/disable the JCL Language Server (default: `true`)
- `jcl.lsp.path`: Path to the jcl-lsp executable (default: `jcl-lsp`)
- `jcl.format.enabled`: Enable/disable formatting on save (default: `true`)
- `jcl.lint.enabled`: Enable/disable linting (default: `true`)

## Commands

This extension provides the following commands:

- `JCL: Format Document`: Format the current JCL file
- `JCL: Lint Document`: Lint the current JCL file

## Language Features

### Syntax Highlighting

Full syntax highlighting for:
- Keywords: `if`, `then`, `else`, `when`, `for`, `in`, `fn`, `import`
- Operators: `+`, `-`, `*`, `/`, `==`, `!=`, `&&`, `||`, `??`, `?.`
- Built-in functions: `map`, `filter`, `jsonencode`, `merge`, and 70+ more
- String interpolation: `"Hello, ${name}!"`
- Comments: `# This is a comment`

### Language Server

The Language Server provides:
- **Diagnostics**: Real-time error and warning reporting
- **Code Completion**: Auto-complete for variables and functions
- **Hover Information**: Type information and documentation
- **Go to Definition**: Jump to variable/function definitions
- **Find References**: Find all usages of a variable/function

### Code Formatting

Format your JCL files automatically with proper indentation and spacing. Enable format-on-save in settings or use the command palette.

### Linting

Detect common issues:
- Unused variables
- Type mismatches
- Undefined variables
- Style violations

## Example JCL Code

```jcl
# Configuration example
app_name = "my-app"
version = "1.0.0"

# Database configuration
database = (
    host = "localhost",
    port = 5432,
    name = "mydb"
)

# List comprehension
ports = [80, 443, 8080]
formatted_ports = [format("Port: {}", p) for p in ports]

# Function definition
fn greet(name) = "Hello, ${name}!"

# Conditional expression
environment = if debug then "development" else "production"
```

## Known Issues

- Language server must be installed separately
- Template file validation requires filesystem access

## Release Notes

### 1.0.0

Initial release with:
- Full syntax highlighting
- LSP integration
- Code formatting
- Linting support
- Auto-closing pairs
- Comment toggling

## Contributing

Contributions are welcome! Please visit the [GitHub repository](https://github.com/hemmer-io/jcl) to report issues or submit pull requests.

## License

MIT OR Apache-2.0

## Learn More

- [JCL Documentation](https://hemmer-io.github.io/jcl/)
- [Language Reference](https://hemmer-io.github.io/jcl/reference/language-spec.html)
- [Built-in Functions](https://hemmer-io.github.io/jcl/reference/functions.html)
- [GitHub Repository](https://github.com/hemmer-io/jcl)
