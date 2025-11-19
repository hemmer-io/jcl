---
layout: default
title: Templating Guide
parent: Guides
nav_order: 6
---

JCL provides powerful templating capabilities through its existing features, without requiring a separate template syntax like Jinja2.

## Why No Separate Template Syntax?

JCL's design philosophy is to keep the language unified and composable. Instead of adding {% raw %}`{% if %}...{% endif %}`{% endraw %} blocks, we leverage:

1. **String interpolation** with `${...}` - embeds any expression
2. **Conditional expressions** - ternary, if/then/else, when
3. **List comprehensions** - generate repeated sections
4. **Multi-line strings** - support complex templates
5. **Pipeline operator** - transform and join data

This approach keeps templates readable and avoids context-switching between "template syntax" and "code syntax".

## Pattern 1: Conditional Content with Ternary

```jcl
env = "prod"

config_file = """
[database]
host = ${env == "prod" ? "db.prod.internal" : "localhost"}
port = ${env == "prod" ? 5432 : 5433}
ssl = ${env == "prod" ? "require" : "disable"}
pool_size = ${env == "prod" ? 20 : 5}
"""
```

## Pattern 2: Conditional Blocks with If Expression

```jcl
env = "prod"
enable_monitoring = true

nginx_config = """
server {
    listen 80;
    server_name example.com;

${if env == "prod" then """
    # Production settings
    worker_processes 4;
    worker_connections 2048;
""" else """
    # Development settings
    worker_processes 1;
    worker_connections 512;
"""}

${if enable_monitoring then """
    # Monitoring endpoint
    location /health {
        return 200 "OK";
    }
""" else ""}
}
"""
```

## Pattern 3: Conditional Blocks with When Expression

```jcl
env = "prod"

database_config = """
[database]
${when env (
  "prod" => """
host = db.prod.internal
port = 5432
ssl = require
max_connections = 100
"""
  "staging" => """
host = db.staging.internal
port = 5432
ssl = prefer
max_connections = 50
"""
  "dev" => """
host = localhost
port = 5433
ssl = disable
max_connections = 10
"""
  * => "# No database configuration"
)}
"""
```

## Pattern 4: Repeated Sections with List Comprehensions

```jcl
servers = [
  (name = "web-1", ip = "10.0.1.10", port = 8080),
  (name = "web-2", ip = "10.0.1.11", port = 8080),
  (name = "api-1", ip = "10.0.2.10", port = 9000)
]

# Generate nginx upstream config
upstream_config = """
upstream backend {
${[
  "    server ${s.ip}:${s.port}; # ${s.name}"
  for s in servers
] | join "\n"}
}
"""

# Result:
# upstream backend {
#     server 10.0.1.10:8080; # web-1
#     server 10.0.1.11:8080; # web-2
#     server 10.0.2.10:9000; # api-1
# }
```

## Pattern 5: Filtered Repetition

```jcl
services = [
  (name = "web", enabled = true, port = 80),
  (name = "api", enabled = true, port = 8080),
  (name = "admin", enabled = false, port = 9000),
  (name = "metrics", enabled = true, port = 9090)
]

# Only include enabled services
docker_compose = """
version: '3'
services:
${[
  """
  ${s.name}:
    image: myapp/${s.name}
    ports:
      - "${s.port}:${s.port}"
"""
  for s in services if s.enabled
] | join ""}
"""

# Result: Only includes web, api, and metrics (admin skipped)
```

## Pattern 6: Nested Conditionals and Loops

```jcl
environments = ["dev", "staging", "prod"]
features = [
  (name = "database", prod_only = false),
  (name = "cache", prod_only = false),
  (name = "monitoring", prod_only = true),
  (name = "analytics", prod_only = true)
]

# Generate environment-specific configs
for env in environments (
  out."config_${env}" = """
# Configuration for ${env}

[features]
${[
  "${f.name} = ${if f.prod_only and env != "prod" then "disabled" else "enabled"}"
  for f in features
] | join "\n"}

[resources]
instance_type = ${when env (
  "prod" => "t3.large"
  "staging" => "t3.medium"
  * => "t3.small"
)}
"""
)
```

## Pattern 7: Complex Multi-Section Templates

```jcl
app_name = "myapp"
env = "prod"
enable_ssl = true
enable_monitoring = true
backends = [
  (host = "10.0.1.10", port = 8080),
  (host = "10.0.1.11", port = 8080)
]

nginx_full_config = """
# Generated nginx config for ${app_name}
# Environment: ${env}

${if enable_ssl then """
# SSL Configuration
ssl_certificate /etc/nginx/ssl/${app_name}.crt;
ssl_certificate_key /etc/nginx/ssl/${app_name}.key;
ssl_protocols TLSv1.2 TLSv1.3;
""" else "# SSL disabled"}

# Upstream servers
upstream ${app_name}_backend {
${[
  "    server ${b.host}:${b.port} max_fails=3 fail_timeout=30s;"
  for b in backends
] | join "\n"}
}

server {
    listen ${enable_ssl ? 443 : 80}${enable_ssl ? " ssl" : ""};
    server_name ${app_name}.example.com;

    location / {
        proxy_pass http://${app_name}_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

${if enable_monitoring then """
    location /health {
        access_log off;
        return 200 "healthy\\n";
    }

    location /metrics {
        stub_status;
    }
""" else ""}
}
"""
```

