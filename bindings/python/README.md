# JCL Python Bindings

Python bindings for the Jack-of-All Configuration Language (JCL).

## Installation

```bash
pip install jcl-lang
```

Or build from source:

```bash
# Install maturin
pip install maturin

# Build and install
maturin develop --features python
```

## Usage

### Parse JCL Code

```python
import jcl_lang as jcl

# Parse JCL source code
result = jcl.parse("""
name = "my-app"
version = "1.0.0"
""")
print(result)  # "Parsed 2 statements"
```

### Evaluate JCL Code

```python
import jcl

# Evaluate JCL and get variables
result = jcl.eval("""
name = "my-app"
version = "1.0.0"
full_name = name ++ "-" ++ version
ports = [80, 443, 8080]
config = (
    debug = true,
    timeout = 30
)
""")

print(result["name"])        # "my-app"
print(result["version"])     # "1.0.0"
print(result["full_name"])   # "my-app-1.0.0"
print(result["ports"])       # [80, 443, 8080]
print(result["config"])      # {'debug': True, 'timeout': 30}
```

### Evaluate from File

```python
import jcl

# Load and evaluate a JCL file
result = jcl.eval_file("config.jcf")
print(result)
```

### Format JCL Code

```python
import jcl

# Format JCL source code
unformatted = """
x=1+2
y   =   "hello"
"""

formatted = jcl.format(unformatted)
print(formatted)
# Output:
# x = 1 + 2
# y = "hello"
```

### Lint JCL Code

```python
import jcl

# Lint JCL source code
issues = jcl.lint("""
x = 1
unused_var = 2
y = x + 1
""")

for issue in issues:
    print(f"{issue['severity']}: {issue['message']}")
    if 'suggestion' in issue:
        print(f"  Suggestion: {issue['suggestion']}")
```

## API Reference

### `jcl.parse(source: str) -> str`

Parse JCL source code and return a summary of the parsed AST.

**Parameters:**
- `source` (str): JCL source code to parse

**Returns:**
- str: Summary message about parsed statements

**Raises:**
- `SyntaxError`: If the source code has syntax errors

### `jcl.eval(source: str) -> dict`

Evaluate JCL source code and return all defined variables.

**Parameters:**
- `source` (str): JCL source code to evaluate

**Returns:**
- dict: Dictionary of variable names to values

**Raises:**
- `SyntaxError`: If the source code has syntax errors
- `RuntimeError`: If evaluation fails

### `jcl.eval_file(path: str) -> dict`

Load and evaluate a JCL file.

**Parameters:**
- `path` (str): Path to the JCL file

**Returns:**
- dict: Dictionary of variable names to values

**Raises:**
- `IOError`: If the file cannot be read
- `SyntaxError`: If the source code has syntax errors
- `RuntimeError`: If evaluation fails

### `jcl.format(source: str) -> str`

Format JCL source code.

**Parameters:**
- `source` (str): JCL source code to format

**Returns:**
- str: Formatted JCL source code

**Raises:**
- `SyntaxError`: If the source code has syntax errors

### `jcl.lint(source: str) -> list[dict]`

Lint JCL source code and return issues.

**Parameters:**
- `source` (str): JCL source code to lint

**Returns:**
- list[dict]: List of linter issues, each with:
  - `rule` (str): Linter rule that was violated
  - `message` (str): Description of the issue
  - `severity` (str): "error", "warning", or "info"
  - `suggestion` (str, optional): Suggestion for fixing the issue

**Raises:**
- `SyntaxError`: If the source code has syntax errors
- `RuntimeError`: If linting fails

## Type Conversions

JCL types are automatically converted to Python types:

| JCL Type | Python Type |
|----------|-------------|
| `string` | `str` |
| `int` | `int` |
| `float` | `float` |
| `bool` | `bool` |
| `null` | `None` |
| `list` | `list` |
| `map` | `dict` |
| `function` | `str` (displays as "&lt;function&gt;") |

## Example: Configuration Management

```python
import jcl

# Define infrastructure configuration in JCL
config = jcl.eval("""
# Environment configuration
env = "production"

# Database configuration
database = (
    host = "db.example.com",
    port = 5432,
    name = "myapp",
    pool_size = 10
)

# Application servers
servers = [
    (name = "app-1", ip = "10.0.1.10"),
    (name = "app-2", ip = "10.0.1.11"),
    (name = "app-3", ip = "10.0.1.12")
]

# Feature flags
features = (
    new_ui = env == "production",
    beta_features = env == "staging",
    debug_mode = env == "development"
)
""")

# Use the configuration in Python
print(f"Environment: {config['env']}")
print(f"Database: {config['database']['host']}:{config['database']['port']}")
print(f"Servers: {len(config['servers'])}")

for server in config['servers']:
    print(f"  - {server['name']}: {server['ip']}")
```

## License

MIT OR Apache-2.0
