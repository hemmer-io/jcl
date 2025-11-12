# JCL - Concise Configuration Syntax

## Core Philosophy
- Declare structure first, define details later
- Dot notation for namespacing
- Parentheses for grouping
- No quotes unless necessary
- Assignment-based, not declaration-based

## Basic Structure

```
# Define structure
environments = (prod, dev, staging)
stacks = (networking, database, application)

# Configure environments
environment.prod = (
  region = us-west-2
  account = 123456789

  vars.instance_type = t3.large
  vars.min_count = 3
  vars.app_version = 1.2.3

  tags.team = platform
  tags.cost_center = engineering
)

# Configure stacks
stack.networking = (
  env = prod

  # Resources in this stack
  resources = (vpc, subnet, security_group)
)

# Define resources
resource.vpc = (
  cidr = 10.0.0.0/16
  enable_dns = true

  tags (
    name = main-vpc
    env = prod
  )
)

resource.subnet = (
  vpc = resource.vpc.id
  cidr = 10.0.1.0/24
  availability_zone = us-west-2a
)
```

## Full Example

```
# Structure declaration
environments = (prod, dev)
stacks = (networking, application)

# Environment configuration
environment.prod = (
  region = us-west-2
  account = 123456789

  vars (
    instance_type = t3.large
    app_version = 1.2.3
    min_instances = 3
  )

  tags (
    team = platform
    managed_by = jcl
  )
)

environment.dev = (
  region = us-west-2

  vars (
    instance_type = t3.small
    app_version = dev
    min_instances = 1
  )

  tags (
    team = platform
    managed_by = jcl
  )
)

# Stack: Networking
stack.networking = (
  env = prod
  resources = (vpc, subnets, security_groups)
)

# Read existing VPC (not managed)
read.vpc = (
  id = vpc-12345
  readonly = true
)

# Stack: Application
stack.application = (
  env = prod
  depends_on = (networking)
  resources = (web_instance, load_balancer, target_group)
)

# Resource definitions
resource.security_group = (
  vpc = read.vpc.id
  name = web-sg

  ingress (
    from_port = 80
    to_port = 80
    protocol = tcp
    cidr = 0.0.0.0/0
  )

  ingress (
    from_port = 443
    to_port = 443
    protocol = tcp
    cidr = 0.0.0.0/0
  )

  tags = environment.prod.tags
)

resource.web_instance = (
  type = aws_instance
  ami = ami-12345
  instance_type = environment.prod.vars.instance_type
  count = environment.prod.vars.min_instances
  security_groups = (resource.security_group.id)

  tags (
    name = web-server
    version = environment.prod.vars.app_version
  )

  # Configuration
  configure (
    # Install packages
    install (nginx, nodejs, git)

    # Configure nginx
    file /etc/nginx/nginx.conf (
      from = template/nginx.conf
      mode = 0644
      owner = root
    )

    # Start service
    service nginx (
      state = running
      enabled = true
    )
  )
)

resource.load_balancer = (
  type = aws_lb
  load_balancer_type = application
  subnets = read.vpc.subnets.*.id
  security_groups = (resource.security_group.id)

  tags = environment.prod.tags
)

resource.target_group = (
  type = aws_lb_target_group
  port = 80
  protocol = http
  vpc = read.vpc.id

  health_check (
    path = /health
    interval = 30
  )
)

# Outputs
output.web_ips = resource.web_instance.*.public_ip
output.lb_dns = resource.load_balancer.dns_name
output.app_url = "http://${resource.load_balancer.dns_name}"
```

## Alternative: Even More Concise

```
# Ultra-compact style
envs = (prod, dev)
stacks = (net, app)

env.prod = (
  region = us-west-2
  vars (type=t3.large count=3 version=1.2.3)
  tags (team=platform env=prod)
)

stack.net = (env=prod resources=(vpc,subnet,sg))
stack.app = (env=prod depends=(net) resources=(instance,lb))

# Existing resources (read-only)
read.vpc = (id=vpc-12345)
read.subnet = (vpc=read.vpc.id tags=(tier=public))

# New resources (managed)
resource.sg = (
  vpc = read.vpc.id
  name = web-sg
  allow (80, 443) from 0.0.0.0/0
)

resource.instance = (
  ami = ami-12345
  type = env.prod.vars.type
  count = env.prod.vars.count
  sg = resource.sg.id

  configure (
    install nginx nodejs
    file /etc/nginx/nginx.conf from=template/nginx.conf mode=0644
    service nginx start enabled
  )
)

resource.lb = (
  type = application
  subnets = read.subnet.*.id
  sg = resource.sg.id

  listen 80 forward=resource.target_group.arn
)

resource.target_group = (
  port = 80
  vpc = read.vpc.id
  health /health interval=30
)

# Outputs
out.ips = resource.instance.*.public_ip
out.url = "http://${resource.lb.dns}"
```

