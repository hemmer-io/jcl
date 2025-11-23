# For-Each Examples in JCL
# JCL already supports for-each - this shows all the patterns

## Example 1: For-Each Over Simple List

servers = (web-1, web-2, web-3, api-1, api-2)

# For-each creates a resource for each item
for server in servers (
  resource.${server} = (
    ami = ami-12345
    instance_type = t3.medium
    tags (name = server)
  )
)

# Result: 5 resources created (web-1, web-2, web-3, api-1, api-2)

## Example 2: For-Each Over Map

# Define configuration per resource
server_configs = (
  web = (type = t3.medium, disk = 50, public = true)
  api = (type = t3.large, disk = 100, public = true)
  db = (type = r5.xlarge, disk = 500, public = false)
  cache = (type = cache.r5.large, disk = 20, public = false)
)

# For-each over map (key, value pairs)
for name, config in server_configs (
  resource.${name} = (
    instance_type = config.type
    root_block_device (
      volume_size = config.disk
    )
    associate_public_ip_address = config.public

    tags (
      name = name
      type = name
    )
  )
)

## Example 3: For-Each with Index

environments = (dev, staging, prod)

# Get both index and value
for i, env in enumerate(environments) (
  resource.${env} = (
    instance_type = t3.small
    priority = i  # 0, 1, 2
    tags (
      name = env
      order = tostring(i)
    )
  )
)

## Example 4: For-Each with Filtering

all_servers = (
  (name = web-1, role = web, zone = a),
  (name = web-2, role = web, zone = b),
  (name = api-1, role = api, zone = a),
  (name = db-1, role = database, zone = a),
  (name = cache-1, role = cache, zone = b)
)

# Only create resources for specific role
for server in all_servers (
  if server.role == "web" or server.role == "api" (
    resource.${server.name} = (
      ami = ami-12345
      instance_type = t3.medium
      availability_zone = "us-west-2${server.zone}"

      tags (
        name = server.name
        role = server.role
      )
    )
  )
)

# Result: Only web-1, web-2, api-1 created (db and cache skipped)

## Example 5: For-Each Over Cartesian Product

regions = (us-west-2, us-east-1, eu-west-1)
environments = (dev, prod)

# Create resource for each combination
for region in regions, env in environments (
  resource.${region}_${env} = (
    region = region
    instance_type = env == prod ? t3.large : t3.small

    tags (
      region = region
      environment = env
      name = "${region}-${env}"
    )
  )
)

# Result: 6 resources (3 regions × 2 envs)

## Example 6: For-Each Creating Multiple Resource Types

applications = (
  (name = web, has_lb = true, has_db = false),
  (name = api, has_lb = true, has_db = true),
  (name = worker, has_lb = false, has_db = true)
)

for app in applications (
  # Always create instance
  resource.instance_${app.name} = (
    ami = ami-12345
    instance_type = t3.medium
    tags (name = "${app.name}-instance")
  )

  # Conditionally create load balancer
  if app.has_lb (
    resource.lb_${app.name} = (
      type = application
      tags (name = "${app.name}-lb")
    )
  )

  # Conditionally create database
  if app.has_db (
    resource.db_${app.name} = (
      engine = postgresql
      instance_class = db.t3.medium
      tags (name = "${app.name}-db")
    )
  )
)

## Example 7: For-Each with Transformation

# Start with simple list
availability_zones = (a, b, c)

# Transform into full AZ names and create subnets
for i, az in enumerate(availability_zones) (
  resource.subnet_${az} = (
    availability_zone = "us-west-2${az}"
    cidr_block = "10.0.${i}.0/24"

    tags (
      name = "subnet-${az}"
      az = az
    )
  )
)

## Example 8: For-Each with Nested Data

departments = (
  (
    name = engineering
    teams = (frontend, backend, devops)
  ),
  (
    name = sales
    teams = (enterprise, smb)
  ),
  (
    name = support
    teams = (tier1, tier2, tier3)
  )
)

# Create resources for each team in each department
for dept in departments (
  for team in dept.teams (
    resource.${dept.name}_${team} = (
      tags (
        department = dept.name
        team = team
        full_name = "${dept.name}-${team}"
      )
    )
  )
)

## Example 9: For-Each with Set Operations

