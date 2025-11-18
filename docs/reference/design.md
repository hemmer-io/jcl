# JCL - Jack Configuration Language
## Design Document

## Overview
A Rust-based configuration language that bridges infrastructure as code and configuration management, combining Ruby's ease of use with Go template's power.

## Core Philosophy

### Ease of Use (Ruby-inspired)
- Clean, readable syntax
- Intuitive block structures
- Minimal ceremony
- Natural language constructs

### Power (Go templates + more)
- Built-in functions for manipulation
- Conditionals and loops
- String interpolation
- Data transformation pipelines

### Safety First
- Strong type system
- Immutability by default
- Validation before execution
- Dry-run capabilities
- State locking
- Explicit mutation markers

### Flexibility
- Extensible through plugins
- Multiple provider support
- Custom resource types
- Reusable modules/components

## Core Abstractions

### 1. Environment
Represents a deployment context (dev, staging, prod, etc.)

```ruby
environment "production" {
  region = "us-west-2"
  account_id = "123456789"

  variables {
    db_size = "large"
    replica_count = 3
  }

  tags {
    owner = "platform-team"
    cost_center = "engineering"
  }
}
```

### 2. Stack
A logical grouping of resources that are managed together

```ruby
stack "web-application" {
  environment = env.production

  depends_on = [
    stack.networking,
    stack.database
  ]

  variables {
    instance_type = "t3.large"
  }
}
```

### 3. Resources
Actual infrastructure or configuration items to be managed

```ruby
resource "aws_instance" "web_server" {
  ami = "ami-12345678"
  instance_type = var.instance_type

  tags = merge(env.tags, {
    name = "web-server-${env.name}"
  })
}
```

### 4. Read-Only References
Reference existing resources without managing them

```ruby
data "aws_vpc" "existing" {
  id = "vpc-12345"

  lifecycle {
    managed = false  # Explicitly read-only
  }
}

resource "aws_subnet" "new" {
  vpc_id = data.aws_vpc.existing.id  # Reference but don't manage
  cidr_block = "10.0.1.0/24"
}
```

## Key Features

### 1. Type Safety
```ruby
# Strong typing with inference
variable "count" : int = 3
variable "name" : string = "app"
variable "settings" : map<string, any> = {
  enabled = true
}

# Type validation at parse time
resource "example" "bad" {
  count = "three"  # ERROR: expected int, got string
}
```

### 2. Immutability & Safety
```ruby
# Variables are immutable by default
variable "region" = "us-west-2"
region = "us-east-1"  # ERROR: cannot reassign immutable variable

# Explicit mutation for computed values
mutable computed "dynamic_tags" {
  value = merge(base_tags, runtime_tags)
}
```

### 3. Built-in Functions (Go template style)
```ruby
# String manipulation
resource "aws_s3_bucket" "data" {
  bucket = lower(replace(var.app_name, "_", "-"))
}

# Collections
resource "aws_security_group_rule" "allow_ports" {
  for_each = range(8000, 8010)

  port = each.value
  protocol = "tcp"
}

# Conditionals
resource "aws_instance" "web" {
  instance_type = if(env.name == "prod", "t3.large", "t3.small")

  monitoring_enabled = when {
    env.name == "prod" -> true
    env.name == "staging" -> true
    default -> false
  }
}

# Data transformation pipelines
output "formatted_endpoints" {
  value = resource.aws_instance.servers[*].private_ip
    | map(ip -> "http://${ip}:8080")
    | filter(endpoint -> is_valid_url(endpoint))
    | sort()
}
```

### 4. Modules & Reusability
```ruby
module "vpc" {
  source = "./modules/vpc"

  cidr_block = "10.0.0.0/16"
  availability_zones = ["us-west-2a", "us-west-2b"]
}

# Use module outputs
resource "aws_subnet" "private" {
  vpc_id = module.vpc.id
}
```