## Syntax Patterns

### Lists
```
# Simple list
regions = (us-west-2, us-east-1, eu-west-1)

# List of resources
resources = (vpc, subnet, instance, lb)

# No commas for short lists
install nginx nodejs git
```

### Maps/Objects
```
# Nested structure
vars (
  instance_type = t3.large
  count = 3
  enabled = true
)

# Inline
tags (name=web env=prod team=platform)

# Mixed
config (
  database (
    host = localhost
    port = 5432
  )
  cache.host = localhost
  cache.port = 6379
)
```

### References
```
# Dot notation
environment.prod.vars.instance_type
resource.vpc.id
read.subnet.*.id

# Array access
resource.instances[0].ip
resource.instances.*.public_ip
```

### Short Forms
```
# Full form
environment.prod = (...)

# Short form
env.prod = (...)

# Ultra short
e.prod = (...)

# Similarly
stack -> stk -> s
resource -> res -> r
output -> out -> o
```

### Conditionals
```
# Ternary
instance_type = env == prod ? t3.large : t3.small

# When
state = when env (
  prod => running
  dev => stopped
  * => stopped
)
```

### Iteration
```
# For loop
for i in 0..3 (
  resource.worker-$i = (
    type = t3.small
    ami = ami-12345
  )
)

# For each resource
for region in (us-west-2, us-east-1) (
  resource.instance-$region = (
    region = $region
    type = t3.medium
  )
)

# List comprehension
ips = [r.ip for r in resource.instances]
active = [r for r in resources if r.state == running]
```

### Configuration Management
```
# Concise package install
install nginx nodejs pm2@latest

# File management
file /etc/nginx/nginx.conf from=template/nginx.conf mode=0644
file /opt/app/.env content="PORT=3000\nENV=prod" mode=0600

# Service management
service nginx start enabled
service app restart

# Git
git clone https://github.com/org/app to=/opt/app version=1.2.3

# Commands
run "npm install" at=/opt/app creates=node_modules
```

## Type Inference

```
# Everything is type-inferred by default
name = myapp                    # string
count = 3                       # int
enabled = true                  # bool
price = 9.99                    # float
regions = (us-west-2, us-east)  # list<string>
ports = (80, 443)               # list<int>

# Explicit types only when needed
var count: int = 3
var config: map = (host=localhost port=5432)
```

## Comments
```
# Single line

## Section header

# Multi-line comment
# continues here
# and here
```

## Complete Real-World Example

