# JCL Language Showcase v1.0
# Demonstrates all features of the finalized language spec

## ============================================================================
## BASIC ASSIGNMENTS
## ============================================================================

# Simple assignments (immutable by default)
app_name = "myapp"
app_version = "2.1.0"
port = 8080
enabled = true

# With type annotations
max_connections: int = 1000
api_url: string = "https://api.example.com"

# Mutable variables
mut counter = 0
counter = counter + 1  # OK, counter is mutable

## ============================================================================
## COLLECTIONS (NEW SYNTAX)
## ============================================================================

# Lists use square brackets []
servers = ["web-1", "web-2", "api-1", "db-1"]
ports = [80, 443, 8080, 9000]
regions = ["us-west-2", "us-east-1", "eu-west-1"]

# Empty list
empty_list = []

# Maps use parentheses ()
config = (
  host = "localhost"
  port = 5432
  ssl = true
  timeout = 30
)

# Map with colon syntax (alternative)
settings = (
  debug: false
  log_level: "info"
  max_retries: 3
)

# Nested collections
app_config = (
  name = app_name
  version = app_version
  database = (
    host = "db.local"
    port = 5432
    name = "appdb"
  )
  cache = (
    host = "redis.local"
    port = 6379
  )
  servers = servers
  regions = regions
)

# Empty map
empty_map = ()

## ============================================================================
## STRING INTERPOLATION
## ============================================================================

# Simple interpolation
greeting = "Hello, ${app_name}!"
message = "Version ${app_version} running on port ${port}"

# With expressions
calculation = "Result: ${2 + 2}"
conditional = "Server is ${enabled ? "enabled" : "disabled"}"

# Nested paths
db_url = "postgresql://${config.host}:${config.port}/${app_config.database.name}"
cache_url = "redis://${app_config.cache.host}:${app_config.cache.port}"

# Multi-line strings with interpolation
deploy_script = """
#!/bin/bash
set -euo pipefail

APP_NAME="${app_name}"
VERSION="${app_version}"
PORT=${port}

echo "Deploying $APP_NAME version $VERSION on port $PORT"
"""

## ============================================================================
## OPERATORS
## ============================================================================

# Arithmetic
sum = 10 + 20
difference = 100 - 25
product = 5 * 4
quotient = 100 / 5
remainder = 17 % 5

# Comparison
is_equal = app_version == "2.1.0"
is_greater = port > 8000
is_less_equal = counter <= 100

# Logical
is_production = enabled and port == 443
should_debug = not enabled or app_config.settings.debug
fallback = enabled or false

# Null coalescing (??)
primary_host = null
secondary_host = "backup.local"
actual_host = primary_host ?? secondary_host ?? "localhost"

# Optional chaining (?.)
optional_config = (
  database = (
    host = "db.local"
  )
)

# Safe navigation - returns null if any part is null
host = optional_config?.database?.host ?? "default-host"
port_value = optional_config?.database?.port ?? 5432

## ============================================================================
## FUNCTIONS
## ============================================================================

# Using built-in functions
uppercased = upper(app_name)
lowercased = lower("HELLO")
trimmed = trim("  spaces  ")

# Encoding
config_json = jsonencode(config)
config_yaml = yamlencode(config)
config_toml = tomlencode(config)

# Collections
server_count = length(servers)
sorted_servers = sort(servers)
first_server = servers[0]

# Lambda functions
double = x => x * 2
add = (x, y) => x + y

# Named functions
fn square(x) = x * x

fn process_name(name) = (
  cleaned = trim(name)
  lowered = lower(cleaned)
  upper(lowered)
)

fn get_env_config(env: string): map = when env (
  "prod" => (type = "t3.large", replicas = 3)
  "staging" => (type = "t3.medium", replicas = 2)
  "dev" => (type = "t3.small", replicas = 1)
  * => (type = "t3.micro", replicas = 1)
)

# Using defined functions
squared = square(10)
processed = process_name("  HeLLo  ")
prod_config = get_env_config("prod")

## ============================================================================
## CONDITIONALS
## ============================================================================

# Ternary operator
instance_type = enabled ? "t3.large" : "t3.small"
protocol = port == 443 ? "https" : "http"

# If expression
result = if counter > 10
  then "large"
  else "small"

# When expression (pattern matching)
status = when app_config.settings.log_level (
  "debug" => "verbose"
  "info" => "normal"
  "warn" => "quiet"
  "error" => "silent"
  * => "unknown"
)

# When with tuple patterns
size = when (app_name, enabled) (
  ("myapp", true) => "large"
  ("myapp", false) => "small"
  (*, true) => "medium"
  * => "tiny"
)

