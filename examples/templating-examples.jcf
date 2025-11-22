# Templating Examples in JCL
# Demonstrates template-like capabilities using JCL's built-in features

## ============================================================================
## Example 1: Simple Conditional Content (replaces Jinja {% if %})
## ============================================================================

env = "prod"
debug_enabled = false

# Generate application config with conditional sections
app_config = """
[app]
name = "myapp"
environment = ${env}
debug = ${debug_enabled}

${if env == "prod" then """
# Production settings
worker_processes = 4
max_connections = 1000
cache_enabled = true
""" else """
# Development settings
worker_processes = 1
max_connections = 100
cache_enabled = false
"""}
"""

## ============================================================================
## Example 2: Multi-Case Conditionals (replaces Jinja {% elif %})
## ============================================================================

environment = "staging"

database_config = """
[database]
${when environment (
  "prod" => """
host = db.prod.example.com
port = 5432
ssl_mode = require
pool_size = 100
replica_count = 3
"""
  "staging" => """
host = db.staging.example.com
port = 5432
ssl_mode = prefer
pool_size = 50
replica_count = 2
"""
  "dev" => """
host = localhost
port = 5433
ssl_mode = disable
pool_size = 10
replica_count = 1
"""
  * => """
# Default configuration
host = localhost
port = 5432
"""
)}
"""

## ============================================================================
## Example 3: Loops/Iteration (replaces Jinja {% for %})
## ============================================================================

servers = [
  (name = "web-1", ip = "10.0.1.10", port = 8080),
  (name = "web-2", ip = "10.0.1.11", port = 8080),
  (name = "api-1", ip = "10.0.2.10", port = 9000),
  (name = "api-2", ip = "10.0.2.11", port = 9000)
]

# Generate nginx upstream configuration
nginx_upstream = """
upstream backend {
${[
  "    server ${s.ip}:${s.port}; # ${s.name}"
  for s in servers
] | join "\n"}
}
"""

# Generate /etc/hosts entries
hosts_file = """
# Generated hosts file
127.0.0.1 localhost

${[
  "${s.ip} ${s.name}.local ${s.name}"
  for s in servers
] | join "\n"}
"""

## ============================================================================
## Example 4: Conditional Loops (replaces Jinja {% for %} {% if %})
## ============================================================================

services = [
  (name = "web", enabled = true, port = 80, ssl = true),
  (name = "api", enabled = true, port = 8080, ssl = true),
  (name = "admin", enabled = false, port = 9000, ssl = false),
  (name = "metrics", enabled = true, port = 9090, ssl = false)
]

# Generate Docker Compose - only for enabled services
docker_compose = """
version: '3.8'
services:
${[
  """
  ${s.name}:
    image: myapp/${s.name}:latest
    ports:
      - "${s.port}:${s.port}"
${if s.ssl then """    environment:
      - SSL_ENABLED=true
      - SSL_CERT=/certs/${s.name}.crt
""" else ""}    restart: unless-stopped
"""
  for s in services if s.enabled
] | join ""}
"""

## ============================================================================
## Example 5: Nested Conditionals and Loops
## ============================================================================

environments = ["dev", "staging", "prod"]
features = [
  (name = "database", all_envs = true),
  (name = "cache", all_envs = true),
  (name = "monitoring", all_envs = false),  # prod only
  (name = "analytics", all_envs = false),   # prod only
  (name = "debug_toolbar", all_envs = false) # dev only
]

# Generate config for each environment
for env in environments (
  out."features_${env}" = """
# Feature flags for ${env}
[features]
${[
  "${f.name} = ${
    if f.name == "debug_toolbar" then
      (env == "dev" ? "enabled" : "disabled")
    else if f.name == "monitoring" or f.name == "analytics" then
      (env == "prod" ? "enabled" : "disabled")
    else
      "enabled"
  }"
  for f in features
] | join "\n"}
"""
)

## ============================================================================
## Example 6: Generate Scripts with Conditionals
## ============================================================================

deploy_env = "prod"
run_tests = deploy_env != "prod"
enable_rollback = deploy_env == "prod"
dry_run = false

deploy_script = """
#!/bin/bash
set -euo pipefail

ENVIRONMENT="${deploy_env}"
APP_NAME="myapp"
${if dry_run then """
DRY_RUN=true
echo "DRY RUN MODE - No actual changes will be made"
""" else """
DRY_RUN=false
"""}

echo "Deploying to $ENVIRONMENT"

${if run_tests then """
# Run tests before deployment
echo "Running test suite..."
npm test
if [ $? -ne 0 ]; then
    echo "Tests failed, aborting deployment"
    exit 1
