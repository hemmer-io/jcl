---
layout: default
title: JCL vs Other Configuration Languages
parent: Guides
nav_order: 1
permalink: /guides/comparison/
---

# JCL vs Other Configuration Languages

This guide compares JCL with popular configuration languages: JSON, YAML, TOML, and HCL. We'll examine the same configuration in each format and highlight JCL's advantages.

## Table of Contents

- [Overview](#overview)
- [JCL vs JSON](#jcl-vs-json)
- [JCL vs YAML](#jcl-vs-yaml)
- [JCL vs TOML](#jcl-vs-toml)
- [JCL vs HCL](#jcl-vs-hcl)
- [Feature Comparison Matrix](#feature-comparison-matrix)
- [Migration Guide](#migration-guide)

---

## Overview

JCL (Jack-of-All Configuration Language) combines the best features from existing configuration languages while addressing their common pain points:

- **Simple & readable** like JSON
- **Powerful** like HCL
- **Feature-rich** with 70+ built-in functions
- **Type-safe** with optional type annotations
- **Developer-friendly** with LSP support and tooling

---

## JCL vs JSON

### The Problem with JSON

JSON is widely used but has significant limitations for configuration:

1. **No comments** - Can't document your configuration
2. **Strict syntax** - Trailing commas cause errors
3. **Limited types** - No date/time types, no way to express durations
4. **No variables** - Repetition everywhere
5. **No functions** - Can't compute values
6. **Verbose** - Excessive quotes and brackets

### Side-by-Side Example

**JSON:**
```json
{
  "name": "web-server",
  "version": "1.0.0",
  "environment": "production",
  "server": {
    "host": "0.0.0.0",
    "port": 80,
    "workers": 4,
    "timeout": 30
  },
  "database": {
    "host": "db.example.com",
    "port": 5432,
    "name": "web-server_production",
    "ssl": true
  },
  "features": {
    "auth": true,
    "api": true,
    "debug": false
  },
  "cors_origins": [
    "https://example.com",
    "https://www.example.com"
  ]
}
```

**JCL:**
```jcl
# Application Configuration
name = "web-server"
version = "1.0.0"
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
    name = "${name}_${environment}",  # Computed from variables
    ssl = environment == "production"  # Conditional value
)

# Feature flags with computed values
features = (
    auth = true,
    api = true,
    debug = environment != "production"  # Automatically false in prod
)

cors_origins = [
    "https://example.com",
    "https://www.example.com"
]
```

### JCL Advantages over JSON

✅ **Comments** - Document your configuration inline  
✅ **String interpolation** - `"${name}_${environment}"` instead of repetition  
✅ **Conditionals** - `if environment == "production" then 80 else 8080`  
✅ **No trailing comma issues** - Commas are flexible  
✅ **Less verbose** - No quotes on keys, cleaner syntax  
✅ **Computed values** - `environment != "production"`  
✅ **70+ built-in functions** - Hash, encode, transform data

### Migration from JSON

```bash
# Convert JSON to JCL
jcl-migrate config.json -o config.jcl

# Or to stdout
jcl-migrate config.json
```

---

## JCL vs YAML

### The Problem with YAML

YAML is popular but notorious for its issues:

1. **Indentation hell** - Whitespace is syntax, hard to debug
2. **Implicit types** - `no` becomes `false`, `1.0` might be a string
3. **Complex spec** - Anchors, aliases, merge keys are confusing
4. **Slow parsing** - YAML parsers are complex and slow
5. **Security issues** - YAML can execute code if not careful
6. **Tab vs space issues** - Mixing tabs/spaces breaks everything

### Side-by-Side Example

**YAML:**
```yaml
name: web-server
version: 1.0.0
environment: production

server:
  host: 0.0.0.0
  port: 80  # Must manually change for dev
  workers: 4
  timeout: 30

database:
  host: db.example.com
  port: 5432
  name: web-server_production  # Manually duplicated
  ssl: true  # Must manually change for dev

features:
  auth: true
  api: true
  debug: false  # Must manually change for dev

cors_origins:
  - https://example.com
  - https://www.example.com

# With anchors (complex and hard to read)
defaults: &defaults
  timeout: 30
  retry: 3

service_a:
  <<: *defaults
  name: service-a

service_b:
  <<: *defaults
  name: service-b
```

**JCL:**
```jcl
name = "web-server"
version = "1.0.0"
environment = "production"

server = (
    host = "0.0.0.0",
    port = if environment == "production" then 80 else 8080,  # Automatic
    workers = 4,
    timeout = 30
)

database = (
    host = "db.example.com",
    port = 5432,
    name = "${name}_${environment}",  # No duplication
    ssl = environment == "production"  # Automatic
)

features = (
    auth = true,
    api = true,
    debug = environment != "production"  # Automatic
)

cors_origins = [
    "https://example.com",
    "https://www.example.com"
]

# Reusable values with functions (clear and simple)
fn default_service(name) = (
    name = name,
    timeout = 30,
    retry = 3
)

service_a = default_service("service-a")
service_b = default_service("service-b")
```

### JCL Advantages over YAML

✅ **No indentation issues** - Uses explicit delimiters `()` and `[]`  
✅ **Explicit types** - No surprising type coercions  
✅ **Simpler spec** - Easy to learn, no anchors/aliases needed  
✅ **Fast parsing** - Rust-based parser is 5-10x faster  
✅ **Safe by default** - No code execution vulnerabilities  
✅ **Functions instead of anchors** - More powerful and readable  
✅ **Tab/space agnostic** - Whitespace is not syntax

### Migration from YAML

```bash
# Convert YAML to JCL
jcl-migrate config.yaml -o config.jcl

# Or to stdout
jcl-migrate config.yaml
```

---

## JCL vs TOML

### The Problem with TOML

TOML is designed for simplicity but has limitations:

1. **Nested structures are verbose** - Deep nesting gets ugly fast
2. **No functions** - Can't compute values
3. **No conditionals** - Can't adapt based on environment
4. **Limited expressions** - No string interpolation
5. **Inconsistent syntax** - Tables, inline tables, arrays of tables

### Side-by-Side Example

**TOML:**
```toml
name = "web-server"
version = "1.0.0"
environment = "production"

[server]
host = "0.0.0.0"
port = 80  # Must manually change for dev
workers = 4
timeout = 30

[database]
host = "db.example.com"
port = 5432
name = "web-server_production"  # Manually duplicated
ssl = true  # Must manually change for dev

[features]
auth = true
api = true
debug = false  # Must manually change for dev

cors_origins = [
    "https://example.com",
    "https://www.example.com"
]

# Deeply nested structures get verbose
[app.services.api.config.auth]
enabled = true
provider = "oauth2"

[app.services.api.config.auth.oauth2]
client_id = "abc123"
client_secret = "secret"
```

**JCL:**
```jcl
name = "web-server"
version = "1.0.0"
environment = "production"

server = (
    host = "0.0.0.0",
    port = if environment == "production" then 80 else 8080,
    workers = 4,
    timeout = 30
)

database = (
    host = "db.example.com",
    port = 5432,
    name = "${name}_${environment}",
    ssl = environment == "production"
)

features = (
    auth = true,
    api = true,
    debug = environment != "production"
)

cors_origins = [
    "https://example.com",
    "https://www.example.com"
]

# Deeply nested structures stay clean
app = (
    services = (
        api = (
            config = (
                auth = (
                    enabled = true,
                    provider = "oauth2",
                    oauth2 = (
                        client_id = "abc123",
                        client_secret = "secret"
                    )
                )
            )
        )
    )
)
```

### JCL Advantages over TOML

✅ **Cleaner nested structures** - No `[table.subtable.deep.nested]` syntax  
✅ **String interpolation** - `"${name}_${environment}"`  
✅ **Conditionals** - `if environment == "production" then 80 else 8080`  
✅ **Functions** - Transform and compute values  
✅ **Consistent syntax** - One way to define objects  
✅ **More flexible** - Lists of objects are natural

### Migration from TOML

```bash
# Convert TOML to JCL
jcl-migrate config.toml -o config.jcl

# Or to stdout
jcl-migrate config.toml
```

---

## JCL vs HCL

### The Relationship with HCL

HCL (HashiCorp Configuration Language) is one of the most powerful configuration languages, used by Terraform. JCL shares some philosophical similarities with HCL but differs in key ways.

### Side-by-Side Example

**HCL (Terraform):**
```hcl
variable "environment" {
  default = "production"
}

resource "aws_instance" "web" {
  ami           = "ami-123456"
  instance_type = var.environment == "production" ? "t3.large" : "t3.micro"

  tags = {
    Name        = "web-server"
    Environment = var.environment
  }
}

locals {
  db_name = "web-server_${var.environment}"
  port    = var.environment == "production" ? 80 : 8080
}

output "database_name" {
  value = local.db_name
}
```

**JCL:**
```jcl
environment = "production"

aws_instance_web = (
    ami = "ami-123456",
    instance_type = if environment == "production" then "t3.large" else "t3.micro",
    tags = (
        Name = "web-server",
        Environment = environment
    )
)

# Locals are just regular variables
db_name = "web-server_${environment}"
port = if environment == "production" then 80 else 8080

# Outputs are just values
database_name = db_name
```

### Differences from HCL

| Feature | HCL | JCL |
|---------|-----|-----|
| **Purpose** | Infrastructure-as-Code | General-purpose configuration |
| **Blocks** | `resource "type" "name" { }` | Maps with keys: `resource = ( ... )` |
| **Variables** | `var.name` | Direct: `name` |
| **Locals** | `local.name` | Direct: `name` |
| **Functions** | ~100 built-in, Terraform-specific | 70+ built-in, general-purpose |
| **Learning curve** | Steeper (blocks, modules, state) | Gentler (simpler concepts) |
| **Use cases** | Terraform, infrastructure | Any application configuration |

### JCL Advantages over HCL

✅ **Simpler syntax** - No `resource`/`data`/`variable` blocks  
✅ **General purpose** - Not tied to infrastructure tools  
✅ **Consistent** - Everything is a variable or expression  
✅ **No special variable syntax** - Use `name` instead of `var.name`  
✅ **Easier to learn** - Fewer concepts to master  
✅ **More flexible** - Not opinionated about structure

### When to Use HCL vs JCL

**Use HCL when:**
- You're using Terraform or other HashiCorp tools
- You need infrastructure-specific features (state, providers)
- You're managing cloud resources

**Use JCL when:**
- You need general application configuration
- You want simpler syntax for config files
- You're not using Terraform
- You want faster parsing and evaluation
- You need better tooling (LSP, formatters, validators)

---

## Feature Comparison Matrix

| Feature | JCL | JSON | YAML | TOML | HCL |
|---------|-----|------|------|------|-----|
| **Comments** | ✅ | ❌ | ✅ | ✅ | ✅ |
| **String interpolation** | ✅ | ❌ | ❌ | ❌ | ✅ |
| **Conditionals** | ✅ | ❌ | ❌ | ❌ | ✅ |
| **Functions** | ✅ (70+) | ❌ | ❌ | ❌ | ✅ (100+) |
| **For loops** | ✅ | ❌ | ❌ | ❌ | ✅ |
| **Type annotations** | ✅ | ❌ | ❌ | ❌ | ❌ |
| **Schema validation** | ✅ | ✅ | ✅ | ❌ | ❌ |
| **LSP support** | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Fast parsing** | ✅ | ✅ | ❌ | ✅ | ✅ |
| **No indentation** | ✅ | ✅ | ❌ | ✅ | ✅ |
| **Learning curve** | Low | Very Low | Low | Low | Medium |
| **Code formatter** | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Migration tools** | ✅ | N/A | N/A | N/A | ❌ |

---

## Migration Guide

JCL provides tools to migrate from other formats seamlessly.

### From JSON

```bash
# Single file
jcl-migrate config.json -o config.jcl

# Multiple files
for file in *.json; do
  jcl-migrate "$file" -o "${file%.json}.jcl"
done

# Format the results
jcl-fmt *.jcl
```

### From YAML

```bash
# Single file
jcl-migrate config.yaml -o config.jcl

# Multiple files
for file in *.yaml *.yml; do
  jcl-migrate "$file" -o "${file%.*}.jcl"
done

# Format the results
jcl-fmt *.jcl
```

### From TOML

```bash
# Single file
jcl-migrate config.toml -o config.jcl

# Multiple files
for file in *.toml; do
  jcl-migrate "$file" -o "${file%.toml}.jcl"
done

# Format the results
jcl-fmt *.jcl
```

### Manual Enhancement After Migration

After migration, you can enhance your configs with JCL features:

**Before (migrated from JSON):**
```jcl
environment = "production"
port = 80
debug = false
```

**After (enhanced with JCL features):**
```jcl
environment = "production"
port = if environment == "production" then 80 else 8080
debug = environment != "production"
```

### Validation After Migration

```bash
# Validate against a schema
jcl-validate config.jcl --schema schema.yaml

# Check formatting
jcl-fmt --check config.jcl

# Test evaluation
jcl eval config.jcl
```

---

## Real-World Examples

### Example 1: Application Configuration

**YAML (old):**
```yaml
app:
  name: myapp
  env: production

database:
  host: db.prod.example.com
  port: 5432
  name: myapp_production

redis:
  host: redis.prod.example.com
  port: 6379
```

**JCL (new):**
```jcl
env = "production"
app_name = "myapp"

app = (
    name = app_name,
    env = env
)

database = (
    host = "db.${env}.example.com",
    port = 5432,
    name = "${app_name}_${env}"
)

redis = (
    host = "redis.${env}.example.com",
    port = 6379
)
```

### Example 2: Feature Flags

**JSON (old):**
```json
{
  "features": {
    "new_ui": true,
    "beta_api": false,
    "analytics": true,
    "debug_mode": false,
    "rate_limiting": true
  }
}
```

**JCL (new):**
```jcl
env = "production"
beta_users = ["user123", "user456"]
current_user = "user789"

features = (
    new_ui = true,
    beta_api = contains(beta_users, current_user),
    analytics = true,
    debug_mode = env != "production",
    rate_limiting = env == "production"
)
```

### Example 3: Multi-Environment Config

**TOML (old - needs 3 separate files):**
```toml
# config.production.toml
[server]
port = 80
workers = 8

[database]
host = "db.prod.example.com"
pool_size = 50
```

**JCL (new - one file for all environments):**
```jcl
environment = "production"  # Change this one line

server = (
    port = if environment == "production" then 80 else 8080,
    workers = if environment == "production" then 8 else 2
)

database = (
    host = "db.${environment}.example.com",
    pool_size = if environment == "production" then 50 else 10
)
```

---

## Summary

JCL combines the best features of existing configuration languages:

- **Simpler than YAML** - No indentation issues
- **More powerful than JSON** - Comments, functions, conditionals
- **More flexible than TOML** - Better nested structures
- **More accessible than HCL** - General-purpose, easier to learn

### Why Choose JCL?

1. **Productivity** - Write less, express more
2. **Safety** - Type annotations and schema validation
3. **Tooling** - LSP, formatters, validators, benchmarks
4. **Migration** - Easy to migrate from existing formats
5. **Performance** - Fast Rust-based implementation
6. **Flexibility** - Use it for any configuration need

### Getting Started

```bash
# Install JCL
cargo install jcl

# Migrate your configs
jcl-migrate old-config.json -o config.jcl

# Format them
jcl-fmt config.jcl

# Start using JCL!
jcl eval config.jcl
```

[Learn more →](../getting-started/index.html)