# When with guards
category = when counter (
  n if n < 0 => "negative"
  n if n == 0 => "zero"
  n if n > 0 and n < 10 => "small"
  n if n >= 10 => "large"
)

## ============================================================================
## ITERATION
## ============================================================================

# For loop over list
for server in servers (
  resource.${server} = (
    type = "t3.medium"
    monitoring = true
  )
)

# For loop over map
for key, value in config (
  env_var.${upper(key)} = tostring(value)
)

# For loop with index
for i, server in enumerate(servers) (
  numbered.${i} = (
    index = i
    name = server
    primary = i == 0
  )
)

# Multi-dimensional for loop (Cartesian product)
environments = ["dev", "staging", "prod"]
services = ["web", "api", "worker"]

for env in environments, service in services (
  deployment.${env}.${service} = (
    replicas = when env (
      "prod" => 3
      "staging" => 2
      * => 1
    )
    size = when (env, service) (
      ("prod", "api") => "t3.xlarge"
      ("prod", *) => "t3.large"
      ("staging", *) => "t3.medium"
      * => "t3.small"
    )
  )
)

# For loop with range
for i in range(5) (
  server.${i} = (
    name = "server-${i}"
    port = 8000 + i
  )
)

## ============================================================================
## LIST COMPREHENSIONS
## ============================================================================

numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

# Basic comprehension
doubled = [n * 2 for n in numbers]

# With filter
evens = [n for n in numbers if n % 2 == 0]

# With transformation
server_names = [upper(s) for s in servers]

# Complex transformation
formatted_servers = [
  "Server: ${s} (${i})"
  for i, s in enumerate(servers)
]

## ============================================================================
## PIPELINE OPERATOR
## ============================================================================

# Chain operations with |
result = servers
  | filter s => contains(s, "web")
  | map s => upper(s)
  | sort
  | join ", "

# Complex pipeline
data = ["  hello  ", "  WORLD  ", "  test  "]
processed_data = data
  | map s => trim(s)
  | map s => lower(s)
  | filter s => length(s) > 3
  | sort

# Pipeline with built-in functions
numbers_pipeline = [1, 2, 3, 4, 5]
  | map x => x * 2
  | filter x => x > 5
  | sum

## ============================================================================
## IMPORTS (HYBRID FEATURE)
## ============================================================================

# Import specific values from another file
import (common_tags, default_ports) from "./common.jcl"

# Import all with namespace
import * as common from "./common.jcl"

# Import with alias
import (common_tags as tags) from "./common.jcl"

# Use imported values
my_tags = merge(common_tags, (app = app_name))
http_port = default_ports.http

## ============================================================================
## ERROR HANDLING
## ============================================================================

# Fail fast (default) - error stops evaluation
# config_data = file("/path/to/config.json")

# Try with fallback
config_data = try(file("/path/to/config.json"), "default config")

# With null coalescing
parsed_config = try(jsondecode(config_data), null) ?? (default = true)

# Safe file operations
file_content = try(file("optional.txt"), null)
has_file = file_content != null

## ============================================================================
## OUTPUTS
## ============================================================================

# Simple outputs
out.app_name = app_name
out.version = app_version
out.config = config

# Computed outputs
out.server_list = servers | join ", "
out.server_count = length(servers)

# Complex outputs
out.deployment_summary = (
  application = app_name
  version = app_version
  environments = length(environments)
  services = length(services)
  total_deployments = length(environments) * length(services)
  config = app_config
)

# Generated script output
out.deploy_sh = """
#!/bin/bash
set -euo pipefail

APP="${app_name}"
VERSION="${app_version}"

${[
  "echo \"Deploying to ${s}\"\nssh ${s} 'docker pull $APP:$VERSION && docker restart $APP'"
  for s in servers
] | join "\n\n"}

echo "Deployment complete!"
"""

# Multi-format outputs
out.config_json = jsonencode(app_config)
out.config_yaml = yamlencode(app_config)
out.config_toml = tomlencode(app_config)

## ============================================================================
## SUMMARY
## ============================================================================

# This file demonstrates:
# ✅ [] for lists, () for maps
# ✅ String interpolation with ${}
# ✅ ?. and ?? operators
# ✅ Lambda and named functions
# ✅ Import system
# ✅ Try/catch with fallbacks
# ✅ All control flow constructs
# ✅ Pipeline operator
# ✅ List comprehensions
# ✅ For loops with multiple patterns
# ✅ Type annotations (optional)
# ✅ Mutable variables with mut
# ✅ Comments
