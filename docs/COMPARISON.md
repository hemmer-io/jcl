# JCL Comparison with HCL and Ansible

This document compares JCL with HCL (Terraform) and Ansible, identifying useful features we should consider adding.

## Current Status

### ✅ Already Implemented in JCL

**From HCL/Terraform:**
- Variables with types
- String interpolation `${var.name}`
- Functions (50+ built-in)
- Count for resources
- Conditional expressions `condition ? true_val : false_val`
- For expressions / comprehensions
- Data sources (read-only references)
- Depends_on
- Outputs
- Locals (variables)
- Dynamic blocks (via for loops)

**From Ansible:**
- Loops (for-each over lists/maps)
- Conditionals (when/if expressions)
- Templates
- Variables
- Handlers (service restart on change)
- Tags (for selective execution)
- Check mode (dry run via validation)

### 🎯 Should Add from HCL

#### 1. **Lifecycle Rules** (PRIORITY: HIGH)
```
resource.instance = (
  ami = ami-12345
  type = t3.medium

  lifecycle (
    create_before_destroy = true
    prevent_destroy = true
    ignore_changes = (tags)
    replace_triggered_by = (resource.ami)
  )
)
```

#### 2. **Provisioners** (PRIORITY: HIGH)
```
resource.instance = (
  ami = ami-12345

  provisioner.remote_exec (
    inline = [
      "sudo apt-get update",
      "sudo apt-get install -y nginx"
    ]
  )

  provisioner.local_exec (
    command = "echo ${self.public_ip} >> inventory.txt"
  )

  provisioner.file (
    source = "app.conf"
    destination = "/etc/app/app.conf"
  )
)
```

#### 3. **Self References**
```
resource.instance = (
  ami = ami-12345

  # Reference own attributes
  user_data = """
    #!/bin/bash
    echo "My IP is ${self.private_ip}"
  """
)
```

#### 4. **Sensitive Values** (PRIORITY: HIGH)
```
variable "database_password" (
  type = string
  sensitive = true  # Won't be shown in logs/output
)

output.db_connection (
  value = "postgresql://user:${var.database_password}@localhost"
  sensitive = true
)
```

#### 5. **Preconditions/Postconditions** (PRIORITY: MEDIUM)
```
resource.instance = (
  ami = ami-12345

  precondition (
    condition = var.instance_count > 0
    error_message = "Must have at least one instance"
  )

  postcondition (
    condition = self.public_ip != ""
    error_message = "Instance must have a public IP"
  )
)
```

#### 6. **Moved Blocks** (for refactoring)
```
moved (
  from = resource.old_instance
  to = resource.new_instance
)
```

#### 7. **Import Blocks**
```
import (
  id = "i-1234567890abcdef0"
  to = resource.existing_instance
)
```

### 🎯 Should Add from Ansible

#### 1. **Handlers/Notifications** (PRIORITY: HIGH)
```
# Define handlers
handlers (
  restart_nginx (
    service nginx (
      state = restarted
    )
  )

  reload_app (
    command = "systemctl reload myapp"
  )
)

# Trigger handlers
configure (
  file /etc/nginx/nginx.conf (
    content = template("nginx.conf.tpl")
    notify = restart_nginx  # Trigger handler on change
  )

  file /etc/myapp/config.yaml (
    content = yamlencode(config)
    notify = reload_app
  )
)
```

#### 2. **Blocks/Task Grouping** (PRIORITY: MEDIUM)
```
configure (
  # Group related tasks
  block install_packages (
    package nginx (state = present)
    package postgresql (state = present)
    package redis (state = present)

    # Run if any task in block fails
    rescue (
      command "apt-get update"
      # Retry the block
    )

    # Always run, even if block fails
    always (
      command "apt-get clean"
    )
  )
)
```

#### 3. **Register (Capture Output)** (PRIORITY: HIGH)
```
configure (
  # Capture command output
  register check_result = command "nginx -t"

  # Use captured output
  if check_result.rc == 0 (
    service nginx (state = restarted)
  ) else (
    fail "Nginx config test failed: ${check_result.stderr}"
  )
)
```

#### 4. **Delegate To** (PRIORITY: MEDIUM)
```
configure (
  # Run on different host
  command "backup-database" (
    delegate_to = "backup-server.local"
  )
)
```

#### 5. **Run Once** (for cluster operations)
```
configure (
  command "initialize-cluster" (
    run_once = true  # Only run on first host
  )
)
```

#### 6. **Async/Poll** (PRIORITY: MEDIUM)
```
configure (
  command "long-running-task" (
    async = 3600  # Run for up to 1 hour
    poll = 10     # Check status every 10 seconds
  )
)
```