## Pattern 8: Environment-Specific Sections

```jcl
env = "prod"
debug = env != "prod"
log_level = when env (
  "prod" => "warn"
  "staging" => "info"
  * => "debug"
)

app_config = """
[app]
name = "myapp"
environment = ${env}

[logging]
level = ${log_level}
${if debug then """
# Debug mode enabled
debug = true
trace = true
verbose = true
""" else """
# Production mode
debug = false
"""
}

${if env == "prod" then """
[production]
# Production-only settings
high_availability = true
auto_scaling = true
backup_enabled = true
""" else ""}

${if env == "dev" then """
[development]
# Development-only settings
hot_reload = true
mock_external_apis = true
""" else ""}
"""
```

## Pattern 9: Using Template Function

For more complex templating needs, use the `template()` function:

```jcl
# template.tpl file
"""
Hello, {{name}}!
Your environment is: {{env}}
{{#if is_admin}}
You have admin privileges.
{{/if}}
"""

# JCL file
context = (
  name = "Alice"
  env = "production"
  is_admin = true
)

rendered = template("template.tpl", context)
```

## Pattern 10: Building Templates Programmatically

```jcl
# Define sections as variables
header = """
#!/bin/bash
set -euo pipefail
"""

deploy_function = """
deploy_to() {
    local server=$1
    echo "Deploying to $server"
    ssh "$server" 'docker pull myapp:latest && docker restart myapp'
}
"""

servers = ["web-1", "web-2", "api-1"]

deploy_commands = [
  """deploy_to "${s}" """
  for s in servers
] | join "\n"

footer = """
echo "Deployment complete!"
"""

# Compose final script
deploy_script = header + "\n" + deploy_function + "\n" + deploy_commands + "\n" + footer
```

## Comparison with Jinja2

**Jinja2 Style:**
{% raw %}
```jinja2
{% if env == "prod" %}
production config
{% else %}
dev config
{% endif %}

{% for server in servers %}
server {{ server.name }}
{% endfor %}
```
{% endraw %}

**JCL Style:**
```jcl
${if env == "prod" then """
production config
""" else """
dev config
"""}

${[
  "server ${s.name}"
  for s in servers
] | join "\n"}
```

## Why This Approach is Better

1. **No context switching** - Same expression syntax everywhere
2. **More composable** - Templates are just strings you can manipulate
3. **Type safe** - All expressions are type-checked
4. **Editor support** - Standard string interpolation, easier to highlight
5. **Simpler mental model** - One language, not two
6. **More powerful** - Can use any JCL expression, not just template constructs

## When to Use Each Pattern

- **Simple conditionals**: Use ternary `${condition ? "a" : "b"}`
- **Multi-line conditionals**: Use if expression with triple-quoted strings
- **Multiple cases**: Use when expression with triple-quoted strings
- **Repetition**: Use list comprehensions with join
- **Complex logic**: Build sections programmatically, then compose

## Best Practices

1. **Keep interpolations simple** - Complex logic should be computed first:
   ```jcl
   # Good
   ssl_mode = env == "prod" ? "require" : "disable"
   config = "ssl_mode = ${ssl_mode}"

   # Less good (but still works)
   config = "ssl_mode = ${env == "prod" ? "require" : "disable"}"
   ```

2. **Use multi-line strings for blocks**:
   ```jcl
   # Good - readable
   block = if condition then """
   line 1
   line 2
   """ else ""

   # Less good - hard to read
   block = if condition then "line 1\nline 2\n" else ""
   ```

3. **Extract repeated patterns**:
   ```jcl
   # Good - DRY
   fn server_config(name, ip) = """
   server {
     name = ${name}
     ip = ${ip}
   }
   """

   configs = [server_config(s.name, s.ip) for s in servers]
   ```

4. **Use functions for complex templates**:
   ```jcl
   fn nginx_upstream(name, servers) = """
   upstream ${name} {
   ${[
     "    server ${s.ip}:${s.port};"
     for s in servers
   ] | join "\n"}
   }
   """
   ```

## Summary

JCL provides powerful templating through composition of existing features:
- ✅ Conditional content: ternary, if, when expressions
- ✅ Repeated sections: list comprehensions
- ✅ String interpolation: `${...}` with any expression
- ✅ Multi-line strings: `"""`
- ✅ Data transformation: pipeline operator
- ✅ Reusable templates: functions
- ✅ Type safety: all expressions are checked
- ✅ Editor support: standard syntax highlighting

No need for {% raw %}`{% %}`{% endraw %} template syntax - JCL's unified design is more powerful and maintainable.
