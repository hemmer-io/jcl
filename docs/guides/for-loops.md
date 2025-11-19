---
layout: default
title: For Loops and Iteration
parent: Guides
nav_order: 3
---

JCL provides powerful iteration constructs for processing collections and generating configurations.

## Table of Contents

- [Basic For Loop](#basic-for-loop)
- [For-Each Over Collections](#for-each-over-collections)
- [Multi-dimensional Iteration](#multi-dimensional-iteration)
- [List Comprehensions](#list-comprehensions)
- [Map Comprehensions](#map-comprehensions)
- [Filtering During Iteration](#filtering-during-iteration)
- [Nested Loops](#nested-loops)
- [Iteration with Index](#iteration-with-index)
- [Pipeline Iteration](#pipeline-iteration)

## Basic For Loop

### Range-based Loop

```
# Iterate over a range
for i in range(5) (
  server.${i}.name = "server-${i}"
  server.${i}.port = 8000 + i
)

# Results in:
# server.0.name = "server-0", server.0.port = 8000
# server.1.name = "server-1", server.1.port = 8001
# ... etc
```

### Range with Start and End

```
# Custom range
for i in range(10, 15) (
  port.${i} = (
    enabled = true
    protocol = tcp
  )
)

# Results in ports 10, 11, 12, 13, 14
```

## For-Each Over Collections

### Iterate Over List

```
servers = (web, api, worker, db)

for server in servers (
  ${server}.config = (
    type = server
    monitoring = true
    backup = server == db
  )
)

# Results in:
# web.config = (type=server monitoring=true backup=false)
# api.config = (type=server monitoring=true backup=false)
# worker.config = (type=server monitoring=true backup=false)
# db.config = (type=server monitoring=true backup=true)
```

### Iterate Over Map

```
ports = (
  http = 80
  https = 443
  ssh = 22
)

for name, port in ports (
  firewall.rule.${name} = (
    port = port
    protocol = tcp
    allow = true
  )
)

# Results in:
# firewall.rule.http = (port=80 protocol=tcp allow=true)
# firewall.rule.https = (port=443 protocol=tcp allow=true)
# firewall.rule.ssh = (port=22 protocol=tcp allow=true)
```

## Multi-dimensional Iteration

### Cartesian Product (Multiple For Variables)

```
resources = (web, api, db)
environments = (dev, staging, prod)

# All combinations
for resource in resources, env in environments (
  config.${env}.${resource} = (
    name = "${resource}-${env}"
    size = env == prod ? large : small
  )
)

# Results in 9 combinations:
# config.dev.web, config.dev.api, config.dev.db,
# config.staging.web, config.staging.api, config.staging.db,
# config.prod.web, config.prod.api, config.prod.db
```

### Three-dimensional Loop

```
regions = (us-west-2, us-east-1, eu-west-1)
environments = (dev, prod)
services = (web, api)

for region in regions, env in environments, service in services (
  config.${region}.${env}.${service} = (
    endpoint = "${service}.${env}.${region}.example.com"
  )
)

# Results in 2 × 2 × 3 = 12 configurations
```

## List Comprehensions

### Basic List Comprehension

```
numbers = (1, 2, 3, 4, 5)

# Double each number
doubled = [n * 2 for n in numbers]
# Result: [2, 4, 6, 8, 10]

# Square each number
squared = [n * n for n in numbers]
# Result: [1, 4, 9, 16, 25]
```

### With Transformation

```
servers = (web-1, web-2, api-1, db-1)

# Extract server types
types = [split(s, "-")[0] for s in servers]
# Result: ["web", "web", "api", "db"]

# Create URLs
urls = ["https://${s}.example.com" for s in servers]
# Result: ["https://web-1.example.com", ...]
```

## Map Comprehensions

### Create Map from List

```
servers = (web, api, db)

# Create config map
config = {
  s: (type = s, enabled = true)
  for s in servers
}

# Result:
# {
#   web: (type=web enabled=true),
#   api: (type=api enabled=true),
#   db: (type=db enabled=true)
# }
```

### Transform Map

```
prices = (small = 10, medium = 20, large = 40)

# Calculate with tax
with_tax = {
  k: v * 1.08
  for k, v in prices
}

# Result: {small: 10.8, medium: 21.6, large: 43.2}
```

## Filtering During Iteration

### Filter in List Comprehension

```
numbers = (1, 2, 3, 4, 5, 6, 7, 8, 9, 10)

# Only even numbers
evens = [n for n in numbers if n % 2 == 0]
# Result: [2, 4, 6, 8, 10]

# Only numbers greater than 5
large = [n for n in numbers if n > 5]
# Result: [6, 7, 8, 9, 10]
```

### Filter with Transformation

```
servers = (web-1, web-2, api-1, db-1, cache-1)

# Only web servers, extract number
web_numbers = [
  tonumber(split(s, "-")[1])
  for s in servers
  if contains(s, "web")
]
# Result: [1, 2]
```

### If-Else in Comprehension

```
numbers = (1, 2, 3, 4, 5)

# Convert to strings with label
labeled = [
  n % 2 == 0 ? "even-${n}" : "odd-${n}"
  for n in numbers
]
# Result: ["odd-1", "even-2", "odd-3", "even-4", "odd-5"]
```

## Nested Loops

### Nested For Loop

```
clusters = (east, west)
nodes = (1, 2, 3)

for cluster in clusters (
  for node in nodes (
    server.${cluster}.${node} = (
      name = "${cluster}-node-${node}"
      cluster = cluster
    )
  )
)
```

### Nested List Comprehension

```
# Generate multiplication table
table = [
  [i * j for j in range(1, 11)]
  for i in range(1, 11)
]

# Flatten nested structure
grid = (
  (a, b, c),
  (d, e, f),
  (g, h, i)
)

flat = [cell for row in grid for cell in row]
# Result: [a, b, c, d, e, f, g, h, i]
```

## Iteration with Index

### Enumerate Pattern

```
servers = (web, api, db)

config = [
  (index = i, name = s, port = 8000 + i)
  for i, s in enumerate(servers)
]

# Result:
# [
#   (index=0 name=web port=8000),
#   (index=1 name=api port=8001),
#   (index=2 name=db port=8002)
# ]
```

### Index-based Configuration

```
servers = (web, api, worker, db)

for i in range(length(servers)) (
  server.${i} = (
    name = servers[i]
    id = i
    primary = i == 0
  )
)
```

## Pipeline Iteration

### Pipeline with Map

```
servers = (web-1, web-2, api-1, db-1)

# Pipeline style
formatted = servers
  | filter s => contains(s, "web")
  | map s => upper(s)
  | sort

# Result: ["WEB-1", "WEB-2"]
```

### Complex Pipeline

```
data = (
  (name = "alice", age = 30, dept = "eng"),
  (name = "bob", age = 25, dept = "sales"),
  (name = "charlie", age = 35, dept = "eng")
)

# Filter engineers, extract names, uppercase
eng_names = data
  | filter person => person.dept == "eng"
  | map person => person.name
  | map name => upper(name)
  | sort

# Result: ["ALICE", "CHARLIE"]
```

## Advanced Patterns

### Conditional Loop Execution

```
environments = (dev, staging, prod)
enable_monitoring = true

for env in environments (
  if enable_monitoring or env == prod (
    monitoring.${env}.enabled = true
  )
)
```

### Loop with Break/Continue (Simulated)

```
# Skip certain iterations
for i in range(10) (
  # Skip even numbers
  if i % 2 == 0 continue

  # Only process up to 7
  if i > 7 break

  config.${i} = (value = i)
)

# Results in: config.1, config.3, config.5, config.7
```

### Accumulation Pattern

```
numbers = (1, 2, 3, 4, 5)

# Cumulative sum
running_sum = [
  sum(numbers[0:i+1])
  for i in range(length(numbers))
]

# Result: [1, 3, 6, 10, 15]
```

## Real-World Examples

### Generate Server Configurations

```
regions = (us-west-2, us-east-1)
environments = (dev, staging, prod)
services = (web, api, worker)

for region in regions, env in environments, service in services (
  deployment.${region}.${env}.${service} = (
    replicas = env == prod ? 3 : 1
    instance_type = when (env, service) (
      (prod, api) => c5.xlarge
      (prod, *) => t3.large
      (staging, *) => t3.medium
      (dev, *) => t3.small
    )

    tags = (
      environment = env
      service = service
      region = region
      name = "${service}-${env}-${region}"
    )

    monitoring = env != dev
    backup = service == db and env != dev
  )
)
```

### Generate Firewall Rules

```
allowed_ports = (
  (name = http, port = 80, public = true),
  (name = https, port = 443, public = true),
  (name = ssh, port = 22, public = false),
  (name = app, port = 8080, public = false)
)

internal_ips = (10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16)

for rule in allowed_ports (
  firewall.ingress.${rule.name} = (
    port = rule.port
    protocol = tcp
    allow_from = rule.public ? "0.0.0.0/0" : internal_ips
    description = "Allow ${rule.name} traffic"
  )
)
```

### Generate DNS Records

```
environments = (dev, staging, prod)
services = (www, api, admin, cdn)

records = [
  (
    name = "${service}.${env == prod ? "" : "${env}."}example.com"
    type = CNAME
    value = "${service}-${env}.elb.amazonaws.com"
  )
  for env in environments, service in services
]

# Flatten and create records
for record in records (
  dns.${record.name} = (
    type = record.type
    value = record.value
    ttl = 300
  )
)
```

### Build Deployment Matrix

```
# Define what to deploy where
deployments = (
  (app = web, envs = (dev, staging, prod), regions = (us-west-2, us-east-1)),
  (app = api, envs = (dev, staging, prod), regions = (us-west-2)),
  (app = admin, envs = (staging, prod), regions = (us-west-2)),
  (app = worker, envs = (prod), regions = (us-west-2, eu-west-1))
)

# Generate all deployment configs
all_deployments = flatten([
  [
    (app = d.app, env = env, region = region)
    for env in d.envs, region in d.regions
  ]
  for d in deployments
])

# Create configurations
for deploy in all_deployments (
  config.${deploy.region}.${deploy.env}.${deploy.app} = (
    deployed = true
    endpoint = "${deploy.app}.${deploy.env}.${deploy.region}.example.com"
  )
)
```

## Performance Tips

1. **Use comprehensions for simple transformations** - More concise than for loops
2. **Filter early** - Apply filters before expensive operations
3. **Avoid nested loops when possible** - Use cartesian() for multi-dimensional iteration
4. **Use pipelines for readability** - Chain operations for clarity
5. **Break complex iterations** - Split into multiple steps for maintainability

## Summary

JCL provides multiple ways to iterate:
- **For loops**: Traditional iteration with side effects
- **List comprehensions**: Functional-style list generation
- **Map comprehensions**: Create maps from iteration
- **Pipelines**: Chain transformations
- **Multi-dimensional**: Cartesian products for matrices

Choose the pattern that best fits your use case!
