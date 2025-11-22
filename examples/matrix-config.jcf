# Matrix/Cartesian Product Configuration Examples
# Defining configurations across multiple dimensions

## Example 1: Simple Matrix with For Loop

resources = (web, db)
stacks = (dev, prod)

# Define configurations for all combinations
for resource in resources, stack in stacks (
  ${stack}.${resource}.instance.size = when (stack, resource) (
    (prod, web) => t2.medium
    (prod, db) => t2.large
    (dev, web) => t2.micro
    (dev, db) => t2.small
  )

  ${stack}.${resource}.instance.count = stack == prod ? 3 : 1

  ${stack}.${resource}.tags = (
    name = "${resource}-${stack}"
    environment = stack
    resource_type = resource
  )
)

## Example 2: Matrix with Explicit Declaration

# Declare the matrix
matrix config_matrix (
  resources = (web, api, worker, db)
  environments = (dev, staging, prod)
)

# Define patterns for the matrix
config_matrix.web.*.instance_type = env => when env (
  prod => t3.large
  staging => t3.medium
  dev => t3.small
)

config_matrix.db.*.instance_type = env => when env (
  prod => db.r5.large
  staging => db.t3.medium
  dev => db.t3.micro
)

config_matrix.*.*.backup_enabled = (env, res) => env == prod

## Example 3: Table-style Configuration

# Define a configuration table
instance_sizes = table (
  # Headers: resource, dev, staging, prod
  (web, t2.micro, t2.small, t2.medium)
  (api, t2.micro, t2.medium, t2.large)
  (worker, t2.micro, t2.small, t2.medium)
  (db, t2.small, t2.medium, t2.large)
)

# Access table values
dev.web.size = instance_sizes[web][dev]
prod.api.size = instance_sizes[api][prod]

## Example 4: Using Zipmap for Matrix

resources = (web, api, worker, db)
environments = (dev, staging, prod)

# Create all combinations
combinations = cartesian(resources, environments)  # [(web,dev), (web,staging), ...]

# Define configuration for each combination
for combo in combinations (
  resource = combo[0]
  env = combo[1]

  ${env}.${resource}.config = (
    instance_type = lookup_size(resource, env)
    count = env == prod ? 3 : 1
    monitoring = env == prod
  )
)

## Example 5: Nested Configuration with Defaults

# Define base configuration
base_config = (
  monitoring = false
  backup = false
  replicas = 1
  instance_type = t2.micro
)

# Override per environment
env_overrides = (
  prod = (
    monitoring = true
    backup = true
    replicas = 3
    instance_type = t2.large
  )
  staging = (
    monitoring = true
    replicas = 2
    instance_type = t2.medium
  )
  dev = ()  # Use all defaults
)

# Override per resource type
resource_overrides = (
  db = (
    backup = true
    instance_type = db.t3.medium
  )
  web = (
    instance_type = t2.small
  )
)

# Build final configurations
for resource in (web, api, db), env in (dev, staging, prod) (
  ${env}.${resource}.config = base_config
    | merge(env_overrides[env] or ())
    | merge(resource_overrides[resource] or ())
)

## Example 6: Compact Matrix Syntax

# Ultra-compact table definition
config = (
  # resource -> (dev, staging, prod)
  web.size = (t2.micro, t2.small, t2.medium)
  web.count = (1, 2, 3)

  api.size = (t2.micro, t2.medium, t2.large)
  api.count = (1, 2, 3)

  db.size = (t2.small, t2.medium, t2.large)
  db.count = (1, 1, 2)
  db.backup = (false, true, true)
)

# Access with index
dev.web.size = config.web.size[0]      # t2.micro
staging.api.size = config.api.size[1]  # t2.medium
prod.db.size = config.db.size[2]       # t2.large

## Example 7: Conditional Matrix Expansion

resources = (web, api, worker, db, cache)
environments = (dev, staging, prod)

# Define rules for which resources exist in which environments
resource_matrix = (
  web = (dev, staging, prod)      # All environments
  api = (dev, staging, prod)      # All environments
  worker = (staging, prod)        # Not in dev
  db = (dev, staging, prod)       # All environments
  cache = (prod)                  # Only prod
)

# Generate configs only for valid combinations
for resource in resources (
  for env in resource_matrix[resource] (
    ${env}.${resource}.enabled = true

    ${env}.${resource}.instance = (
      type = default_size(resource, env)
      count = env == prod ? 3 : 1
    )
  )
)

## Example 8: Complex Multi-dimensional Configuration

# Three dimensions: region, environment, resource
regions = (us-west-2, us-east-1, eu-west-1)
environments = (dev, staging, prod)
resources = (web, api, db)

# Define configuration for all combinations
for region in regions, env in environments, resource in resources (
  config.${region}.${env}.${resource} = (
    instance_type = size_for(resource, env)
    availability_zones = azs_for(region)
    count = env == prod ? 3 : 1

    tags = (
      region = region
      environment = env
      resource = resource
      name = "${resource}-${env}-${region}"
    )
  )
)

# Access specific configuration
us_west_prod_web = config.us-west-2.prod.web

## Example 9: Using Functions for Matrix Generation

# Helper function to generate instance size based on resource and environment
fn size_for(resource, environment) = when (resource, environment) (
  (web, prod) => t3.large
  (web, staging) => t3.medium
  (web, dev) => t3.small

  (api, prod) => t3.xlarge
  (api, staging) => t3.large
  (api, dev) => t3.medium

  (db, prod) => db.r5.xlarge
  (db, staging) => db.t3.large
  (db, dev) => db.t3.medium

  * => t3.micro  # Default
)

# Generate matrix using function
for resource in (web, api, db), env in (dev, staging, prod) (
  ${env}.${resource}.instance.type = size_for(resource, env)
)

## Example 10: Outputs from Matrix

# Generate outputs for all combinations
out.all_configs = [
  (
    env = env
    resource = resource
    config = ${env}.${resource}.config
  )
  for resource in resources, env in environments
]

# Filter outputs
out.prod_resources = [
  cfg
  for cfg in out.all_configs
  if cfg.env == prod
]

# Format as map
out.config_map = zipmap(
  [
    "${r}-${e}"
    for r in resources, e in environments
  ],
  [
    ${e}.${r}.config
    for r in resources, e in environments
  ]
)
