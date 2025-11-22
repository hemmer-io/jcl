---
layout: default
title: Getting Started
nav_order: 1
permalink: /getting-started/
---

## What is JCL?

JCL (Jack-of-All Configuration Language) is a modern, powerful configuration language designed for clarity, flexibility, and developer productivity. It combines the simplicity of JSON with the power of a full programming language.

## Key Features

- **Simple & Readable** - Clean syntax without excessive punctuation
- **Powerful Functions** - 70+ built-in functions for strings, encoding, hashing, dates, and more
- **Type Safe** - Optional type annotations and schema validation
- **Developer Friendly** - LSP support, syntax highlighting, auto-formatting
- **Cross-Platform** - Native Rust implementation with bindings for Python, Node.js, Go, Java, and Ruby

## Installation

### From Source

```bash
git clone https://github.com/hemmer-io/jcl
cd jcl
cargo build --release

# Install CLI tools
cargo install --path .
```

### Using Language Bindings

**Python:**
```bash
pip install jcl-lang
```

**Node.js:**
```bash
npm install @hemmer-io/jcl
```

**Go:**
```bash
go get github.com/hemmer-io/jcl
```

## Your First JCL File

Create a file `config.jcf`:

```jcl
name = "my-app"
version = "1.0.0"
port = 8080
enabled = true

database = (
    host = "localhost",
    port = 5432,
    name = "myapp_db"
)

features = ["auth", "api", "websockets"]
```

Evaluate it:

```bash
jcl eval config.jcf
```

## Basic Syntax

### Variables

```jcl
name = "John"
age = 30
active = true
score = 95.5
```

### Lists

```jcl
ports = [80, 443, 8080]
names = ["Alice", "Bob", "Carol"]
mixed = [1, "two", true, 4.5]

# Generate number sequences with ranges
numbers = [0..5]           # [0, 1, 2, 3, 4, 5]
evens = [0..10:2]          # [0, 2, 4, 6, 8, 10]
countdown = [5..1:-1]      # [5, 4, 3, 2, 1]
```

### Maps (Objects)

```jcl
person = (
    name = "Alice",
    age = 30,
    email = "alice@example.com"
)

# Nested maps
config = (
    app = (
        name = "myapp",
        version = "1.0.0"
    ),
    database = (
        host = "localhost",
        port = 5432
    )
)
```

### String Interpolation

```jcl
name = "World"
greeting = "Hello, ${name}!"

port = 8080
url = "http://localhost:${port}"
```

### Multi-line Strings (Heredocs)

Heredoc syntax is ideal for embedding scripts, SQL, YAML, or any multi-line content:

```jcl
# Basic heredoc
startup_script = <<BASH
#!/bin/bash
echo "Starting application..."
./app --port 8080
BASH

# Heredoc with variable interpolation
db_name = "myapp"
init_sql = <<SQL
CREATE DATABASE ${db_name};
USE ${db_name};
CREATE TABLE users (id INT, name VARCHAR(100));
SQL

# Heredoc with indentation stripping (<<-)
# Automatically removes common leading whitespace
nginx_config = <<-CONF
    server {
        listen 80;
        server_name example.com;
        root /var/www/html;
    }
CONF
```

### Comments

```jcl
# Single-line comment

x = 42  # Inline comment
```

### Conditionals

```jcl
env = "production"
debug = if env == "development" then true else false

port = if env == "production" then 80 else 8080
```

### Functions

```jcl
# Define a function
fn double(x) = x * 2

# Use it
result = double(21)  # 42

# Function with multiple parameters
fn add(a, b) = a + b
sum = add(10, 20)  # 30

# Multi-line function
fn greet(name) = (
    prefix = "Hello",
    message = "${prefix}, ${name}!"
)
```

### List Comprehensions

```jcl
# Map over a list
numbers = [1, 2, 3, 4, 5]
doubled = [x * 2 for x in numbers]
# Result: [2, 4, 6, 8, 10]

# Use ranges to generate sequences
squares = [x * x for x in [1..10]]
# Result: [1, 4, 9, 16, 25, 36, 49, 64, 81, 100]

# Filter a list
evens = [x for x in numbers if x % 2 == 0]
# Result: [2, 4]

# Transform objects
users = [(name = "Alice"), (name = "Bob")]
names = [user.name for user in users]
# Result: ["Alice", "Bob"]

# Extract attributes with splat operator (shorthand)
users = [(name = "Alice", age = 30), (name = "Bob", age = 25)]
names = users[*].name  # ["Alice", "Bob"]
ages = users[*].age    # [30, 25]
```

### Type Annotations

```jcl
name: String = "Alice"
age: Int = 30
score: Float = 95.5
active: Bool = true

numbers: List<Int> = [1, 2, 3]
config: Map<String, Int> = (x = 1, y = 2)
```

## Built-in Functions

JCL comes with 70+ built-in functions organized by category:

### String Functions

```jcl
upper("hello")           # "HELLO"
lower("WORLD")           # "world"
trim("  space  ")        # "space"
split("a,b,c", ",")      # ["a", "b", "c"]
join(["a", "b"], "-")    # "a-b"
replace("hello", "l", "L")  # "heLLo"
```

### List Functions

```jcl
length([1, 2, 3])        # 3
reverse([1, 2, 3])       # [3, 2, 1]
sort([3, 1, 2])          # [1, 2, 3]
unique([1, 2, 2, 3])     # [1, 2, 3]
flatten([[1, 2], [3, 4]]) # [1, 2, 3, 4]
```

