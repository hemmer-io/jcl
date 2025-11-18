# JCL Syntax Guide

## Overview

JCL uses a Ruby-inspired syntax that's clean, readable, and expressive. This guide covers the language syntax in detail.

## Comments

```ruby
# Single-line comment

# Multi-line comments
# are written like this
# across multiple lines

/* Block comments
   are also supported */
```

## Variables

### Declaration

```ruby
# Simple variable (immutable by default)
variable "app_name" = "myapp"

# With type annotation
variable "count" : int = 3
variable "enabled" : bool = true
variable "tags" : map<string, string> = {
  env = "prod"
}

# With default and description
variable "instance_type" : string = "t3.medium" {
  description = "EC2 instance type"
  validation {
    condition = contains(["t3.small", "t3.medium", "t3.large"], var.instance_type)
    error_message = "Must be a t3 instance type"
  }
}

# Mutable variable (for computed values)
mutable computed "dynamic_value" {
  value = some_function()
}
```

### Types

```ruby
# Primitive types
: string
: int
: float
: bool

# Collection types
: list<string>
: map<string, int>
: set<string>

# Structured types
: object {
    name: string
    age: int
    active: bool
  }

# Any type (no type checking)
: any
```

## Literals

```ruby
# Strings
"hello"
'single quotes also work'
"""
Multi-line string
can span multiple lines
"""

# String interpolation
"Hello, ${var.name}!"
"Count: ${var.count + 1}"

# Numbers
42          # int
3.14        # float
1_000_000   # underscores for readability

# Booleans
true
false

# Null
null

# Lists
[1, 2, 3, 4, 5]
["apple", "banana", "cherry"]

# Maps
{
  name = "Alice"
  age = 30
  active = true
}

# Nested structures
{
  server = {
    host = "localhost"
    port = 8080
    ssl = false
  }
  database = {
    host = "db.example.com"
    port = 5432
  }
}
```

## Expressions

### References

```ruby
# Variable reference
var.instance_type

# Environment reference
env.name
env.vars.app_version
env.tags

# Resource reference
resource.aws_instance.web.id
resource.aws_instance.web.public_ip

# Data source reference
data.aws_vpc.main.id

# Stack reference
stack.networking.output.vpc_id

# Module reference
module.vpc.output.id
```

### Operators

```ruby
# Arithmetic
1 + 2
10 - 5
3 * 4
20 / 4
10 % 3

# Comparison
a == b
a != b
a < b
a <= b
a > b
a >= b

# Logical
a && b
a || b
!a

# String concatenation
"Hello, " + "world"
```

### Conditionals

```ruby
# Ternary
value = condition ? "yes" : "no"

# If expression
instance_type = if(env.name == "prod", "t3.large", "t3.small")

# When expression (pattern matching)
state = when {
  env.name == "prod" -> "running"
  env.name == "staging" -> "running"
  env.name == "dev" -> "stopped"
  default -> "stopped"
}
```

### Functions

```ruby
# Built-in functions
upper("hello")                    # "HELLO"
lower("WORLD")                    # "world"
trim("  spaces  ")                # "spaces"
replace("hello", "l", "L")        # "heLLo"
split("a,b,c", ",")               # ["a", "b", "c"]
join(["a", "b", "c"], "-")        # "a-b-c"

# Collection functions
length([1, 2, 3])                 # 3
contains([1, 2, 3], 2)            # true
range(1, 5)                       # [1, 2, 3, 4, 5]
range(10)                         # [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]

# Map functions
merge(map1, map2)
lookup(map, "key", "default")
keys(map)
values(map)

# Encoding functions
base64encode("hello")
base64decode("aGVsbG8=")
jsonencode({name = "test"})
jsondecode('{"name":"test"}')

# Template functions
template("./file.tpl")
template("./file.tpl", {var = "value"})
templatefile("./file.tpl", {var = "value"})

# Validation functions
is_valid_email("test@example.com")
is_valid_url("https://example.com")
```

### Iteration

```ruby
# For each (returns list)
[for x in [1, 2, 3]: x * 2]           # [2, 4, 6]
[for s in ["a", "b"]: upper(s)]       # ["A", "B"]

# For each with filter
[for x in range(10): x if x % 2 == 0] # [0, 2, 4, 6, 8]

# For each (returns map)
{for k, v in map: k => upper(v)}

# Splat expressions
resource.aws_instance.servers[*].id
resource.aws_instance.servers[*].private_ip
```

### Pipelines

```ruby
# Pipeline operator (like Unix pipes)
value = data
  | map(x -> x * 2)
  | filter(x -> x > 10)
  | sort()
  | join(", ")

# Example
endpoints = resource.aws_instance.servers[*].private_ip
  | map(ip -> "http://${ip}:8080")
  | filter(url -> is_valid_url(url))
  | sort()
```

## Blocks

### Environment

```ruby
environment "production" {
  region = "us-west-2"
  account_id = "123456789"

  variables {
    app_version = "1.2.3"
    instance_type = "t3.large"
  }

  tags {
    managed_by = "jcl"
    environment = "production"
  }

  provider "aws" {
    region = "us-west-2"
    profile = "production"
  }
}
```

### Stack

```ruby
stack "web_application" {
  environment = env.production

  depends_on = [
    stack.networking,
    stack.database
  ]

  variables {
    min_instances = 3
  }

  # Resources go here
}
```

### Resource

