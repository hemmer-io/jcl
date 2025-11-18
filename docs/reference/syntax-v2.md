# JCL - Human-Readable Syntax Design

## Philosophy
- Minimal punctuation
- Natural language flow
- Parentheses for grouping, not braces
- Less ceremony, more clarity
- No quotes unless necessary

## Core Syntax

### Environments

```
# Simple list
environments = (prod, dev, staging)

# With configuration
environment prod (
  region = us-west-2
  account = 123456789

  vars (
    instance_type = t3.large
    min_count = 3
    app_version = 1.2.3
  )

  tags (
    team = platform
    cost_center = engineering
  )
)
```

### Stacks

```
# Simple declaration
stacks = (networking, database, application)

# Stack with resources
stack application (
  env = prod
  depends_on = (networking, database)

  vars (
    port = 8080
    replicas = 3
  )
)
```

### Resources

```
# Concise resource syntax
resource aws_instance web (
  ami = ami-12345
  type = t3.medium
  count = 3

  tags (
    name = web-server
    env = prod
  )
)

# Even more concise - infer resource type
instance web (
  ami = ami-12345
  type = t3.medium

  # Configuration inline
  install (nginx, nodejs, git)

  file /etc/nginx/nginx.conf from template/nginx.conf

  service nginx start enabled
)
```

### Data Sources (Read-Only)

```
# Reference existing infrastructure
data vpc main (
  id = vpc-12345
  readonly = true
)

# Or even shorter
read vpc main (
  tags (name = main-vpc)
)

# Use the reference
instance app (
  vpc = vpc.main.id
  subnet = vpc.main.subnets[0]
)
```

### Variables

```
# Simple assignment
app_name = myapp
version = 1.2.3
port = 8080

# Typed variables
var instance_type: string = t3.medium
var count: int = 3
var enabled: bool = true

# Lists and maps without quotes
regions = (us-west-2, us-east-1, eu-west-1)
ports = (80, 443, 8080)

config = (
  host = localhost
  port = 5432
  ssl = true
)
```

### Conditionals

```
# Inline conditional
instance_type = prod ? t3.large : t3.small

# When expression
state = when env (
  prod => running
  staging => running
  * => stopped
)

# Pattern matching
action = match status (
  healthy => continue
  degraded => alert
  failed => restart
  * => monitor
)
```

### Iteration

```
# For each
for region in (us-west-2, us-east-1) (
  instance app-$region (
    region = $region
    type = t3.medium
  )
)

# Range
for i in 0..3 (
  instance worker-$i (
    type = t3.small
  )
)

# List comprehension
instance_ips = [ip for instance in servers]
active_servers = [s for s in servers if s.status == running]
```

### Functions

```
# Built-in functions (no parens for single arg)
upper app_name
lower region
trim input

# Multiple args need parens
replace(app_name, "-", "_")
join(regions, ",")
merge(base_tags, env_tags)

# Pipe operator
result = data
  | filter x => x > 10
  | map x => x * 2
  | sort
  | join ", "
```

## Complete Examples

### Example 1: Simple Web Server

```
# environments
environments = (prod, dev)

environment prod (
  region = us-west-2

  vars (
    instance_type = t3.large
    app_version = 1.2.3
  )
)

# infrastructure
stack web (
  env = prod

  # reference existing vpc (not managed)
  read vpc main (
    tags (name = main-vpc)
  )

  # create security group
  security_group web (
    vpc = vpc.main.id
    name = web-sg

    allow (
      port = 80
      protocol = tcp
      from = 0.0.0.0/0
    )

    allow (
      port = 443
      protocol = tcp
      from = 0.0.0.0/0
    )
  )

  # create instances
  instance web (
    ami = ami-12345
    type = env.vars.instance_type
    count = 3
    security_groups = (web)

    tags (
      name = web-server
      version = env.vars.app_version
    )

    # configuration management
    configure (
      # install packages
      install (nginx, nodejs)

      # manage files
      file /etc/nginx/nginx.conf (
        from = template/nginx.conf
        mode = 0644
        owner = root
      )

      # manage services
      service nginx (
        state = running
        enabled = true
      )
    )
  )

  # outputs
  output web_ips = instance.web.*.public_ip
  output web_urls = instance.web.*.public_ip | map ip => "http://$ip"
)
```

### Example 2: Full Application Stack

```
environments = (prod, staging, dev)

environment prod (
  region = us-west-2
  account = 123456789

  vars (
    app_name = myapp
    app_version = 2.1.0
    instance_type = t3.large
    min_instances = 3
    max_instances = 10
  )

  tags (
    team = platform
    cost_center = engineering
  )
)

# network stack (references existing)
stack network (
  env = prod

  read vpc main (id = vpc-12345)

  read subnets public (
    vpc = vpc.main.id
    tier = public
  )

  read subnets private (
    vpc = vpc.main.id
    tier = private
  )
)

# application stack
stack app (
  env = prod
  depends_on = (network)

  # security groups
  security_group alb (
    vpc = network.vpc.main.id
    name = $env.vars.app_name-alb-sg

    allow (80, 443) from 0.0.0.0/0
  )

  security_group app (
    vpc = network.vpc.main.id
    name = $env.vars.app_name-app-sg

    allow 3000 from alb
    allow 22 from 10.0.0.0/8
  )

  # load balancer
  load_balancer app (
    type = application
    subnets = network.subnets.public.*.id
    security_groups = (alb)

    listener http (
      port = 80
      forward_to = target_group.app
    )
  )

  target_group app (
    port = 3000
    protocol = http
    vpc = network.vpc.main.id

    health_check (
      path = /health
      interval = 30
      timeout = 5
    )
  )

  # auto scaling
  launch_template app (
    ami = ami-12345
    type = env.vars.instance_type
    security_groups = (app)
  )

  autoscaling_group app (
    min = env.vars.min_instances
    max = env.vars.max_instances
    desired = env.vars.min_instances

    subnets = network.subnets.private.*.id
    target_groups = (app)
    launch_template = app

    health_check (
      type = elb
      grace_period = 300
    )

    scale_on cpu > 70%
  )

  # outputs
  output url = load_balancer.app.dns_name | x => "http://$x"
)
```