fi
""" else """
echo "Skipping tests (production deployment)"
"""}

${if enable_rollback then """
# Create rollback point
echo "Creating rollback snapshot..."
./scripts/create-snapshot.sh
ROLLBACK_ID=$(cat .rollback-id)
echo "Rollback ID: $ROLLBACK_ID"
""" else ""}

# Deploy application
${if dry_run then """
echo "Would deploy: docker push $APP_NAME:latest"
echo "Would restart: docker service update $APP_NAME"
""" else """
docker push $APP_NAME:latest
docker service update --image $APP_NAME:latest $APP_NAME
"""}

echo "Deployment complete!"
"""

## ============================================================================
## Example 7: Complex Nginx Configuration
## ============================================================================

site_name = "example.com"
enable_ssl = true
enable_compression = true
enable_caching = true
backend_servers = [
  (ip = "10.0.1.10", port = 8080, weight = 5),
  (ip = "10.0.1.11", port = 8080, weight = 5),
  (ip = "10.0.1.12", port = 8080, weight = 3)
]

nginx_config = """
# Nginx configuration for ${site_name}

${if enable_ssl then """
# SSL Configuration
ssl_certificate /etc/nginx/ssl/${site_name}.crt;
ssl_certificate_key /etc/nginx/ssl/${site_name}.key;
ssl_protocols TLSv1.2 TLSv1.3;
ssl_ciphers HIGH:!aNULL:!MD5;
ssl_prefer_server_ciphers on;
""" else ""}

${if enable_compression then """
# Compression
gzip on;
gzip_vary on;
gzip_types text/plain text/css application/json application/javascript;
gzip_min_length 1000;
""" else ""}

# Upstream servers
upstream ${site_name}_backend {
${[
  "    server ${s.ip}:${s.port} weight=${s.weight} max_fails=3 fail_timeout=30s;"
  for s in backend_servers
] | join "\n"}
}

server {
    listen ${enable_ssl ? "443 ssl http2" : "80"};
    server_name ${site_name};

${if enable_ssl then """
    # Redirect HTTP to HTTPS
    if ($scheme != "https") {
        return 301 https://$host$request_uri;
    }
""" else ""}

    location / {
        proxy_pass http://${site_name}_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

${if enable_caching then """
        # Caching
        proxy_cache my_cache;
        proxy_cache_valid 200 1h;
        proxy_cache_use_stale error timeout http_500 http_502 http_503;
""" else ""}
    }

    location /health {
        access_log off;
        return 200 "healthy\\n";
    }
}
"""

## ============================================================================
## Example 8: Kubernetes Manifest Generation
## ============================================================================

app_name = "myapp"
namespace = "production"
replicas = 3
enable_autoscaling = true
enable_monitoring = true
enable_service_mesh = false

k8s_deployment = """
apiVersion: apps/v1
kind: Deployment
metadata:
  name: ${app_name}
  namespace: ${namespace}
${if enable_monitoring then """  annotations:
    prometheus.io/scrape: "true"
    prometheus.io/port: "9090"
""" else ""}spec:
  replicas: ${replicas}
  selector:
    matchLabels:
      app: ${app_name}
  template:
    metadata:
      labels:
        app: ${app_name}
${if enable_service_mesh then """        sidecar.istio.io/inject: "true"
""" else ""}    spec:
      containers:
      - name: ${app_name}
        image: ${app_name}:latest
        ports:
        - containerPort: 8080
${if enable_monitoring then """        - containerPort: 9090
          name: metrics
""" else ""}        resources:
          requests:
            memory: "256Mi"
            cpu: "100m"
          limits:
            memory: "512Mi"
            cpu: "500m"
${if enable_autoscaling then """
---
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: ${app_name}
  namespace: ${namespace}
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: ${app_name}
  minReplicas: ${replicas}
  maxReplicas: ${replicas * 3}
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
""" else ""}
"""

## ============================================================================
## Example 9: Infrastructure as Code with Conditionals
## ============================================================================

cloud_provider = "aws"
environment = "prod"
enable_monitoring = true
enable_backup = environment == "prod"
multi_az = environment == "prod"

infrastructure = """
# Infrastructure configuration for ${environment}

${when cloud_provider (
  "aws" => """
[aws]
region = us-west-2
instance_type = ${environment == "prod" ? "t3.large" : "t3.small"}
${if multi_az then """
availability_zones = ["us-west-2a", "us-west-2b", "us-west-2c"]
multi_az_enabled = true
""" else """
availability_zones = ["us-west-2a"]
multi_az_enabled = false
"""}
"""
  "gcp" => """
[gcp]
region = us-central1
machine_type = ${environment == "prod" ? "n1-standard-2" : "n1-standard-1"}
"""
  "azure" => """