```
# Infrastructure for web application

# Define structure
environments = (prod, staging, dev)
stacks = (network, database, application, monitoring)

# Production environment
env.prod = (
  region = us-west-2
  account = 123456789

  vars (
    app_name = myapp
    app_version = 2.1.0
    instance_type = t3.large
    db_instance = db.t3.large
    min_instances = 3
    max_instances = 10
  )

  tags (
    team = platform
    cost_center = engineering
    env = production
  )
)

# Network stack (references existing infrastructure)
stack.network = (
  env = prod
  resources = (vpc, subnets, security_groups)
)

read.vpc = (id = vpc-12345)
read.subnet_public = (vpc=read.vpc.id tags=(tier=public))
read.subnet_private = (vpc=read.vpc.id tags=(tier=private))

resource.sg_alb = (
  vpc = read.vpc.id
  name = ${env.prod.vars.app_name}-alb-sg
  allow (80, 443) from 0.0.0.0/0
)

resource.sg_app = (
  vpc = read.vpc.id
  name = ${env.prod.vars.app_name}-app-sg
  allow 3000 from resource.sg_alb
  allow 22 from 10.0.0.0/8
)

# Database stack
stack.database = (
  env = prod
  depends_on = (network)
  resources = (db_instance, db_subnet_group)
)

resource.db_subnet_group = (
  name = ${env.prod.vars.app_name}-db-subnet
  subnets = read.subnet_private.*.id
)

resource.db_instance = (
  identifier = ${env.prod.vars.app_name}-db
  engine = postgres
  engine_version = 14.5
  instance_class = env.prod.vars.db_instance
  allocated_storage = 100

  db_subnet_group = resource.db_subnet_group.name
  vpc_security_groups = (resource.sg_app.id)

  backup_retention_period = 7
  multi_az = true

  tags = env.prod.tags
)

# Application stack
stack.application = (
  env = prod
  depends_on = (network, database)
  resources = (lb, target_group, launch_template, asg)
)

resource.lb = (
  name = ${env.prod.vars.app_name}-alb
  type = application
  subnets = read.subnet_public.*.id
  security_groups = (resource.sg_alb.id)

  listener (
    port = 80
    protocol = http
    default_action = forward
    target_group = resource.target_group.arn
  )

  listener (
    port = 443
    protocol = https
    certificate = read.certificate.arn
    default_action = forward
    target_group = resource.target_group.arn
  )

  tags = env.prod.tags
)

resource.target_group = (
  name = ${env.prod.vars.app_name}-tg
  port = 3000
  protocol = http
  vpc = read.vpc.id

  health_check (
    path = /health
    interval = 30
    timeout = 5
    healthy_threshold = 2
    unhealthy_threshold = 3
  )

  tags = env.prod.tags
)

resource.launch_template = (
  name = ${env.prod.vars.app_name}-lt
  image_id = ami-12345
  instance_type = env.prod.vars.instance_type
  vpc_security_group_ids = (resource.sg_app.id)

  user_data = base64encode(file://userdata.sh)

  tags = env.prod.tags
)

resource.asg = (
  name = ${env.prod.vars.app_name}-asg
  min_size = env.prod.vars.min_instances
  max_size = env.prod.vars.max_instances
  desired_capacity = env.prod.vars.min_instances

  vpc_zone_identifier = read.subnet_private.*.id
  target_group_arns = (resource.target_group.arn)

  launch_template (
    id = resource.launch_template.id
    version = $Latest
  )

  health_check_type = elb
  health_check_grace_period = 300

  # Auto-scaling policy
  scale_on cpu > 70%
  scale_on requests > 1000

  tags = env.prod.tags
)

# Configuration for instances
configure.app_instances = (
  targets = resource.asg.instances

  install (
    nginx@1.20
    nodejs@18.x
    pm2@latest
  )

  user app (
    shell = /bin/bash
    home = /opt/app
    system = true
  )

  git clone https://github.com/org/app to /opt/app (
    version = env.prod.vars.app_version
    owner = app
  )

  file /opt/app/.env (
    content = """
      NODE_ENV=production
      PORT=3000
      VERSION=${env.prod.vars.app_version}
      DB_HOST=${resource.db_instance.endpoint}
      DB_NAME=appdb
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

  run "npm install --production" at /opt/app (
    user = app
    creates = /opt/app/node_modules
  )

  systemd_unit app (
    type = simple
    user = app
    working_directory = /opt/app
    exec_start = "/usr/bin/pm2 start app.js"
    restart = on-failure
  )

  service nginx restart if changed
  service app start enabled
)

# Outputs
out.lb_dns = resource.lb.dns_name
out.app_url = "https://${resource.lb.dns_name}"
out.db_endpoint = resource.db_instance.endpoint
out.asg_name = resource.asg.name
```

## Key Principles

1. **Declare First, Define Later**: List your stacks, then define them
2. **Dot Notation**: Everything uses consistent dot notation
3. **Assignment Style**: `thing.name = (...)` not `thing name { }`
4. **Minimal Syntax**: No quotes, minimal punctuation
5. **Progressive Disclosure**: Can be concise or verbose as needed
6. **Type Inference**: Types inferred unless explicitly needed
7. **Natural Reading**: Reads like configuration, not code

This makes JCL feel more like a configuration file than a programming language, while maintaining all the power you need.