### Example 3: Configuration Management Focus

```
stack config (
  env = prod

  # reference existing instances
  read instances app (
    tags (name = app-server-*)
  )

  # configure all app instances
  for instance in instances.app (
    configure instance (
      # package management
      install (
        nginx@1.20
        nodejs@18.x
        pm2@latest
      )

      # user management
      user app (
        shell = /bin/bash
        home = /opt/app
        system = true
      )

      # git repo
      git clone https://github.com/org/app to /opt/app (
        version = env.vars.app_version
        owner = app
      )

      # files
      file /opt/app/.env (
        content = """
          NODE_ENV=production
          PORT=3000
          VERSION=$env.vars.app_version
        """
        mode = 0600
        owner = app
      )

      template /etc/nginx/sites-available/app from nginx/app.conf (
        vars (
          port = 3000
          server_name = app.example.com
        )
      )

      symlink /etc/nginx/sites-enabled/app to /etc/nginx/sites-available/app

      # commands
      run "npm install" at /opt/app (
        user = app
        creates = /opt/app/node_modules
      )

      run "npm run build" at /opt/app (
        user = app
        creates = /opt/app/dist
      )

      # systemd service
      systemd_unit app (
        type = simple
        user = app
        working_directory = /opt/app
        exec_start = "/usr/bin/pm2 start app.js"
        restart = on-failure
      )

      # service management
      service nginx restart if changed
      service app (
        state = running
        enabled = true
      )
    )
  )
)
```

### Example 4: Concise Syntax Showcase

```
# super concise
env prod region=us-west-2

stack web env=prod (
  read vpc main tags=(name=main-vpc)

  sg web vpc=vpc.main.id allow=(80,443)

  instance web ami=ami-12345 type=t3.medium count=3 sg=web (
    install nginx nodejs
    file /etc/nginx/nginx.conf from=template/nginx.conf mode=0644
    service nginx start enabled
  )

  output ips = instance.web.*.public_ip
)
```

## Syntax Rules

### Identifiers
```
# No quotes needed for simple identifiers
name = web-server
region = us-west-2
app_version = 1.2.3

# Quotes only when necessary (spaces, special chars)
name = "web server with spaces"
path = "/path/with/slashes"
```

### Lists and Tuples
```
# Parentheses for lists
regions = (us-west-2, us-east-1, eu-west-1)
ports = (80, 443, 8080)

# No commas needed for simple lists
install nginx nodejs git

# Mixed
allow (80, 443) from 0.0.0.0/0
```

### Maps/Objects
```
# Nested with parentheses
config (
  database (
    host = localhost
    port = 5432
    ssl = true
  )

  cache (
    host = localhost
    port = 6379
  )
)

# Inline assignment
tags (name=web env=prod team=platform)
```

### String Interpolation
```
# Use $ for simple interpolation
name = web-$env
url = "http://$host:$port"

# Use ${} for expressions
url = "http://${instance.web.ip}:${config.port}"
message = "Total: ${count * price}"
```

### Comments
```
# Single line comment

## Section header comment

# Multi-line comments
# span multiple lines
# like this
```

### Operators
```
# Assignment
name = value

# Comparison
x == y
x != y
x > y
x < y

# Logical
x and y
x or y
not x

# Arithmetic
x + y
x - y
x * y
x / y

# Pipeline
data | filter | map | sort

# Lambda
x => x * 2
(x, y) => x + y
```

## Type System

```
# Type annotations (optional)
var name: string = web
var count: int = 3
var enabled: bool = true
var ports: list<int> = (80, 443)
var config: map<string, any> = (host=localhost, port=5432)

# Type inference (preferred)
name = web        # inferred as string
count = 3         # inferred as int
enabled = true    # inferred as bool
ports = (80, 443) # inferred as list<int>
```

## Safety Features

```
# Read-only resources
read vpc main (id = vpc-12345)

# Validation
var instance_type: string = t3.medium (
  validate = instance_type in (t3.small, t3.medium, t3.large)
  error = "must be a valid t3 instance type"
)

# Lifecycle rules
resource instance web (
  # ... config ...

  lifecycle (
    prevent_destroy = true
    ignore_changes = (tags)
    create_before_destroy = true
  )
)

# Dependencies
stack app (
  depends_on = (network, database)
  # ...
)
```

## Philosophy

This syntax prioritizes:

1. **Readability**: Looks like natural configuration, not code
2. **Conciseness**: Minimal ceremony, maximum clarity
3. **Consistency**: Same patterns throughout
4. **Safety**: Type-safe, with clear readonly vs managed
5. **Flexibility**: Can be verbose or concise as needed

The goal is configuration that reads like documentation.
