# Matrix/Cartesian Product Configurations in JCL

JCL provides powerful features for defining configurations across multiple dimensions (environments, resources, regions, etc.) without repeating yourself.

## Basic Concepts

When you have multiple dimensions (e.g., resources and environments), you often need to define configurations for all combinations. JCL provides several ways to handle this elegantly.

## Syntax Options

### 1. For Loop with Multiple Iterators

```
resources = (web, db, cache)
environments = (dev, prod)

# Cartesian product: all combinations
for resource in resources, env in environments (
  ${env}.${resource}.instance.size = when (env, resource) (
    (prod, web) => t2.medium
    (prod, db) => t2.large
    (dev, web) => t2.micro
    (dev, db) => t2.small
    * => t2.micro
  )

  ${env}.${resource}.count = env == prod ? 3 : 1
)

# Results in:
# prod.web.instance.size = t2.medium
# prod.db.instance.size = t2.large
# prod.cache.instance.size = t2.micro  (from default)
# dev.web.instance.size = t2.micro
# dev.db.instance.size = t2.small
# dev.cache.instance.size = t2.micro
```

### 2. Pattern Matching with Wildcards

```
matrix config_matrix (
  resources = (web, api, db)
  environments = (dev, staging, prod)
)

# Define pattern for all combinations
config_matrix.*.*.enabled = true

# Define pattern for specific resource across all envs
config_matrix.web.*.instance_type = t2.medium

# Define pattern for specific env across all resources
config_matrix.*.prod.monitoring = true

# Define for specific combination
config_matrix.db.prod.instance_type = db.r5.xlarge
```

### 3. Table-Style Configuration

```
# Define a configuration table
instance_types = table (
  # headers: resource, dev, staging, prod
  (web,     t2.micro,  t2.small,   t2.medium)
  (api,     t2.micro,  t2.medium,  t2.large)
  (worker,  t2.micro,  t2.small,   t2.medium)
  (db,      t2.small,  t2.medium,  t2.large)
)

# Access values
dev.web.type = instance_types[web][dev]         # t2.micro
prod.api.type = instance_types[api][prod]       # t2.large

# Or with function call
instance_types.get(web, staging)  # t2.small
```

### 4. Compact Tuple Syntax

```
# Define configs as tuples indexed by environment
config = (
  web.size = (t2.micro, t2.small, t2.medium)  # (dev, staging, prod)
  api.size = (t2.micro, t2.medium, t2.large)
  db.size = (t2.small, t2.medium, t2.large)
)

# Access with index (0=dev, 1=staging, 2=prod)
dev.web.size = config.web.size[0]
staging.api.size = config.api.size[1]
prod.db.size = config.db.size[2]
```

### 5. Merging Defaults with Overrides

```
# Base configuration
base = (
  instance_type = t2.micro
  monitoring = false
  backup = false
  count = 1
)

# Environment-specific overrides
env_config = (
  prod = (monitoring = true backup = true count = 3 instance_type = t2.large)
  staging = (monitoring = true count = 2 instance_type = t2.medium)
  dev = ()
)

# Resource-specific overrides
resource_config = (
  db = (backup = true instance_type = db.t3.medium)
  cache = (backup = false instance_type = cache.t3.small)
  web = ()
)

# Build final configs for all combinations
for resource in (web, api, db, cache), env in (dev, staging, prod) (
  ${env}.${resource}.config = base
    | merge(env_config[env] or ())
    | merge(resource_config[resource] or ())
    | merge((name = "${resource}-${env}"))
)
```

## Advanced Patterns

### Multi-Dimensional Matrices

```
regions = (us-west-2, us-east-1, eu-west-1)
environments = (dev, staging, prod)
resources = (web, api, db)

# Three-dimensional matrix
for region in regions, env in environments, resource in resources (
  config.${region}.${env}.${resource} = (
    ami = ami_for(region, resource)
    instance_type = size_for(resource, env)
    az_count = region == us-west-2 ? 3 : 2
    replicas = env == prod ? 3 : 1
  )
)
```

### Conditional Matrix Expansion

```
# Not all resources exist in all environments
resource_envs = (
  web = (dev, staging, prod)
  api = (dev, staging, prod)
  admin = (staging, prod)      # No dev
  analytics = (prod)            # Only prod
)

# Only generate configs for valid combinations
for resource in keys(resource_envs) (
  for env in resource_envs[resource] (
    ${env}.${resource}.config = (...)
  )
)
```

### Helper Functions for Matrices