### Encoding Functions

```jcl
base64_encode("hello")   # "aGVsbG8="
base64_decode("aGVsbG8=") # "hello"
url_encode("hello world") # "hello%20world"
```

### Hashing Functions

```jcl
md5("hello")            # "5d41402abc4b2a76b9719d911017c592"
sha1("hello")           # "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d"
sha256("hello")         # "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c..."
```

### Date/Time Functions

```jcl
now()                   # Current timestamp
format_date(now(), "%Y-%m-%d")
```

### Type Checking

```jcl
is_string("hello")      # true
is_int(42)              # true
is_float(3.14)          # true
is_bool(true)           # true
is_list([1, 2])         # true
is_map((x = 1))         # true
```

[See full function reference →](../reference/functions/)

## CLI Tools

JCL comes with a comprehensive suite of tools:

### jcl - Main CLI

```bash
# Evaluate a file
jcl eval config.jcf

# Start REPL
jcl repl

# Format files
jcl fmt config.jcf

# Lint files
jcl lint config.jcf
```

### jcl-validate - Schema Validation

```bash
# Validate against a schema
jcl-validate config.jcf --schema schema.yaml
```

### jcl-migrate - Format Migration

```bash
# Convert from JSON
jcl-migrate config.json > config.jcf

# Convert from YAML
jcl-migrate config.yaml -o config.jcf

# Convert from TOML
jcl-migrate config.toml > config.jcf
```

### jcl-fmt - Code Formatter

```bash
# Format files in place
jcl-fmt config.jcf

# Check formatting (CI mode)
jcl-fmt --check config.jcf

# Format multiple files
jcl-fmt *.jcf
```

### jcl-watch - Auto-format on Save

```bash
# Watch a directory
jcl-watch ./configs --recursive

# Watch specific files
jcl-watch config.jcf app.jcf
```

### jcl-bench - Performance Benchmarking

```bash
# Benchmark a file
jcl-bench config.jcf

# Run built-in benchmarks
jcl-bench --builtin

# Custom iteration count
jcl-bench config.jcf -n 10000
```

### jcl-lsp - Language Server

```bash
# Start LSP server
jcl-lsp
```

## Interactive REPL

The JCL REPL provides an interactive environment for experimenting:

```bash
$ jcl repl
JCL REPL v0.1.0
Type :help for help, :quit to exit

jcl:1 x = 42
✓
jcl:2 x * 2
84
jcl:3 fn double(n) = n * 2
✓
jcl:4 double(21)
42
jcl:5 :vars
Variables:
  double = <function>
  x = 42
jcl:6 :quit
Goodbye!
```

**REPL Features:**
- Persistent history (`~/.jcf_history`)
- Multi-line input (use `\` at end of line)
- Tab completion
- History search (Ctrl-R)
- Variable inspection (`:vars`)

## Example: Complete Configuration

```jcl
# Application Configuration
app_name = "web-server"
version = "1.2.3"
environment = "production"

# Server configuration
server = (
    host = "0.0.0.0",
    port = if environment == "production" then 80 else 8080,
    workers = 4,
    timeout = 30
)

# Database configuration
database = (
    host = "db.example.com",
    port = 5432,
    name = "${app_name}_${environment}",
    pool_size = 20,
    ssl = environment == "production"
)

# Feature flags
features = (
    auth = true,
    api = true,
    websockets = environment == "production",
    debug = environment != "production"
)

# Allowed origins for CORS
cors_origins = [
    "https://example.com",
    "https://www.example.com"
]

# Build connection string
db_url = "postgres://${database.host}:${database.port}/${database.name}"

# Generate a unique deployment ID
deployment_id = sha256("${app_name}-${version}-${now()}")

# Log configuration
log_level = if features.debug then "debug" else "info"
log_format = "json"
```

## Working with Multiple Files

JCL supports importing configurations from other files for better organization and reusability.

### Basic Imports

```jcl
# database.jcf
database = (
    host = "localhost",
    port = 5432,
    name = "myapp_db"
)

connection_string = "postgres://${database.host}:${database.port}/${database.name}"
```

```jcl
# main.jcf
import "./database.jcf" as db

# Use imported configuration
app_config = (
    name = "myapp",
    db_url = db.connection_string
)
```

### Selective Imports

Import only what you need:

```jcl
# main.jcf
import (database, connection_string) from "./database.jcf"

# Use directly
app_config = (name = "myapp", db_url = connection_string)
```

### Organizing Configuration

**Recommended structure:**
```
config/
├── database.jcf       # Database settings
├── server.jcf         # Server configuration
├── features.jcf       # Feature flags
└── main.jcf           # Main config that imports others
```

**main.jcf:**
```jcl
import "./database.jcf" as db
import "./server.jcf" as srv
import "./features.jcf" as feat

# Combine configurations
config = (
    database = db.database,
    server = srv.server,
    features = feat.features
)
```

[Learn more about imports →](../reference/language-spec/#imports)

## Next Steps

- [Language Specification](../reference/language-spec/) - Complete syntax reference
- [Built-in Functions](../reference/functions/) - All 70+ functions documented
- [CLI Tools](../reference/cli-tools/) - Complete CLI reference
- [Comparison Guide](../guides/comparison/) - JCL vs JSON/YAML/TOML/HCL

## Editor Support

- **VS Code** - Full LSP support with syntax highlighting and auto-completion
- **Vim** - Syntax highlighting available in `editors/vim/`
- **Any LSP-compatible editor** - Use `jcl-lsp` for full language support