#### 7. **Facts/Host Variables** (PRIORITY: HIGH)
```
# Gather system information
facts = gather_facts()

# Use in configuration
configure (
  if facts.os_family == "Debian" (
    package nginx (state = present)
  ) else if facts.os_family == "RedHat" (
    package nginx (state = present, use = yum)
  )
)
```

#### 8. **Vault/Secrets Management** (PRIORITY: HIGH)
```
# Load encrypted secrets
secrets = vault.decrypt("secrets.vault", key = var.vault_password)

# Use in config
database_password = secrets.db_password
api_key = secrets.api_key
```

#### 9. **Tags for Selective Execution** (PRIORITY: MEDIUM)
```
configure (
  package nginx (
    state = present
    tags = (packages, nginx)
  )

  file /etc/nginx/nginx.conf (
    content = template("nginx.conf.tpl")
    tags = (config, nginx)
  )

  service nginx (
    state = running
    tags = (service, nginx)
  )
)

# Run only specific tags
# jcl apply --tags=config,service
```

#### 10. **Check Mode / Diff Mode** (PRIORITY: HIGH)
```
# Already have plan, but add diff output

resource.file = (
  path = "/etc/config.yaml"
  content = yamlencode(config)

  # Show diff when changed
  diff = true
)
```

### 🎯 Unique Features to Add

#### 1. **Watches/Triggers** (PRIORITY: HIGH)
```
# React to changes
watch resource.config_file (
  on_change = (
    restart service.app
    notify "Config changed"
  )
)
```

#### 2. **Retry Logic** (PRIORITY: HIGH)
```
configure (
  command "curl https://api.example.com/health" (
    retries = 3
    delay = 5
    until = result.rc == 0
  )
)
```

#### 3. **Assertions** (PRIORITY: MEDIUM)
```
# Runtime assertions
assert (
  condition = resource.instance.state == "running"
  message = "Instance must be running"
  severity = error  # or warning
)
```

#### 4. **Rollback on Failure** (PRIORITY: HIGH)
```
stack deployment (
  rollback_on_failure = true

  # If any resource fails, rollback all
  resource.instance = (...)
  resource.database = (...)
)
```

#### 5. **Parallel Execution** (PRIORITY: MEDIUM)
```
configure (
  parallel (
    # These can run simultaneously
    task install_nginx
    task install_postgresql
    task install_redis
  )

  # This runs after all parallel tasks complete
  task configure_services (
    depends_on = (install_nginx, install_postgresql, install_redis)
  )
)
```

#### 6. **State Locking** (PRIORITY: HIGH)
```
state (
  backend = s3
  config = (
    bucket = "my-state-bucket"
    key = "terraform.tfstate"
    region = "us-west-2"
    dynamodb_table = "terraform-locks"
    encrypt = true
  )
)
```

#### 7. **Workspaces** (PRIORITY: MEDIUM)
```
# Multiple state files for same config
workspace dev (
  vars = (instance_type = t3.small)
)

workspace prod (
  vars = (instance_type = t3.large)
)

# jcl workspace select prod
# jcl apply
```

#### 8. **Module Inputs/Outputs** (PRIORITY: HIGH)
```
module networking (
  source = "./modules/vpc"

  # Inputs
  inputs = (
    cidr_block = "10.0.0.0/16"
    availability_zones = ["us-west-2a", "us-west-2b"]
  )
)

# Use module outputs
resource.instance = (
  subnet_id = module.networking.outputs.subnet_id
)
```

#### 9. **Validation Rules** (PRIORITY: MEDIUM)
```
variable "instance_count" (
  type = int
  default = 1

  validation (
    condition = var.instance_count >= 1 && var.instance_count <= 10
    error_message = "Instance count must be between 1 and 10"
  )
)
```

#### 10. **Custom Functions/Macros** (PRIORITY: LOW)
```
# Define reusable functions
fn create_server(name, size) = (
  type = "aws_instance"
  ami = "ami-12345"
  instance_type = size
  tags = (name = name)
)

# Use custom function
resource.web = create_server("web-server", "t3.medium")
resource.api = create_server("api-server", "t3.large")
```

## Recommended Implementation Priority

### Phase 1 (Core Features - High Priority)
1. **Lifecycle rules** - Essential for production use
2. **Sensitive values** - Security requirement
3. **Handlers/notifications** - Common pattern in automation
4. **Register (capture output)** - Needed for conditional logic
5. **Facts gathering** - System information access
6. **Secrets/vault** - Secure credential management
7. **Retry logic** - Handle transient failures
8. **State locking** - Prevent concurrent modifications
9. **Provisioners** - Execute actions during resource lifecycle