[azure]
location = eastus
vm_size = ${environment == "prod" ? "Standard_D2s_v3" : "Standard_B1s"}
"""
  * => "# Cloud provider not configured"
)}

${if enable_monitoring then """
[monitoring]
enabled = true
${when cloud_provider (
  "aws" => "provider = cloudwatch"
  "gcp" => "provider = stackdriver"
  "azure" => "provider = azure-monitor"
  * => "provider = prometheus"
)}
retention_days = ${environment == "prod" ? 90 : 30}
""" else ""}

${if enable_backup then """
[backup]
enabled = true
schedule = "0 2 * * *"  # Daily at 2 AM
retention_count = 30
""" else ""}
"""

## ============================================================================
## Example 10: Generate Terraform-like Configuration
## ============================================================================

regions = ["us-west-2", "us-east-1", "eu-west-1"]
instance_types = (
  prod = "t3.large"
  staging = "t3.medium"
  dev = "t3.small"
)

# Generate VPC configuration for each region
for region in regions (
  out."vpc_${region}" = """
resource "aws_vpc" "${region}_vpc" {
  cidr_block = "10.${when region (
    "us-west-2" => "0"
    "us-east-1" => "1"
    "eu-west-1" => "2"
    * => "99"
  )}.0.0/16"

  tags = {
    Name = "vpc-${region}"
    Region = "${region}"
  }
}

${[
  """
resource "aws_subnet" "${region}_subnet_${az}" {
  vpc_id = aws_vpc.${region}_vpc.id
  cidr_block = "10.${when region (
    "us-west-2" => "0"
    "us-east-1" => "1"
    "eu-west-1" => "2"
    * => "99"
  )}.${i}.0/24"
  availability_zone = "${region}${az}"

  tags = {
    Name = "subnet-${region}-${az}"
  }
}
"""
  for i, az in enumerate(["a", "b", "c"])
] | join "\n"}
"""
)

## ============================================================================
## Example 11: Ansible-like Playbook Generation
## ============================================================================

target_hosts = ["web-1", "web-2", "api-1"]
install_packages = ["nginx", "postgresql", "redis"]
environment_type = "production"

ansible_playbook = """
---
- name: Configure servers
  hosts: ${target_hosts | join ","}
  become: yes

  vars:
    environment: ${environment_type}
    debug_mode: ${environment_type != "production"}

  tasks:
${[
  """
    - name: Install ${pkg}
      apt:
        name: ${pkg}
        state: present
        update_cache: yes
"""
  for pkg in install_packages
] | join "\n"}

${if environment_type == "production" then """
    - name: Configure firewall
      ufw:
        rule: allow
        port: "{{ item }}"
      loop:
        - 22
        - 80
        - 443

    - name: Enable automatic security updates
      apt:
        name: unattended-upgrades
        state: present
""" else """
    - name: Install development tools
      apt:
        name: "{{ item }}"
        state: present
      loop:
        - build-essential
        - git
        - vim
"""}

    - name: Start and enable services
      systemd:
        name: "{{ item }}"
        state: started
        enabled: yes
      loop:
${[
  "        - ${pkg}"
  for pkg in install_packages
] | join "\n"}
"""

## ============================================================================
## Example 12: Multi-Format Output from Same Data
## ============================================================================

app_config_data = (
  name = "myapp"
  version = "2.1.0"
  database = (
    host = "db.local"
    port = 5432
    ssl = true
  )
  cache = (
    host = "redis.local"
    port = 6379
  )
  features = [
    "authentication",
    "authorization",
    "logging",
    "monitoring"
  ]
)

# Output as JSON
out.config_json = jsonencode(app_config_data)

# Output as YAML
out.config_yaml = yamlencode(app_config_data)

# Output as TOML
out.config_toml = tomlencode(app_config_data)

# Output as environment variables
out.config_env = """
APP_NAME=${app_config_data.name}
APP_VERSION=${app_config_data.version}
DB_HOST=${app_config_data.database.host}
DB_PORT=${app_config_data.database.port}
DB_SSL=${app_config_data.database.ssl}
REDIS_HOST=${app_config_data.cache.host}
REDIS_PORT=${app_config_data.cache.port}
"""

# Output as shell script
out.config_sh = """
#!/bin/bash
# Generated configuration

APP_NAME="${app_config_data.name}"
APP_VERSION="${app_config_data.version}"

export DATABASE_URL="postgresql://${app_config_data.database.host}:${app_config_data.database.port}/mydb"
export REDIS_URL="redis://${app_config_data.cache.host}:${app_config_data.cache.port}"

echo "Configuration loaded for $APP_NAME v$APP_VERSION"
"""

## ============================================================================
## Summary
## ============================================================================

# JCL provides powerful templating through:
# 1. String interpolation: ${expr}
# 2. Conditionals: ternary, if/then/else, when
# 3. Loops: list comprehensions with filters
# 4. Multi-line strings: """..."""
# 5. Pipeline operator: | for transformations
# 6. Composition: build complex templates from parts
#
# No need for Jinja-style {% %} syntax - JCL is more powerful and unified!
