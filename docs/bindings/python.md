---
layout: page
title: Python Bindings
permalink: /bindings/python/
parent: Language Bindings
---

# Python Bindings

JCL provides high-performance Python bindings using [PyO3](https://pyo3.rs/), allowing you to use JCL directly in your Python applications.

## Installation

```bash
pip install jcl-lang
```

## Quick Start

```python
import jcl

# Load and evaluate a JCL file
config = jcl.eval_file("config.jcf")
print(config["database"]["host"])

# Evaluate JCL from a string
result = jcl.eval("""
app_name = "myapp"
version = "1.0.0"
port = 8080
""")

print(result)  # {'app_name': 'myapp', 'version': '1.0.0', 'port': 8080}
```

## API Reference

### `jcl.eval(source: str) -> dict`

Parse and evaluate JCL source code.

**Parameters**:
- `source` (str): JCL source code as a string

**Returns**: Dictionary containing the evaluated variables

**Raises**: `SyntaxError` if parsing fails, `RuntimeError` if evaluation fails

**Example**:
```python
config = jcl.eval('x = 42\ny = "hello"')
print(config)  # {'x': 42, 'y': 'hello'}
```

### `jcl.eval_file(path: str) -> dict`

Load and evaluate a JCL file.

**Parameters**:
- `path` (str): Path to the JCL file

**Returns**: Dictionary containing the evaluated variables

**Raises**: `IOError` if file cannot be read, `SyntaxError` if parsing fails

**Example**:
```python
config = jcl.eval_file("config.jcf")
database_host = config["database"]["host"]
```

### `jcl.parse(source: str) -> str`

Parse JCL source code and return a status message.

**Parameters**:
- `source` (str): JCL source code

**Returns**: String describing parse result (e.g., "Parsed 5 statements")

**Example**:
```python
result = jcl.parse("x = 42\ny = 100")
print(result)  # "Parsed 2 statements"
```

### `jcl.format(source: str) -> str`

Format JCL source code.

**Parameters**:
- `source` (str): JCL source code to format

**Returns**: Formatted JCL code

**Example**:
```python
formatted = jcl.format('x=42\ny="hello"')
print(formatted)
# Output:
# x = 42
# y = "hello"
```

### `jcl.lint(source: str) -> list`

Lint JCL source code and return issues.

**Parameters**:
- `source` (str): JCL source code to lint

**Returns**: List of dictionaries, each containing:
  - `rule` (str): The lint rule name
  - `message` (str): Description of the issue
  - `severity` (str): "error", "warning", or "info"
  - `suggestion` (str, optional): Suggested fix

**Example**:
```python
issues = jcl.lint(source_code)
for issue in issues:
    print(f"{issue['severity']}: {issue['message']}")
    if 'suggestion' in issue:
        print(f"  Suggestion: {issue['suggestion']}")
```

## Common Patterns

### Loading Configuration

```python
import jcl
import os

# Load config based on environment
env = os.getenv("APP_ENV", "development")
config = jcl.eval_file(f"config/{env}.jcf")

# Access nested values
database_url = f"postgresql://{config['database']['host']}:{config['database']['port']}/{config['database']['name']}"
```

### Using with Flask/Django

```python
from flask import Flask
import jcl

app = Flask(__name__)

# Load configuration at startup
app_config = jcl.eval_file("config.jcf")
app.config.update(app_config["flask"])

@app.route("/")
def index():
    return f"Running {app_config['app_name']} v{app_config['version']}"
```

### Dynamic Configuration with Templates

```python
import jcl

config_template = """
environment = "${env}"
database = (
    host = "${db_host}",
    port = ${db_port},
    name = "myapp_${env}"
)
"""

# You would evaluate this with environment variables
result = jcl.eval(config_template)
```

## Performance

The Python bindings use PyO3 for zero-copy data sharing between Python and Rust:

```python
import jcl
import time

# Typical performance: ~0.05ms per evaluation
start = time.time()
for _ in range(1000):
    config = jcl.eval_file("config.jcf")
elapsed = time.time() - start
print(f"Average: {elapsed/1000*1000:.2f}ms per load")
# Output: Average: 0.05ms per load
```

**50-100x faster** than using the CLI via subprocess:
- Library: ~0.05ms
- CLI subprocess: ~5ms

## Error Handling

```python
import jcl

try:
    config = jcl.eval_file("config.jcf")
except SyntaxError as e:
    print(f"Parse error: {e}")
except RuntimeError as e:
    print(f"Evaluation error: {e}")
except IOError as e:
    print(f"File error: {e}")
```

## Type Conversion

JCL types map to Python types as follows:

| JCL Type | Python Type |
|----------|------------|
| String | `str` |
| Int | `int` |
| Float | `float` |
| Bool | `bool` |
| Null | `None` |
| List | `list` |
| Map | `dict` |
| Function | `None` (not serializable) |

## Version

```python
import jcl

print(jcl.__version__)  # e.g., "1.0.0"
```

## Requirements

- Python 3.8 or higher
- Works on Linux, macOS, and Windows

## See Also

- [Language Specification](../reference/language-spec) - Full JCL syntax reference
- [Built-in Functions](../reference/functions) - Available functions
- [Node.js Bindings](./nodejs) - JavaScript/TypeScript usage
- [Ruby Bindings](./ruby) - Ruby usage