### 5. Configuration Management Integration
```ruby
# Infrastructure + Configuration in one
resource "aws_instance" "app_server" {
  ami = "ami-12345"
  instance_type = "t3.medium"

  # Configure the instance after creation
  configure {
    # Ansible-style tasks
    package "nginx" {
      state = "present"
      version = "1.20"
    }

    file "/etc/nginx/nginx.conf" {
      content = template("./nginx.conf.tpl")
      mode = "0644"
      owner = "root"
    }

    service "nginx" {
      state = "running"
      enabled = true
    }
  }
}

# Or use separate configuration resources
config "app_deployment" {
  target = resource.aws_instance.app_server

  tasks {
    git_clone {
      repo = "https://github.com/org/app"
      dest = "/opt/app"
      version = var.app_version
    }

    systemd_unit "myapp" {
      content = template("./myapp.service.tpl")
      state = "running"
    }
  }
}
```

### 6. Validation & Testing
```ruby
# Built-in validation
resource "aws_instance" "web" {
  instance_type = var.instance_type

  validation {
    condition = contains(["t3.small", "t3.medium", "t3.large"], var.instance_type)
    error_message = "Instance type must be t3.small, medium, or large"
  }
}

# Testing framework
test "web_server_configuration" {
  stack = stack.web_application

  assert {
    resource.aws_instance.web_server.instance_type == "t3.large"
    length(resource.aws_instance.web_server.tags) > 0
  }

  # Integration tests
  after_apply {
    http_check "https://${resource.aws_instance.web_server.public_ip}" {
      status = 200
      timeout = "30s"
    }
  }
}
```

## Architecture

### Rust Implementation Benefits
1. **Safety**: Memory safety, thread safety, strong type system
2. **Performance**: Fast parsing and execution
3. **Reliability**: Robust error handling
4. **Concurrency**: Parallel resource operations
5. **Ecosystem**: Excellent libraries (serde, tokio, etc.)

### Core Components

```
┌─────────────────────────────────────────────────────────┐
│                     CLI / REPL                          │
└─────────────────────────────────────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────┐
│                   Parser / Lexer                        │
│              (pest or nom for parsing)                  │
└─────────────────────────────────────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────┐
│                   AST / Type Checker                    │
│            (Validate types, references)                 │
└─────────────────────────────────────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────┐
│                  Evaluation Engine                      │
│     (Resolve variables, functions, dependencies)        │
└─────────────────────────────────────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────┐
│                    Planner / DAG                        │
│        (Build dependency graph, plan changes)           │
└─────────────────────────────────────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────┐
│                  Provider Interface                     │
│         (Abstract interface for all providers)          │
└─────────────────────────────────────────────────────────┘
                            │
         ┌──────────────────┼──────────────────┐
         │                  │                  │
┌────────▼────────┐ ┌──────▼──────┐ ┌────────▼────────┐
│  IaC Providers  │ │   Config    │ │    Custom       │
│  (AWS, GCP,     │ │   Providers │ │    Providers    │
│   Azure, etc)   │ │  (Ansible-  │ │   (Plugins)     │
│                 │ │   style)    │ │                 │
└─────────────────┘ └─────────────┘ └─────────────────┘
                            │
┌─────────────────────────────────────────────────────────┐
│                   State Management                      │
│         (Track managed resources, state lock)           │
└─────────────────────────────────────────────────────────┘
```

## Safety Features

### 1. State Management
- Distributed state locking
- Conflict detection
- Rollback capabilities
- State versioning

### 2. Validation Layers
- **Parse time**: Syntax validation
- **Check time**: Type checking, reference validation
- **Plan time**: Constraint validation, dependency checks
- **Apply time**: Pre-condition checks

### 3. Explicit Mutations
```ruby
# Clear distinction between managed and unmanaged
resource "aws_instance" "managed" {
  # This will be created/updated/deleted by JCL
}

data "aws_instance" "reference_only" {
  # This is only referenced, never modified
  lifecycle {
    managed = false
  }
}
```

### 4. Dry-Run & Plan
```bash
# Always plan before apply
jcl plan stack.web_application
jcl apply stack.web_application --auto-approve=false
```

## Example: Complete Stack