```
# Define helper function
fn instance_size(resource, env) = when (resource, env) (
  (web, prod) => t3.large
  (web, staging) => t3.medium
  (web, dev) => t3.small

  (db, prod) => db.r5.xlarge
  (db, *) => db.t3.large

  (*, prod) => t3.large
  (*, staging) => t3.medium
  (*, dev) => t3.small
)

# Use in matrix generation
for resource in (web, api, db), env in (dev, staging, prod) (
  ${env}.${resource}.size = instance_size(resource, env)
)
```

### Cartesian Product Function

```
# Built-in cartesian product function
combos = cartesian((web, api, db), (dev, staging, prod))
# Result: [(web,dev), (web,staging), (web,prod), (api,dev), ...]

# Use with destructuring
for combo in combos (
  resource = combo[0]
  env = combo[1]

  ${env}.${resource}.config = (...)
)
```

### Matrix with Filters

```
# Generate matrix but skip certain combinations
for resource in (web, api, db, cache), env in (dev, staging, prod) (
  # Skip cache in dev
  if (resource == cache and env == dev) continue

  # Skip db in dev (use shared dev db)
  if (resource == db and env == dev) continue

  ${env}.${resource}.config = (...)
)
```

## Real-World Example

```
# Multi-region, multi-environment application

regions = (us-west-2, us-east-1, eu-west-1)
environments = (dev, staging, prod)
services = (web, api, worker, db, cache)

# Base configuration
base_config = (
  monitoring = false
  backup = false
  replicas = 1
  instance_type = t3.micro
)

# Environment overrides
env_overrides = (
  prod = (monitoring = true backup = true replicas = 3)
  staging = (monitoring = true replicas = 2)
  dev = ()
)

# Service-specific overrides
service_overrides = (
  db = (
    backup = true
    instance_type = db.t3.medium
    storage = 100
  )
  cache = (
    instance_type = cache.t3.small
    persistence = false
  )
  web = (
    public = true
  )
)

# Region-specific settings
region_settings = (
  us-west-2 = (az_count = 3 primary = true)
  us-east-1 = (az_count = 3 primary = false)
  eu-west-1 = (az_count = 2 primary = false)
)

# Generate all configurations
for region in regions, env in environments, service in services (
  # Skip certain combinations
  if (service == cache and env == dev) continue

  config.${region}.${env}.${service} = base_config
    | merge(env_overrides[env] or ())
    | merge(service_overrides[service] or ())
    | merge(region_settings[region])
    | merge((
        name = "${service}-${env}-${region}"
        region = region
        environment = env
        service = service
      ))
)

# Access specific configuration
web_prod_west = config.us-west-2.prod.web

# Generate outputs by environment
for env in environments (
  out.${env}_services = [
    service
    for region in regions, service in services
    if exists(config.${region}.${env}.${service})
  ]
)

# Generate cost estimates
total_instances = sum([
  config.${r}.${e}.${s}.replicas
  for r in regions, e in environments, s in services
  if exists(config.${r}.${e}.${s})
])
```

## Built-in Functions for Matrix Operations

### cartesian(list1, list2, ...) -> list
Creates Cartesian product of multiple lists.

```
cartesian((a, b), (1, 2))  # [(a,1), (a,2), (b,1), (b,2)]
```

### table(rows...) -> table
Creates a table data structure.

```
t = table(
  (web, small, medium, large),
  (db, tiny, small, medium)
)

t[web][medium]  # Access cell
t.get(db, small)  # Alternative access
```

### matrix(dimensions...) -> matrix
Creates a matrix structure.

```
m = matrix(
  resources = (web, api, db)
  environments = (dev, prod)
)

# Access and set
m.web.prod = (...)
```

### combinations(list, n) -> list
Generate all n-length combinations.

```
combinations((a, b, c), 2)  # [(a,b), (a,c), (b,c)]
```

### permutations(list, n) -> list
Generate all n-length permutations.

```
permutations((a, b, c), 2)  # [(a,b), (a,c), (b,a), (b,c), (c,a), (c,b)]
```

## Tips and Best Practices

1. **Use defaults and overrides**: Define base configuration and selectively override
2. **Helper functions**: Extract complex logic into reusable functions
3. **Filter combinations**: Skip invalid/unused combinations early
4. **Name consistently**: Use predictable naming patterns for generated configs
5. **Document matrices**: Comment on what dimensions represent
6. **Test outputs**: Generate output summaries to verify matrix expansion
