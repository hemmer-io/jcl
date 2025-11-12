# Getting Started with JCL

## What is JCL?

JCL (Jack Configuration Language) is a human-readable configuration language for infrastructure as code and configuration management. It combines the ease of use of Ruby with the power of Go templates, all while prioritizing safety and flexibility.

## Key Features

- **Human-readable syntax**: Minimal punctuation, natural language flow
- **Unified IaC + Config Management**: Provision infrastructure and configure systems in one language
- **Type-safe**: Strong type system with inference
- **Read-only references**: Reference existing infrastructure without managing it
- **Rust-powered**: Fast, safe, and reliable

## Installation

```bash
# Build from source
git clone https://github.com/turner-hemmer/jcl
cd jcl
cargo build --release

# Install
cargo install --path .
```

## Your First JCL Configuration

Create a file `hello.jcl`:

```
# Simple example
environments = (dev)

env.dev = (
  region = us-west-2

  vars (
    greeting = "Hello, JCL!"
  )
)

out.message = env.dev.vars.greeting
```

Validate it:

```bash
jcl validate hello.jcl
```

## Basic Concepts

### Environments

Environments represent deployment contexts (dev, staging, prod):

```
environments = (prod, dev)

env.prod = (
  region = us-west-2

  vars (
    instance_type = t3.large
  )

  tags (
    team = platform
    env = production
  )
)
```

### Stacks

Stacks are logical groupings of resources:

```
stacks = (networking, application)

stack.networking = (
  env = prod
  resources = (vpc, subnet)
)

stack.application = (
  env = prod
  depends_on = (networking)
  resources = (instance, lb)
)
```

### Resources

Resources are infrastructure components you want to manage:

```
resource.web_instance = (
  ami = ami-12345
  type = t3.medium

  tags (
    name = web-server
  )
)
```

### Data Sources (Read-Only References)

Reference existing infrastructure without managing it:

```
read.vpc = (
  id = vpc-12345
)

resource.subnet = (
  vpc = read.vpc.id
  cidr = 10.0.1.0/24
)
```

### Configuration Management

Configure instances after creation:

```
resource.web_instance = (
  ami = ami-12345
  type = t3.medium

  configure (
    install nginx nodejs

    file /etc/nginx/nginx.conf (
      from = template/nginx.conf
      mode = 0644
    )

    service nginx start enabled
  )
)
```

## Syntax Basics

### No Quotes Needed

```
# These work without quotes
name = web-server
region = us-west-2
version = 1.2.3

# Quotes only for spaces or special cases
name = "web server with spaces"
path = "/path/with/slashes"
```

### Lists with Parentheses

```
regions = (us-west-2, us-east-1, eu-west-1)
ports = (80, 443, 8080)

# Or without commas for simple lists
install nginx nodejs git
```

### Maps with Parentheses

```
tags (
  name = web-server
  env = production
  team = platform
)

# Inline style
tags (name=web env=prod team=platform)
```

### References with Dots

```
env.prod.vars.instance_type
resource.vpc.id
read.subnet.*.id
```

### String Interpolation

```
# Simple
name = web-$env

# Expression
url = "http://${resource.lb.dns_name}:${port}"
```

## Common Patterns

### Multiple Instances

```
resource.worker = (
  count = 3
  ami = ami-12345
  type = t3.small

  tags (
    name = worker-${count.index}
  )
)
```

### Conditional Values

```
instance_type = env == prod ? t3.large : t3.small
```

### Referencing Other Resources

```
resource.subnet = (
  vpc = resource.vpc.id
  cidr = 10.0.1.0/24
)

resource.instance = (
  subnet = resource.subnet.id
  security_groups = (resource.sg.id)
)
```

## CLI Commands

```bash
# Initialize a new project
jcl init myproject

# Validate configuration
jcl validate .

# Plan changes
jcl plan application

# Apply changes
jcl apply application

# Destroy resources
jcl destroy application

# Show state
jcl show application

# List stacks
jcl list

# Format files
jcl fmt .
```

## Example: Complete Web Application

```
# Structure
environments = (prod)
stacks = (network, app)

# Environment
env.prod = (
  region = us-west-2
  vars (app_name=myapp version=1.0.0 type=t3.medium count=2)
  tags (team=platform env=prod)
)

# Network (existing)
stack.network = (env=prod resources=(vpc,subnet,sg))

read.vpc = (id=vpc-12345)
read.subnet = (vpc=read.vpc.id tags=(tier=public))

resource.sg = (
  vpc = read.vpc.id
  name = ${env.prod.vars.app_name}-sg
  allow (80, 443) from 0.0.0.0/0
)

# Application
stack.app = (env=prod depends_on=(network) resources=(instance,lb))

resource.instance = (
  ami = ami-12345
  type = env.prod.vars.type
  count = env.prod.vars.count
  subnet = read.subnet.id
  security_groups = (resource.sg.id)

  configure (
    install nginx
    file /var/www/html/index.html (
      content = "<h1>${env.prod.vars.app_name} v${env.prod.vars.version}</h1>"
    )
    service nginx start enabled
  )
)

resource.lb = (
  type = application
  subnets = (read.subnet.id)
  security_groups = (resource.sg.id)

  listener (port=80 forward_to=resource.instance.*.id)
  health_check (path=/health interval=30)
)

out.url = "http://${resource.lb.dns_name}"
```

## Next Steps

- Check out [SYNTAX_V3.md](../SYNTAX_V3.md) for complete syntax reference
- See [examples/](../examples/) for more examples
- Read [DESIGN.md](../DESIGN.md) for architecture details

## Integration with Hemmer

JCL is designed to work with Hemmer for module and registry management. Hemmer handles:
- Module packaging and distribution
- Registry operations
- Dependency resolution

JCL focuses on:
- Configuration language and syntax
- Resource provisioning
- Configuration management
- State management

Together, they provide a complete infrastructure management solution.