### Phase 2 (Enhanced Functionality - Medium Priority)
1. **Preconditions/postconditions** - Better validation
2. **Blocks with rescue/always** - Error handling
3. **Tags for selective execution** - Flexible runs
4. **Parallel execution** - Performance improvement
5. **Workspaces** - Multi-environment support
6. **Module inputs/outputs** - Better modularity
7. **Delegate to** - Multi-host operations
8. **Async/poll** - Long-running tasks

### Phase 3 (Nice to Have - Low Priority)
1. **Watches/triggers** - Reactive automation
2. **Custom functions** - Extensibility
3. **Moved blocks** - Refactoring support
4. **Import blocks** - Existing resource import
5. **Run once** - Cluster operations
6. **Assertions** - Runtime checks
7. **Rollback** - Failure recovery

## Implementation Examples

### For-Each Pattern (Already Supported)

```
# For-each over list
servers = (web-1, web-2, api-1)

for server in servers (
  resource.${server} = (
    ami = ami-12345
    type = t3.medium
  )
)

# For-each over map
server_configs = (
  web = (type = t3.medium, count = 2)
  api = (type = t3.large, count = 3)
  db = (type = r5.large, count = 1)
)

for name, config in server_configs (
  resource.${name} = (
    ami = ami-12345
    instance_type = config.type
    count = config.count
  )
)

# For-each with filter
all_servers = (web-1, web-2, api-1, db-1, cache-1)

for server in all_servers (
  # Only process web servers
  if contains(server, "web") (
    resource.${server} = (...)
  )
)
```

### Handler Pattern (To Implement)

```
# Define handlers that run when notified
handlers (
  restart_nginx (
    service nginx (state = restarted)
    wait_for (port = 80, timeout = 30)
  )

  reload_app (
    command "systemctl reload myapp"
    register = reload_result
    assert reload_result.rc == 0
  )

  clear_cache (
    command "redis-cli FLUSHALL"
    delegate_to = "cache-server"
  )
)

# Trigger handlers when files change
configure (
  file /etc/nginx/nginx.conf (
    content = template("nginx.conf.tpl")
    notify = restart_nginx
  )

  file /etc/nginx/sites-available/myapp (
    content = template("myapp-site.tpl")
    notify = restart_nginx  # Same handler, runs once
  )

  file /opt/app/config.yaml (
    content = yamlencode(app_config)
    notify = (reload_app, clear_cache)  # Multiple handlers
  )
)

# Handlers run at the end, only if notified
```

### Facts/System Info Pattern (To Implement)

```
# Gather system information
facts = gather_facts()

# Facts include:
# - facts.hostname
# - facts.os_family (Debian, RedHat, etc)
# - facts.os_version
# - facts.architecture (x86_64, arm64)
# - facts.memory_total
# - facts.cpu_cores
# - facts.ip_address
# - facts.interfaces

# Use facts in configuration
configure (
  # OS-specific package management
  if facts.os_family == "Debian" (
    package nginx (
      state = present
      use = apt
    )
  ) else if facts.os_family == "RedHat" (
    package nginx (
      state = present
      use = yum
    )
  )

  # Architecture-specific binary
  file /opt/app/binary (
    source = "binary-${facts.architecture}"
    mode = 0755
  )

  # Memory-based tuning
  config_tuning = (
    worker_processes = facts.cpu_cores
    worker_connections = facts.memory_total > 8192 ? 4096 : 1024
  )
)
```

### Secrets Management (To Implement)

```
# Load secrets from vault
secrets = load_secrets("production", (
  vault_addr = "https://vault.example.com"
  auth_method = "token"
  token = env("VAULT_TOKEN")
))

# Or from encrypted file
secrets = decrypt_file("secrets.enc", key = var.vault_key)

# Use secrets (marked sensitive automatically)
database = (
  host = "db.internal"
  username = "appuser"
  password = secrets.db_password  # Sensitive
)

# Secrets are never shown in output
output.db_config = database  # password redacted in output
```

## Summary

JCL should focus on adding:
1. **Handlers/Notifications** - Most valuable from Ansible
2. **Lifecycle Rules** - Essential from Terraform
3. **Sensitive Values** - Security requirement
4. **Facts Gathering** - System introspection
5. **Secrets Management** - Secure credentials
6. **Retry Logic** - Reliability
7. **State Locking** - Concurrency safety

These features would make JCL production-ready while maintaining its clean, human-readable syntax.
