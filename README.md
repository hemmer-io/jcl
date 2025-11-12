# JCL - Jack Configuration Language

A modern, safe, and flexible configuration language for infrastructure as code and configuration management, written in Rust.

## Vision

JCL bridges the gap between infrastructure provisioning (like Terraform) and configuration management (like Ansible/Puppet), providing a unified, type-safe language that's easy to use yet powerful enough for complex deployments.

## Key Features

🎯 **Unified IaC + Config Management**
- Provision infrastructure and configure systems in one language
- No context switching between tools

🔒 **Safety First**
- Strong type system with inference
- Immutability by default
- Validation at every stage
- Dry-run and plan before apply

🚀 **Developer Friendly**
- Ruby-inspired clean syntax
- Go template-like built-in functions
- Clear error messages
- Fast execution (Rust-powered)

🏗️ **Powerful Abstractions**
- Environment, Stack, and Resource hierarchy
- Read-only resource references
- Reusable modules
- Dependency management

## Example

```ruby
environment "production" {
  region = "us-west-2"

  variables {
    instance_type = "t3.large"
    app_version = "1.2.3"
  }
}

stack "web_app" {
  environment = env.production

  # Reference existing VPC (read-only)
  data "aws_vpc" "main" {
    tags = { name = "main-vpc" }
    lifecycle { managed = false }
  }

  # Provision infrastructure
  resource "aws_instance" "web_server" {
    ami = "ami-12345"
    instance_type = env.vars.instance_type
    vpc_id = data.aws_vpc.main.id

    # Configure the instance
    configure {
      package "nginx" { state = "present" }

      file "/etc/nginx/nginx.conf" {
        content = template("./nginx.conf.tpl")
        mode = "0644"
      }

      service "nginx" {
        state = "running"
        enabled = true
      }
    }
  }
}
```

## Project Status

🚧 **Early Design Phase** 🚧

This project is in the early design and planning phase. We're currently:
- Defining the language syntax and grammar
- Architecting the core components
- Gathering feedback from the community

See [DESIGN.md](./DESIGN.md) for the detailed design document.

## Core Concepts

### Environment
A deployment context (dev, staging, prod) with associated variables and configuration.

### Stack
A logical grouping of related resources that are managed together.

### Resources
Infrastructure or configuration items to be created and managed.

### Data Sources
References to existing resources that are read-only (not managed).

## Architecture

```
CLI → Parser → Type Checker → Evaluator → Planner → Providers → State
```

Built in Rust for:
- Memory safety and performance
- Strong type system
- Excellent concurrency
- Rich ecosystem

## Roadmap

- [ ] Language specification and grammar
- [ ] Parser implementation
- [ ] Type system and checker
- [ ] Evaluation engine
- [ ] Dependency graph / planner
- [ ] Provider interface design
- [ ] State management
- [ ] AWS provider (initial)
- [ ] Configuration management provider
- [ ] CLI tools
- [ ] Documentation and examples
- [ ] Testing framework
- [ ] Module system and registry

## Why JCL?

**vs. Terraform:**
- Better syntax (more readable, less verbose)
- Unified config management
- Stronger type safety
- Read-only resource references built-in

**vs. Ansible:**
- Infrastructure provisioning built-in
- Better dependency management
- Type safety
- Declarative for both IaC and config

**vs. Pulumi:**
- Purpose-built DSL (simpler than full programming language)
- Consistent experience across all use cases
- No need to learn language-specific cloud SDKs

## Contributing

This project is in early stages. We welcome:
- Feedback on the design
- Syntax suggestions
- Use case examples
- Architecture discussions

Please see [DESIGN.md](./DESIGN.md) for the current design thinking.

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Contact

Project is in early development. More information coming soon!