```ruby
# environments/production.jcl
environment "production" {
  provider "aws" {
    region = "us-west-2"
    account_id = "123456789"
  }

  variables {
    app_version = "1.2.3"
    instance_type = "t3.large"
    min_instances = 3
  }
}

# stacks/web-app.jcl
stack "web_application" {
  environment = env.production

  # Reference existing VPC (not managed)
  data "aws_vpc" "main" {
    tags = { name = "main-vpc" }
    lifecycle { managed = false }
  }

  # Managed security group
  resource "aws_security_group" "web" {
    vpc_id = data.aws_vpc.main.id
    name = "web-servers-${env.name}"

    ingress {
      from_port = 80
      to_port = 80
      protocol = "tcp"
      cidr_blocks = ["0.0.0.0/0"]
    }

    ingress {
      from_port = 443
      to_port = 443
      protocol = "tcp"
      cidr_blocks = ["0.0.0.0/0"]
    }
  }

  # Compute instances
  resource "aws_instance" "web_servers" {
    count = env.vars.min_instances

    ami = data.aws_ami.ubuntu.id
    instance_type = env.vars.instance_type
    vpc_security_group_ids = [resource.aws_security_group.web.id]

    tags = merge(env.tags, {
      name = "web-server-${count.index}"
      app_version = env.vars.app_version
    })

    # Configuration management
    configure {
      # Install packages
      package "nginx" { state = "present" }
      package "nodejs" { version = "18.x" }

      # Deploy application
      git_clone {
        repo = "https://github.com/org/app"
        dest = "/opt/app"
        version = env.vars.app_version
      }

      # Configure nginx
      file "/etc/nginx/sites-available/app" {
        content = template("./nginx-app.conf.tpl", {
          port = 3000
          server_name = "app.example.com"
        })
      }

      # Start services
      service "nginx" { state = "running", enabled = true }
      service "app" { state = "running", enabled = true }
    }
  }

  # Load balancer
  resource "aws_lb" "web" {
    name = "web-lb-${env.name}"
    load_balancer_type = "application"
    subnets = data.aws_vpc.main.public_subnets[*].id
    security_groups = [resource.aws_security_group.web.id]
  }

  resource "aws_lb_target_group" "web" {
    name = "web-targets-${env.name}"
    port = 80
    protocol = "HTTP"
    vpc_id = data.aws_vpc.main.id

    health_check {
      path = "/health"
      interval = 30
    }
  }

  resource "aws_lb_target_group_attachment" "web" {
    for_each = resource.aws_instance.web_servers

    target_group_arn = resource.aws_lb_target_group.web.arn
    target_id = each.value.id
    port = 80
  }

  # Outputs
  output "load_balancer_dns" {
    value = resource.aws_lb.web.dns_name
  }

  output "instance_ips" {
    value = resource.aws_instance.web_servers[*].private_ip
  }
}
```

## Next Steps

1. **Language Grammar**: Define formal syntax (PEG, EBNF)
2. **Parser Implementation**: Use `pest` or `nom` in Rust
3. **Type System**: Design type inference and checking
4. **Standard Library**: Built-in functions and utilities
5. **Provider SDK**: Plugin interface for extensibility
6. **State Backend**: Design state storage (local, S3, etc.)
7. **CLI Tools**: Command-line interface design

## Questions to Consider

1. **Syntax**: How Ruby-like should it be? DSL vs. full language?
2. **Execution Model**: Sequential, parallel, or hybrid?
3. **State Format**: JSON, TOML, custom binary?
4. **Provider Protocol**: gRPC like Terraform? Or simpler?
5. **Modules**: Package manager? Registry?
6. **Secrets**: Built-in secret management or external?
7. **Remote Execution**: Support for remote backends?
8. **Workspace Isolation**: Multiple environments per repo?

## Inspirations & Differentiators

**Learn from:**
- Terraform: Provider ecosystem, state management
- Ansible: Ease of use, agentless architecture
- Pulumi: Real programming language integration
- CloudFormation: Stack abstraction
- SaltStack: Event-driven execution

**Differentiate with:**
- Unified IaC + Config Management
- Superior type safety
- Better testing story
- Cleaner syntax than HCL
- Faster execution (Rust)
- Read-only resource references
- Built-in validation and safety
