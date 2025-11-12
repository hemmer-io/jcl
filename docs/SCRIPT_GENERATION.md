# Script Generation in JCL

JCL can be used as a powerful configuration source for generating scripts in various languages (Bash, Python, PowerShell, etc.). This allows you to maintain configuration in JCL and generate language-specific scripts from it.

## Table of Contents

- [Why Generate Scripts from JCL?](#why-generate-scripts-from-jcl)
- [Basic Concepts](#basic-concepts)
- [Generating Bash Scripts](#generating-bash-scripts)
- [Generating Python Scripts](#generating-python-scripts)
- [Generating PowerShell Scripts](#generating-powershell-scripts)
- [Generating SQL Scripts](#generating-sql-scripts)
- [Generating Dockerfiles](#generating-dockerfiles)
- [Template-based Generation](#template-based-generation)
- [Advanced Patterns](#advanced-patterns)

## Why Generate Scripts from JCL?

1. **Single Source of Truth**: Maintain config in one place, generate scripts for different targets
2. **Type Safety**: JCL validates your configuration before generating scripts
3. **DRY Principle**: Define once, use in multiple script formats
4. **Consistency**: Ensure all generated scripts use the same configuration
5. **Templating**: Use JCL's powerful functions to transform data for scripts

## Basic Concepts

### Template Rendering

JCL provides `template()` and `templatefile()` functions for rendering scripts:

```
# Data in JCL
config = (
  app_name = myapp
  version = 1.2.3
  port = 8080
)

# Generate bash script
deploy_script = template("""
#!/bin/bash
set -euo pipefail

APP_NAME="{{app_name}}"
VERSION="{{version}}"
PORT={{port}}

echo "Deploying $APP_NAME version $VERSION on port $PORT"
docker run -d -p $PORT:$PORT $APP_NAME:$VERSION
""", config)

# Write to file
out.deploy_sh = deploy_script
```

### String Building

Build scripts programmatically using string operations:

```
servers = (web-1, web-2, api-1)

# Generate bash array
bash_array = "SERVERS=(" + join(servers, " ") + ")"
# Result: SERVERS=(web-1 web-2 api-1)
```

## Generating Bash Scripts

### Simple Bash Script

```
# Configuration
servers = (
  (name = web-1, ip = 10.0.1.10, port = 8080),
  (name = web-2, ip = 10.0.1.11, port = 8080),
  (name = api-1, ip = 10.0.2.10, port = 9000)
)

# Generate server list
server_defs = [
  "declare -A ${s.name}=([ip]=\"${s.ip}\" [port]=\"${s.port}\")"
  for s in servers
] | join "\n"

# Generate deployment script
bash_script = """
#!/bin/bash
set -euo pipefail

# Server definitions
${server_defs}

# Deploy function
deploy_to_server() {
  local server_name=$1
  local server_var="${server_name}[ip]"
  local port_var="${server_name}[port]"

  echo "Deploying to ${!server_var}:${!port_var}"
  ssh ${!server_var} "docker pull myapp:latest && docker restart myapp"
}

# Deploy to all servers
${[for s in servers: "deploy_to_server ${s.name}"] | join "\n"}
"""

out.deploy_script = bash_script
```

### Bash with Environment Variables

```
environments = (dev, staging, prod)

env_configs = (
  dev = (
    api_url = "https://dev-api.example.com"
    db_host = "dev-db.internal"
    log_level = "debug"
  )
  staging = (
    api_url = "https://staging-api.example.com"
    db_host = "staging-db.internal"
    log_level = "info"
  )
  prod = (
    api_url = "https://api.example.com"
    db_host = "prod-db.internal"
    log_level = "warn"
  )
)

# Generate env file for each environment
for env in environments (
  env_vars = [
    "${upper(k)}=\"${v}\""
    for k, v in env_configs[env]
  ] | join "\n"

  out.${env}_env_sh = """
#!/bin/bash
# Environment: ${env}

${env_vars}

export API_URL DB_HOST LOG_LEVEL
"""
)
```

### Bash Installation Script

```
packages = (
  (name = nginx, version = "1.20"),
  (name = nodejs, version = "18.x"),
  (name = postgresql, version = "14"),
  (name = redis, version = "latest")
)

install_commands = [
  when pkg.version (
    "latest" => "apt-get install -y ${pkg.name}"
    * => "apt-get install -y ${pkg.name}=${pkg.version}"
  )
  for pkg in packages
] | join "\n  "

install_script = template("""
#!/bin/bash
set -euo pipefail

echo "Installing packages..."

# Update package list
apt-get update

# Install packages
{{install_commands}}

echo "Installation complete!"
""", (install_commands = install_commands))

out.install_sh = install_script
```

## Generating Python Scripts

### Python Deployment Script

```
servers = (
  (host = "web-1.example.com", port = 22, user = "deploy"),
  (host = "web-2.example.com", port = 22, user = "deploy"),
  (host = "api-1.example.com", port = 22, user = "deploy")
)

app_config = (
  name = "myapp"
  version = "1.2.3"
  container_port = 8080
)

# Generate Python list
server_list = jsonencode([
  (host = s.host, port = s.port, user = s.user)
  for s in servers
])

python_script = template("""
#!/usr/bin/env python3
import subprocess
import sys
import json

SERVERS = {{server_list}}
APP_NAME = "{{app_config.name}}"
VERSION = "{{app_config.version}}"
PORT = {{app_config.container_port}}

def deploy_to_server(server):
    host = server['host']
    user = server['user']
    port = server['port']

    print(f"Deploying {APP_NAME}:{VERSION} to {host}...")

    commands = [
        f"docker pull {APP_NAME}:{VERSION}",
        f"docker stop {APP_NAME} || true",
        f"docker rm {APP_NAME} || true",
        f"docker run -d --name {APP_NAME} -p {PORT}:{PORT} {APP_NAME}:{VERSION}"
    ]

    for cmd in commands:
        ssh_cmd = f"ssh -p {port} {user}@{host} '{cmd}'"
        result = subprocess.run(ssh_cmd, shell=True, capture_output=True)
        if result.returncode != 0:
            print(f"Error: {result.stderr.decode()}", file=sys.stderr)
            return False

    print(f"Successfully deployed to {host}")
    return True

def main():
    success = True
    for server in SERVERS:
        if not deploy_to_server(server):
            success = False

    return 0 if success else 1

if __name__ == "__main__":
    sys.exit(main())
""", (
  server_list = server_list
  app_config = app_config
))

out.deploy_py = python_script
```

### Python Configuration Generator

```
database_config = (
  host = "db.internal"
  port = 5432
  database = "myapp"
  user = "appuser"
  pool_size = 20
  timeout = 30
)

cache_config = (
  host = "redis.internal"
  port = 6379
  db = 0
  ttl = 3600
)

python_config = template("""
#!/usr/bin/env python3
\"\"\"Auto-generated configuration - DO NOT EDIT\"\"\"

DATABASE = {
    {{[for k, v in database_config: "'${k}': ${jsonencode(v)},"] | join "\n    "}}
}

CACHE = {
    {{[for k, v in cache_config: "'${k}': ${jsonencode(v)},"] | join "\n    "}}
}

# Validation
def validate_config():
    assert DATABASE['port'] > 0, "Database port must be positive"
    assert CACHE['ttl'] > 0, "Cache TTL must be positive"
    print("Configuration is valid!")

if __name__ == "__main__":
    validate_config()
""", (
  database_config = database_config
  cache_config = cache_config
))

out.config_py = python_config
```

## Generating PowerShell Scripts

### PowerShell Deployment

```
services = (
  (name = "WebService", path = "C:\\Services\\Web", port = 80),
  (name = "APIService", path = "C:\\Services\\API", port = 443),
  (name = "WorkerService", path = "C:\\Services\\Worker", port = 8080)
)

powershell_script = """
# Auto-generated PowerShell deployment script

$ErrorActionPreference = "Stop"

${[for s in services: """
Write-Host "Starting ${s.name}..."
$service = Get-Service -Name "${s.name}" -ErrorAction SilentlyContinue
if ($service) {
    Restart-Service "${s.name}"
} else {
    New-Service -Name "${s.name}" -BinaryPathName "${s.path}\\${s.name}.exe" -StartupType Automatic
    Start-Service "${s.name}"
}
Write-Host "${s.name} is running on port ${s.port}"
"""] | join "\n"}

Write-Host "All services deployed successfully!"
"""

out.deploy_ps1 = powershell_script
```

## Generating SQL Scripts

### SQL Schema Generation

```
tables = (
  (
    name = "users"
    columns = (
      (name = "id", type = "SERIAL", primary_key = true),
      (name = "email", type = "VARCHAR(255)", unique = true),
      (name = "name", type = "VARCHAR(255)"),
      (name = "created_at", type = "TIMESTAMP", default = "NOW()")
    )
  ),
  (
    name = "posts"
    columns = (
      (name = "id", type = "SERIAL", primary_key = true),
      (name = "user_id", type = "INTEGER", references = "users(id)"),
      (name = "title", type = "VARCHAR(255)"),
      (name = "content", type = "TEXT"),
      (name = "created_at", type = "TIMESTAMP", default = "NOW()")
    )
  )
)

# Generate CREATE TABLE statements
create_tables = [
  """
CREATE TABLE IF NOT EXISTS ${t.name} (
  ${[
    col.name + " " + col.type +
    (col.primary_key ? " PRIMARY KEY" : "") +
    (col.unique ? " UNIQUE" : "") +
    (col.default ? " DEFAULT " + col.default : "") +
    (col.references ? " REFERENCES " + col.references : "")
    for col in t.columns
  ] | join ",\n  "}
);
"""
  for t in tables
] | join "\n"

out.schema_sql = create_tables
```

### SQL Data Migration

```
users = (
  (id = 1, email = "alice@example.com", name = "Alice"),
  (id = 2, email = "bob@example.com", name = "Bob"),
  (id = 3, email = "charlie@example.com", name = "Charlie")
)

insert_statements = [
  "INSERT INTO users (id, email, name) VALUES (${u.id}, '${u.email}', '${u.name}');"
  for u in users
] | join "\n"

migration_sql = """
-- Auto-generated migration
BEGIN;

${insert_statements}

COMMIT;
"""

out.migration_sql = migration_sql
```

## Generating Dockerfiles

### Dockerfile from Config

```
app_config = (
  base_image = "node:18-alpine"
  workdir = "/app"
  port = 3000
  dependencies = ("package*.json")
  source_files = ("src", "public", "config")
  build_command = "npm run build"
  start_command = "node dist/index.js"
  env_vars = (
    NODE_ENV = "production"
    PORT = "3000"
  )
)

dockerfile = """
FROM ${app_config.base_image}

WORKDIR ${app_config.workdir}

# Copy dependencies
${[for f in app_config.dependencies: "COPY ${f} ."] | join "\n"}

# Install dependencies
RUN npm ci --only=production

# Copy source
${[for d in app_config.source_files: "COPY ${d} ${d}"] | join "\n"}

# Build
RUN ${app_config.build_command}

# Environment
${[for k, v in app_config.env_vars: "ENV ${k}=${v}"] | join "\n"}

EXPOSE ${app_config.port}

CMD ["${app_config.start_command}"]
"""

out.Dockerfile = dockerfile
```

## Template-based Generation

### Using External Templates

```
# config.jcl
servers = (
  (name = "web-1", ip = "10.0.1.10"),
  (name = "web-2", ip = "10.0.1.11")
)

# Generate from template file
deploy_script = templatefile("deploy.sh.tpl", (
  servers = servers
  version = "1.2.3"
))

out.deploy_sh = deploy_script
```

**deploy.sh.tpl:**
```bash
#!/bin/bash
set -euo pipefail

VERSION="{{version}}"

{{#each servers}}
echo "Deploying to {{name}} ({{ip}})"
ssh {{ip}} "docker pull myapp:$VERSION && docker restart myapp"
{{/each}}

echo "Deployment complete!"
```

## Advanced Patterns

### Multi-format Output

Generate the same configuration in multiple formats:

```
config = (
  database = (
    host = "localhost"
    port = 5432
    name = "myapp"
  )
  cache = (
    host = "localhost"
    port = 6379
  )
)

# As Bash env vars
out.config_sh = """
export DB_HOST="${config.database.host}"
export DB_PORT=${config.database.port}
export DB_NAME="${config.database.name}"
export CACHE_HOST="${config.cache.host}"
export CACHE_PORT=${config.cache.port}
"""

# As Python
out.config_py = """
DATABASE = ${jsonencode(config.database)}
CACHE = ${jsonencode(config.cache)}
"""

# As JSON
out.config_json = jsonencode(config)

# As YAML
out.config_yaml = yamlencode(config)

# As TOML
out.config_toml = tomlencode(config)
```

### Conditional Script Generation

```
environments = (dev, prod)
enable_monitoring = true

for env in environments (
  monitoring_setup = env == prod or enable_monitoring ? """
  # Setup monitoring
  apt-get install -y prometheus-node-exporter
  systemctl enable prometheus-node-exporter
  systemctl start prometheus-node-exporter
  """ : "# Monitoring disabled"

  out.setup_${env}_sh = """
#!/bin/bash
set -euo pipefail

echo "Setting up ${env} environment..."

# Install base packages
apt-get update
apt-get install -y nginx postgresql

${monitoring_setup}

echo "Setup complete!"
"""
)
```

### Script with Embedded Data

```
config_data = (
  servers = ("web-1", "web-2", "api-1")
  ports = (8080, 8081, 9000)
  settings = (timeout = 30, retries = 3)
)

# Embed entire config as base64-encoded JSON
embedded_config = base64encode(jsonencode(config_data))

script_with_data = """
#!/bin/bash
# Script with embedded configuration

CONFIG_B64="${embedded_config}"

# Decode config
CONFIG=$(echo "$CONFIG_B64" | base64 -d)

# Use jq to parse
SERVERS=$(echo "$CONFIG" | jq -r '.servers[]')

for server in $SERVERS; do
  echo "Processing $server..."
done
"""

out.script_with_data_sh = script_with_data
```

## Best Practices

1. **Use Templates for Complex Scripts**: Template files are easier to maintain than string concatenation
2. **Validate Configuration**: Use JCL's type system to validate before generating
3. **Add Shebang**: Always include appropriate shebang (`#!/bin/bash`, `#!/usr/bin/env python3`)
4. **Set Error Handling**: Add `set -euo pipefail` for bash, try/except for Python
5. **Add Comments**: Generate comments explaining what the script does
6. **Version Your Templates**: Keep templates in version control
7. **Test Generated Scripts**: Always test generated scripts before using in production
8. **Escape Properly**: Be careful with special characters in generated code
9. **Use Language-Appropriate Encoding**: JSON for Python/JS, env vars for bash
10. **Document Generation**: Add comments showing this is auto-generated

## Summary

JCL's script generation capabilities allow you to:
- Maintain configuration in one place (JCL)
- Generate scripts for multiple languages/platforms
- Ensure consistency across deployments
- Leverage JCL's type safety and validation
- Use powerful templating and transformation functions

This makes JCL ideal as a "configuration compiler" that outputs executable scripts!