# Define sets
web_servers = (web-1, web-2, web-3)
api_servers = (api-1, api-2)
all_servers = web_servers + api_servers  # Concatenate

# Create monitoring for each server
for server in all_servers (
  resource.monitor_${server} = (
    target = server
    enabled = true
  )
)

## Example 10: For-Each with Complex Logic

services = (
  (name = web, port = 80, ssl = true, workers = 4),
  (name = api, port = 8080, ssl = true, workers = 8),
  (name = admin, port = 9000, ssl = false, workers = 2)
)

for service in services (
  # Security group
  resource.sg_${service.name} = (
    name = "${service.name}-sg"

    ingress (
      from_port = service.port
      to_port = service.port
      protocol = tcp
      cidr_blocks = ["0.0.0.0/0"]
    )

    # Add HTTPS if SSL enabled
    if service.ssl (
      ingress (
        from_port = 443
        to_port = 443
        protocol = tcp
        cidr_blocks = ["0.0.0.0/0"]
      )
    )
  )

  # Application configuration
  resource.config_${service.name} = (
    port = service.port
    workers = service.workers
    ssl_enabled = service.ssl
    ssl_cert = service.ssl ? "/etc/ssl/${service.name}.crt" : null
  )
)

## Example 11: For-Each with Resource Dependencies

# Create databases first
databases = (users, products, orders)

for db in databases (
  resource.database_${db} = (
    engine = postgresql
    name = db
    tags (name = "db-${db}")
  )
)

# Then create applications that depend on databases
applications = (
  (name = user-service, database = users),
  (name = product-service, database = products),
  (name = order-service, database = orders)
)

for app in applications (
  resource.${app.name} = (
    database_endpoint = resource.database_${app.database}.endpoint

    # Explicit dependency
    depends_on = [resource.database_${app.database}]

    tags (name = app.name)
  )
)

## Example 12: For-Each with Outputs

environments = (dev, staging, prod)

for env in environments (
  resource.instance_${env} = (
    ami = ami-12345
    instance_type = env == prod ? t3.large : t3.small
  )
)

# Generate outputs for each
for env in environments (
  out.${env}_instance_id = resource.instance_${env}.id
  out.${env}_public_ip = resource.instance_${env}.public_ip
  out.${env}_endpoint = "https://${env}.example.com"
)

## Example 13: For-Each with List Comprehension

# Create ports list
ports = [8000 + i for i in range(5)]  # [8000, 8001, 8002, 8003, 8004]

# Create security group rule for each port
for port in ports (
  resource.allow_${port} = (
    type = ingress
    from_port = port
    to_port = port
    protocol = tcp
    cidr_blocks = ["0.0.0.0/0"]
  )
)

## Example 14: For-Each with Conditional Resource Creation

features = (
  (name = monitoring, enabled = true),
  (name = logging, enabled = true),
  (name = tracing, enabled = false),
  (name = metrics, enabled = true)
)

# Only create resources for enabled features
for feature in features (
  if feature.enabled (
    resource.${feature.name} = (
      enabled = true
      name = feature.name
      tags (feature = feature.name)
    )
  )
)

# Result: Only monitoring, logging, and metrics created (tracing skipped)

## Example 15: For-Each with Map and Filter Pipeline

servers = (
  (name = web-1, status = running, healthy = true),
  (name = web-2, status = running, healthy = false),
  (name = api-1, status = stopped, healthy = true),
  (name = db-1, status = running, healthy = true)
)

# Get only running and healthy servers
healthy_servers = servers
  | filter s => s.status == "running"
  | filter s => s.healthy
  | map s => s.name

# Create backups only for healthy servers
for server in healthy_servers (
  resource.backup_${server} = (
    source = server
    schedule = "daily"
  )
)

# Result: backups for web-1 and db-1 only

## Summary

JCL's for-each is powerful because:
1. Works with lists, maps, and cartesian products
2. Supports filtering and transformation
3. Can iterate with index (enumerate)
4. Allows nested loops
5. Integrates with list/map comprehensions
6. Supports conditional resource creation
7. Can reference other resources
8. Works with pipelines and filters

The key difference from Terraform's for_each:
- JCL: `for item in items (...)` - Imperative style, more flexible
- Terraform: `for_each = var.items` - Declarative, more constrained

JCL's approach is more like Ansible's loops but with more power!
