
---
layout: default
title: Language Specification
parent: Reference
nav_order: 1
permalink: /reference/language-spec/
---

A general-purpose configuration language designed for human readability, type safety, and powerful data manipulation.

## Table of Contents

- [Design Principles](#design-principles)
- [Syntax Overview](#syntax-overview)
- [Types](#types)
- [Collections](#collections)
- [Variables](#variables)
- [String Interpolation](#string-interpolation)
- [Operators](#operators)
- [Functions](#functions)
- [Control Flow](#control-flow)
- [Imports](#imports)
- [Error Handling](#error-handling)
- [Comments](#comments)

---

## Design Principles

1. **Human Readable** - Minimal punctuation, natural language flow
2. **Type Safe** - Strong typing with automatic inference
3. **Immutable by Default** - Variables are immutable unless explicitly marked mutable
4. **Fail Fast** - Errors are caught early with clear messages
5. **Composable** - Easy to break into reusable pieces

---

## Syntax Overview

### Basic Structure

```
# Assignment
name = "myapp"
version = "1.2.3"
enabled = true

# Collections
servers = ["web-1", "web-2", "api-1"]
config = (
  host = "localhost"
  port = 5432
  ssl = true
)

# Functions
result = upper("hello")
filtered = servers | filter s => contains(s, "web")

# Conditionals
size = env == "prod" ? "large" : "small"

# List comprehensions
server_configs = [
  (name = server, type = "t3.medium")
  for server in servers
]
```

---

## Types

### Primitive Types

```
# String
name: string = "myapp"
path: string = "/etc/config"

# Integer
count: int = 42
port: int = 8080

# Float
price: float = 19.99
ratio: float = 0.75

# Boolean
enabled: bool = true
debug: bool = false

# Null
value = null
```

### Type Inference

Types are automatically inferred:

```
name = "myapp"        # Inferred as string
count = 42            # Inferred as int
price = 19.99         # Inferred as float
enabled = true        # Inferred as bool
servers = ["web", "api"]  # Inferred as list<string>
```

### Optional Type Annotations

Explicit types are optional but recommended for clarity:

```
# Without annotation (inferred)
port = 8080

# With annotation (explicit)
port: int = 8080

# Annotation without initial value
port: int
port = 8080  # Assigned later
```

---

## Collections

### Lists

Use square brackets `[]` for lists:

```
# Simple list
servers = ["web-1", "web-2", "api-1"]

# Multi-line
ports = [
  80,
  443,
  8080
]

# Type annotation
servers: list<string> = ["web", "api"]

# Empty list
empty = []
empty: list<int> = []
```

### Range Syntax

Generate sequences of numbers using range syntax:

```
# Inclusive range (includes both start and end)
numbers = [0..5]         # [0, 1, 2, 3, 4, 5]
decade = [2020..2030]    # [2020, 2021, 2022, ..., 2030]

# Exclusive range (excludes end)
indices = [0..<5]        # [0, 1, 2, 3, 4]

# Range with step (custom increment)
evens = [0..10:2]        # [0, 2, 4, 6, 8, 10]
odds = [1..10:2]         # [1, 3, 5, 7, 9]

# Descending range (negative step)
countdown = [10..0:-1]   # [10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0]
reverse = [5..1:-1]      # [5, 4, 3, 2, 1]

# Float ranges
decimals = [0.0..2.0:0.5]  # [0.0, 0.5, 1.0, 1.5, 2.0]

# Use in list comprehensions
squares = [x * x for x in [1..10]]  # [1, 4, 9, 16, 25, 36, 49, 64, 81, 100]
```

**Range syntax details:**
- `[start..end]` - Inclusive range (includes `end`)
- `[start..<end]` - Exclusive range (excludes `end`)
- `[start..end:step]` - Range with custom step
- Step defaults to `1` for ascending, `-1` for descending
- Supports both integer and float ranges
- Range expressions return `list<int>` or `list<float>`

### Maps

Use parentheses `()` for maps:

```
# Simple map
config = (host = "localhost", port = 5432)

# Multi-line (preferred)
config = (
  host = "localhost"
  port = 5432
  ssl = true
)

# Nested maps
app = (
  name = "myapp"
  database = (
    host = "db.local"
    port = 5432
  )
  cache = (
    host = "redis.local"
    port = 6379
  )
)

# Type annotation
config: map<string, any> = (host = "localhost")

# Empty map
empty = ()
```

### Key Syntax in Maps

```
# Using equals sign
config = (key = value)

# Using colon (alternative, both valid)
config = (key: value)

# Mixed (not recommended but allowed)
config = (
  key1 = value1
  key2: value2
)
```

---

## Variables

### Assignment

Variables are immutable by default:

```
# Basic assignment
name = "myapp"
version = "1.2.3"

# Cannot reassign (ERROR)
name = "newapp"  # Error: Cannot reassign immutable variable
```

### Mutable Variables

Use `mut` keyword for mutable variables:

```
mut counter = 0
counter = counter + 1  # OK

mut items = []
items = items + ["new"]  # OK
```

### Scoping

Variables are lexically scoped:

```
x = 10

# In a block
result = (
  y = 20  # Local to this block
  x + y   # x is accessible, y is local
)

# y is not accessible here
```

### Computed Values

```
# Simple computation
total = price * quantity

# With functions
formatted = upper(name)

# Multi-step computation
result = (
  step1 = transform(data)
  step2 = filter(step1)
  finalize(step2)
)
```

---

## String Interpolation

Use `${...}` for interpolation:

```
# Simple variable
name = "world"
greeting = "Hello, ${name}!"

# Expressions
count = 5
message = "You have ${count} items"
math = "Result: ${2 + 2}"

# Nested paths
url = "http://${config.api.host}:${config.api.port}"

# Function calls
formatted = "User: ${upper(username)}"

# Complex expressions
status = "Server is ${running ? "up" : "down"}"
```

### Multi-line Strings

```
# Triple quotes for multi-line
script = """
#!/bin/bash
echo "Hello, ${name}"
cd /opt/app
./run.sh
"""

# Preserves indentation
config = """
  server {
    listen ${port};
    server_name ${domain};
  }
"""
```

### Heredoc Strings

Heredoc syntax (borrowed from Bash/Ruby/HCL) provides a cleaner way to embed multi-line content:

```
# Basic heredoc - preserves all whitespace
script = <<EOF
#!/bin/bash
echo "Hello, World"
./run.sh
EOF

# Heredoc with string interpolation
greeting = <<MSG
Hello, ${name}!
Welcome to ${app_name}.
MSG

# Heredoc with indentation stripping (<<-)
# Strips common leading whitespace from all lines
dockerfile = <<-DOCKERFILE
    FROM ubuntu:22.04
    RUN apt-get update
    RUN apt-get install -y nginx
    COPY . /app
    CMD ["nginx", "-g", "daemon off;"]
DOCKERFILE

# Multiple heredocs in one file
sql_query = <<SQL
SELECT *
FROM users
WHERE active = true
SQL

config_yaml = <<YAML
server:
  host: localhost
  port: 8080
YAML
```

**Heredoc Features:**
- `<<DELIMITER` - Preserves all whitespace exactly as written
- `<<-DELIMITER` - Strips common leading indentation from all lines
- Delimiter can be any alphanumeric identifier (e.g., `EOF`, `SQL`, `YAML`, `CONFIG`)
- Supports string interpolation with `${...}`
- Closing delimiter must be on its own line

**When to use heredocs:**
- Embedding scripts (bash, SQL, YAML, etc.)
- Multi-line configuration that needs clean indentation
- Any content where the triple-quote syntax feels cluttered
- Infrastructure-as-code scenarios (see Hemmer integration)

---

## Operators

### Arithmetic

```
x + y     # Addition
x - y     # Subtraction
x * y     # Multiplication
x / y     # Division
x % y     # Modulo
```

### Comparison

```
x == y    # Equal
x != y    # Not equal
x < y     # Less than
x <= y    # Less than or equal
x > y     # Greater than
x >= y    # Greater than or equal
```

### Logical

```
x and y   # Logical AND
x or y    # Logical OR
not x     # Logical NOT
```

### Null Safety

```
# Optional chaining - returns null if any part is null
value = config?.database?.host

# Null coalescing - provides default if null
host = config?.database?.host ?? "localhost"

# Combined
port = config?.database?.port ?? 5432
```

### Pipeline

```
# Pipeline operator - chains operations
result = data
  | trim
  | upper
  | split " "
  | sort

# With lambdas
numbers = [1, 2, 3, 4, 5]
  | filter x => x % 2 == 0
  | map x => x * 2
  | sum
```

### String Concatenation

```
# Using +
full_name = first + " " + last

# Using interpolation (preferred)
full_name = "${first} ${last}"
```

---

## Functions

### Built-in Functions

See [FUNCTIONS.md](./FUNCTIONS.md) for complete list. Examples:

```
# String functions
upper("hello")
lower("WORLD")
trim("  spaces  ")

# Encoding
jsonencode((name = "app"))
yamlencode((version = "1.0"))
tomlencode((key = "value"))

# Collections
length([1, 2, 3])
merge(map1, map2)
sort([3, 1, 2])

# Math
sum([1, 2, 3, 4])
max([10, 20, 5])
round(3.7)
```

### Lambda Functions

Single-expression anonymous functions:

```
# Single parameter
double = x => x * 2

# Multiple parameters
add = (x, y) => x + y

# Used inline
result = map([1, 2, 3], x => x * 2)

# Used in pipelines
filtered = data | filter x => x > 10
```

### Named Functions

Define reusable functions:

```
# Simple function
fn double(x) = x * 2

# Multiple parameters
fn add(x, y) = x + y

# Multi-line body
fn process(data) = (
  cleaned = trim(data)
  normalized = lower(cleaned)
  upper(normalized)
)

# With type annotations
fn multiply(x: int, y: int): int = x * y

# Call named functions
result = double(21)      # 42
sum = add(10, 20)        # 30
output = process("  HeLLo  ")  # "HELLO"
```

### Function Composition

```
# Define functions
fn double(x) = x * 2
fn square(x) = x * x

# Compose
numbers = [1, 2, 3]
  | map double
  | map square

# Or inline
process = x => square(double(x))
```

---

## Control Flow

### Conditionals

#### Ternary Operator

```
value = condition ? true_value : false_value

# Examples
size = env == "prod" ? "large" : "small"
port = ssl ? 443 : 80
```

#### If Expression

```
result = if condition then value1 else value2

# Multi-line
result = if x > 10
  then "large"
  else "small"
```

#### When Expression (Pattern Matching)

```
state = when env (
  "prod" => "running"
  "staging" => "running"
  "dev" => "stopped"
  * => "unknown"  # Default case
)

# With multiple conditions
size = when (env, resource) (
  ("prod", "web") => "t3.large"
  ("prod", "api") => "t3.xlarge"
  ("dev", *) => "t3.small"
  * => "t3.medium"
)

# With guards
category = when value (
  n if n < 0 => "negative"
  n if n == 0 => "zero"
  n if n > 0 and n < 10 => "small"
  n if n >= 10 => "large"
)
```

### Iteration

JCL uses list and map comprehensions for iteration, not standalone for loops.

#### List Comprehensions

List comprehensions provide a concise way to create lists by transforming and filtering iterables.

##### Basic Comprehensions

```
# Basic transformation
doubled = [x * 2 for x in numbers]

# With filter
evens = [x for x in numbers if x % 2 == 0]

# With transformation and filter
squares = [x * x for x in numbers if x > 0]

# String transformation
uppercased = [upper(s) for s in strings]
```

##### Multiple For Clauses (Flattening)

Multiple `for` clauses in a single comprehension create a flattened result, equivalent to nested loops:

```
# Cartesian product - flattened
pairs = [x + y for x in [1, 2] for y in [10, 20]]
# Result: [11, 21, 12, 22]
# Equivalent to: for x in [1,2]: for y in [10,20]: result.append(x+y)

# Flattening nested lists
nested = [[1, 2], [3, 4], [5, 6]]
flattened = [num for sublist in nested for num in sublist]
# Result: [1, 2, 3, 4, 5, 6]

# With filter on flattened result
positive_sums = [x + y for x in [-1, 0, 1] for y in [1, 2] if x + y > 0]
# Result: [1, 2, 1, 2, 3]
```

##### Nested Comprehensions

When a comprehension's expression is itself another comprehension, the result is nested (not flattened):

```
# 2D matrix
matrix = [[i * j for j in [1, 2, 3]] for i in [1, 2, 3]]
# Result: [[1, 2, 3], [2, 4, 6], [3, 6, 9]]

# Nested with filters
filtered_matrix = [[i * j for j in [1, 2, 3, 4, 5] if j > 2] for i in [1, 2, 3] if i > 1]
# Result: [[6, 8, 10], [9, 12, 15]]

# Processing 2D data
coordinates = [[[x, y] for y in [0, 1, 2]] for x in [0, 1, 2]]
# Result: [[[0,0], [0,1], [0,2]], [[1,0], [1,1], [1,2]], [[2,0], [2,1], [2,2]]]
```

**Key Difference:**
- **Multiple `for` in one comprehension** → flattened: `[expr for x in A for y in B]`
- **Nested comprehensions** → nested structure: `[[expr for y in B] for x in A]`

#### Splat Operator

Extract attributes from all elements in a list using the splat operator `[*]`:

```jcl
users = [
  (name = "Alice", age = 30),
  (name = "Bob", age = 25),
  (name = "Carol", age = 35)
]

# Extract a single field from all elements
names = users[*].name
# Result: ["Alice", "Bob", "Carol"]

ages = users[*].age
# Result: [30, 25, 35]
```

##### Nested Access

Chain member access after splat to access nested fields:

```jcl
orders = [
  (id = 1, customer = (name = "Alice", email = "alice@example.com")),
  (id = 2, customer = (name = "Bob", email = "bob@example.com"))
]

# Chain member access after splat
emails = orders[*].customer.email
# Result: ["alice@example.com", "bob@example.com"]

names = orders[*].customer.name
# Result: ["Alice", "Bob"]
```

##### Comparison with List Comprehensions

The splat operator is syntactic sugar for simple attribute extraction:

```jcl
# These are equivalent:
names_splat = users[*].name
names_comp = [u.name for u in users]

# Splat is more concise for simple field access
ages_splat = users[*].age
ages_comp = [u.age for u in users]

# List comprehensions are more flexible for complex transformations
upper_names = [upper(u.name) for u in users]  # Splat can't do this
filtered = [u.name for u in users if u.age > 25]  # Splat can't filter
```

**When to use splat vs list comprehensions:**
- **Use splat** (`[*]`) for simple attribute extraction
- **Use list comprehensions** for transformations, filtering, or complex logic

**Splat operator details:**
- `list[*].field` - Extract `field` from each element in `list`
- Requires a list (type error otherwise)
- Returns a list of the extracted values
- Works with chained member access: `list[*].a.b.c`
- Returns `list<T>` where `T` is the field type
- Empty list returns empty list: `[][*].field` → `[]`

#### List Slicing

Lists support Python-style slicing with `[start:end:step]` syntax:

```
numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

# Basic slicing [start:end]
first_three = numbers[0:3]       # [1, 2, 3]
middle = numbers[3:7]             # [4, 5, 6, 7]

# Omit start (defaults to 0)
first_five = numbers[:5]          # [1, 2, 3, 4, 5]

# Omit end (defaults to list length)
from_fifth = numbers[5:]          # [6, 7, 8, 9, 10]

# Full copy
copy = numbers[:]                 # [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

# Negative indices (from end)
last_three = numbers[-3:]         # [8, 9, 10]
all_but_last = numbers[:-1]       # [1, 2, 3, 4, 5, 6, 7, 8, 9]

# Step parameter [start:end:step]
evens = numbers[::2]              # [1, 3, 5, 7, 9] (every other)
odds = numbers[1::2]              # [2, 4, 6, 8, 10]

# Reverse with negative step
reverse = numbers[::-1]           # [10, 9, 8, 7, 6, 5, 4, 3, 2, 1]
partial_reverse = numbers[7:2:-1] # [8, 7, 6, 5, 4]

# Empty slices
empty = numbers[5:2]              # [] (start > end with positive step)
```

**Slice Semantics:**
- `start` (optional): Starting index (inclusive), defaults to 0 for positive step, end of list for negative step
- `end` (optional): Ending index (exclusive), defaults to list length for positive step, before beginning for negative step
- `step` (optional): Step size, defaults to 1. Cannot be 0. Negative step reverses direction
- Negative indices count from the end: `-1` is last element, `-2` is second-to-last, etc.
- Out-of-bounds indices are clamped to valid range

#### Map Comprehensions

```
# Transform map
doubled = {k: v * 2 for k, v in numbers}

# Filter map
large = {k: v for k, v in sizes if v > 100}

# Create map from list
indexed = {i: item for i, item in enumerate(items)}
```

---

## Imports

JCL supports importing values from other JCL files:

### Basic Import

```
# common.jcl
common_tags = (
  managed_by = "jcl"
  team = "platform"
)

default_config = (
  timeout = 30
  retries = 3
)
```

```
# main.jcl
import (common_tags, default_config) from "./common.jcl"

# Use imported values
my_tags = merge(common_tags, (app = "myapp"))
my_config = merge(default_config, (timeout = 60))
```

### Import All

```
import * from "./common.jcl"

# Access with namespace
tags = common.common_tags
config = common.default_config
```

### Import with Alias

```
import (common_tags as tags, default_config as config) from "./common.jcl"

# Use aliases
my_tags = merge(tags, (app = "myapp"))
```

### Relative and Absolute Paths

```
# Relative to current file
import values from "./config.jcl"
import values from "../shared/common.jcl"

# Absolute path
import values from "/etc/jcl/common.jcl"
```

---

## Error Handling

### Fail Fast (Default)

By default, any error stops evaluation:

```
# If file doesn't exist, evaluation stops with error
content = file("/path/to/missing.txt")

# If function fails, evaluation stops
result = some_operation()
```

### Try Function (Optional Fallback)

Use `try()` when you want to handle errors gracefully:

```
# Provide fallback value
content = try(file("/path/to/file.txt"), "default content")

# With null fallback
data = try(jsondecode(input), null)

# Chain with null coalescing
config = try(file("config.json"), null)
  ?? (default = "config")
```

### Null Coalescing for Missing Values

```
# Handle null/missing values
host = config?.database?.host ?? "localhost"

# Chain multiple fallbacks
value = primary ?? secondary ?? tertiary ?? "default"
```

### Error Messages

Errors should be clear and actionable:

```
Error: Function 'upper' requires 1 argument, got 0
  --> config.jcl:42:15
   |
42 | result = upper()
   |          ^^^^^^^ Expected 1 argument
   |

Error: Type mismatch
  --> config.jcl:15:8
   |
15 | port = "8080"
   |        ^^^^^^ Expected int, got string
   |
Help: Use tonumber() to convert: tonumber("8080")
```

---

## Comments

### Single-Line Comments

```
# This is a comment
name = "value"  # Inline comment
```

### Documentation Comments

```
## This is a documentation comment
## Used for generating documentation
## Can span multiple lines
fn process(data) = ...
```

### Multi-Line Comments

```
/*
This is a multi-line comment
It can span multiple lines
Useful for larger explanations
*/
```

---

## Complete Example

```
# Application configuration in JCL

# Constants
app_name = "myapp"
app_version = "2.1.0"

# Environments
environments = ["dev", "staging", "prod"]

# Environment-specific config
env_config = (
  dev = (
    instance_type = "t3.small"
    replicas = 1
    debug = true
  )
  staging = (
    instance_type = "t3.medium"
    replicas = 2
    debug = false
  )
  prod = (
    instance_type = "t3.large"
    replicas = 3
    debug = false
  )
)

# Helper function
fn get_config(env) = env_config[env] ?? env_config["dev"]

# Server definitions
servers = [
  (name = "web-1", role = "web", zone = "a"),
  (name = "web-2", role = "web", zone = "b"),
  (name = "api-1", role = "api", zone = "a")
]

# Generate configurations for each environment using map comprehension
config = {
  env: (
    app_name = app_name,
    app_version = app_version,
    settings = get_config(env),

    # Generate server configs
    servers = {
      s.name: (
        role = s.role,
        zone = s.zone,
        type = get_config(env).instance_type
      )
      for s in servers
    },

    # Generate deployment script
    deploy_script = """
#!/bin/bash
set -euo pipefail

APP="${app_name}"
VERSION="${app_version}"
ENV="${env}"

echo "Deploying $APP version $VERSION to $ENV"

${[
  "ssh ${s.name} 'docker pull $APP:$VERSION && docker restart $APP'"
  for s in servers
] | join "\n"}

echo "Deployment complete!"
"""
  )
  for env in environments
}

# Outputs
out.environments = environments
out.configs = config
out.deploy_dev = config.dev?.deploy_script ?? "No script"
```

---

## Reserved Keywords

```
# Control flow
if, then, else, when, match, for, in

# Type annotations
string, int, float, bool, list, map, any

# Declarations
fn, mut, import, from, as

# Literals
true, false, null

# Operators
and, or, not
```

---

## Syntax Rules

### Naming Conventions

```
# Variables: snake_case or kebab-case
my_variable = value
my-variable = value

# Functions: snake_case
fn my_function(x) = ...

# Constants: SCREAMING_SNAKE_CASE (by convention)
MAX_RETRIES = 3
API_URL = "https://api.example.com"
```

### Semicolons

Semicolons are **optional** and not recommended:

```
# Without semicolons (preferred)
x = 10
y = 20

# With semicolons (allowed but unnecessary)
x = 10;
y = 20;
```

### Line Breaks

Line breaks are significant for statement separation:

```
# Multiple statements
x = 10
y = 20
z = 30

# Single statement across multiple lines (use parentheses)
value = (
  step1 = transform(data)
  step2 = process(step1)
  finalize(step2)
)
```

### Whitespace

Whitespace is generally ignored except for line breaks:

```
# These are equivalent
result = x+y
result = x + y
result=x+y

# But this is different (line break separates statements)
result = x
+ y  # This is a separate statement
```

---

## Type System Rules

### Type Compatibility

```
# Exact match required
x: int = 42     # OK
x: int = "42"   # ERROR: Type mismatch

# Implicit conversions (none)
x: float = 42   # ERROR: Must explicitly convert

# Explicit conversion
x: float = tofloat(42)  # OK
```

### Null Handling

```
# Variables can be null
x = null

# Type annotations allow null
x: string? = null  # Optional string
x: string? = "value"  # OK

# Or use any type
x: any = null  # OK
```

### Collection Types

```
# Homogeneous lists (enforced)
numbers: list<int> = [1, 2, 3]     # OK
mixed: list<int> = [1, "two", 3]   # ERROR

# Heterogeneous lists (use any)
mixed: list<any> = [1, "two", true]  # OK

# Maps with typed values
config: map<string, string> = (key = "value")  # OK
```

---

This specification defines JCL v1.0. Implementation should strictly follow these rules for consistency and predictability.