```ruby
# Basic resource
resource "aws_instance" "web" {
  ami = "ami-12345"
  instance_type = var.instance_type

  tags = {
    name = "web-server"
  }
}

# Resource with lifecycle
resource "aws_s3_bucket" "data" {
  bucket = "my-data-bucket"

  lifecycle {
    prevent_destroy = true
    ignore_changes = ["tags"]
  }
}

# Resource with dependencies
resource "aws_instance" "app" {
  # ...

  depends_on = [
    resource.aws_security_group.app,
    resource.aws_iam_role.app
  ]
}

# Resource with count
resource "aws_instance" "workers" {
  count = 3

  ami = "ami-12345"
  instance_type = "t3.small"

  tags = {
    name = "worker-${count.index}"
  }
}

# Resource with for_each
resource "aws_instance" "servers" {
  for_each = {
    web = "t3.medium"
    api = "t3.large"
    admin = "t3.small"
  }

  ami = "ami-12345"
  instance_type = each.value

  tags = {
    name = each.key
  }
}
```

### Data Source

```ruby
# Read-only reference to existing resource
data "aws_vpc" "main" {
  id = "vpc-12345"

  lifecycle {
    managed = false  # Explicitly read-only
  }
}

# Data source with filter
data "aws_ami" "ubuntu" {
  most_recent = true

  filter {
    name = "name"
    values = ["ubuntu/images/hvm-ssd/ubuntu-focal-20.04-amd64-server-*"]
  }

  filter {
    name = "virtualization-type"
    values = ["hvm"]
  }

  owners = ["099720109477"]  # Canonical

  lifecycle {
    managed = false
  }
}
```

### Configuration

```ruby
# Configuration management block
resource "aws_instance" "web" {
  ami = "ami-12345"
  instance_type = "t3.medium"

  # Configure after creation
  configure {
    # Install packages
    package "nginx" {
      state = "present"
      version = "1.20"
    }

    package "nodejs" {
      state = "present"
      version = "18.x"
    }

    # Manage files
    file "/etc/nginx/nginx.conf" {
      content = template("./nginx.conf.tpl")
      mode = "0644"
      owner = "root"
      group = "root"
    }

    file "/opt/app/.env" {
      content = """
        APP_VERSION=${env.vars.app_version}
        NODE_ENV=production
      """
      mode = "0600"
    }

    # Git clone
    git_clone {
      repo = "https://github.com/org/app"
      dest = "/opt/app"
      version = env.vars.app_version
    }

    # Run command
    command "npm install" {
      cwd = "/opt/app"
      creates = "/opt/app/node_modules"
    }

    # Manage services
    service "nginx" {
      state = "running"
      enabled = true
    }

    service "app" {
      state = "running"
      enabled = true
    }
  }
}
```

### Module

```ruby
# Define a reusable module
module "vpc" {
  source = "./modules/vpc"

  cidr_block = "10.0.0.0/16"
  availability_zones = ["us-west-2a", "us-west-2b"]

  tags = env.tags
}

# Use module outputs
resource "aws_instance" "web" {
  subnet_id = module.vpc.public_subnet_ids[0]
  vpc_security_group_ids = [module.vpc.default_security_group_id]
}
```

### Output

```ruby
# Simple output
output "instance_id" {
  value = resource.aws_instance.web.id
}

# Output with description
output "web_url" {
  value = "http://${resource.aws_instance.web.public_ip}"
  description = "Public URL for the web server"
}

# Computed output
output "all_ips" {
  value = resource.aws_instance.servers[*].private_ip
    | sort()
    | join(", ")
  description = "All server IP addresses"
}
```

### Validation

```ruby
variable "instance_type" {
  validation {
    condition = contains(["t3.small", "t3.medium", "t3.large"], var.instance_type)
    error_message = "Instance type must be t3.small, t3.medium, or t3.large"
  }
}

resource "aws_s3_bucket" "data" {
  bucket = var.bucket_name

  validation {
    condition = length(var.bucket_name) >= 3 && length(var.bucket_name) <= 63
    error_message = "Bucket name must be between 3 and 63 characters"
  }
}
```

### Testing

```ruby
test "web_server_configuration" {
  stack = stack.web_application

  # Assertions
  assert {
    resource.aws_instance.web.instance_type == "t3.large"
    length(resource.aws_instance.web.tags) > 0
    resource.aws_instance.web.tags.environment == "production"
  }

  # Integration tests (run after apply)
  after_apply {
    http_check "https://${resource.aws_instance.web.public_ip}" {
      status = 200
      timeout = "30s"
      contains = "Welcome"
    }

    command "curl -f https://${resource.aws_instance.web.public_ip}/health" {
      exit_code = 0
    }
  }
}
```

## Complete Example

```ruby
# environments/production.jcl

environment "production" {
  provider "aws" {
    region = "us-west-2"
  }

  variables {
    app_name = "myapp"
    app_version = "1.2.3"
    instance_type = "t3.large"
  }

  tags {
    environment = "production"
    managed_by = "jcl"
  }
}

stack "application" {
  environment = env.production

  data "aws_vpc" "main" {
    tags = { name = "main-vpc" }
    lifecycle { managed = false }
  }

  resource "aws_security_group" "app" {
    vpc_id = data.aws_vpc.main.id
    name = "${env.vars.app_name}-sg"

    ingress {
      from_port = 443
      to_port = 443
      protocol = "tcp"
      cidr_blocks = ["0.0.0.0/0"]
    }
  }

  resource "aws_instance" "app" {
    count = 3

    ami = "ami-12345"
    instance_type = env.vars.instance_type
    vpc_security_group_ids = [resource.aws_security_group.app.id]

    tags = merge(env.tags, {
      name = "${env.vars.app_name}-${count.index}"
      version = env.vars.app_version
    })

    configure {
      package "nginx" { state = "present" }

      file "/var/www/html/index.html" {
        content = "<h1>App v${env.vars.app_version}</h1>"
      }

      service "nginx" {
        state = "running"
        enabled = true
      }
    }
  }

  output "instance_ips" {
    value = resource.aws_instance.app[*].public_ip | join(", ")
  }
}
```
