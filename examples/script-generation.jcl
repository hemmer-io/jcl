# JCL Script Generation Examples
# Generate scripts in multiple languages from JCL configuration

## Configuration Data

app_config = (
  name = "myapp"
  version = "2.1.0"
  port = 8080
  health_check_path = "/health"
)

servers = (
  (name = "web-1", ip = "10.0.1.10", role = "web"),
  (name = "web-2", ip = "10.0.1.11", role = "web"),
  (name = "api-1", ip = "10.0.2.10", role = "api"),
  (name = "db-1", ip = "10.0.3.10", role = "database")
)

environments = (dev, staging, prod)

env_config = (
  dev = (
    api_url = "http://localhost:8080"
    db_host = "localhost"
    log_level = "debug"
    replicas = 1
  )
  staging = (
    api_url = "https://staging-api.example.com"
    db_host = "staging-db.internal"
    log_level = "info"
    replicas = 2
  )
  prod = (
    api_url = "https://api.example.com"
    db_host = "prod-db.internal"
    log_level = "warn"
    replicas = 3
  )
)

## Example 1: Generate Bash Deployment Script

web_servers = [s for s in servers if s.role == "web"]

bash_deploy = """
#!/bin/bash
set -euo pipefail

# Configuration
APP_NAME="${app_config.name}"
VERSION="${app_config.version}"
PORT=${app_config.port}

# Servers
${[for s in web_servers: "SERVER_${upper(replace(s.name, "-", "_"))}=\"${s.ip}\""] | join "\n"}

# Deploy function
deploy_to_server() {
  local name=$1
  local ip=$2

  echo "Deploying $APP_NAME:$VERSION to $name ($ip)..."

  ssh $ip << 'ENDSSH'
    docker pull $APP_NAME:$VERSION
    docker stop $APP_NAME || true
    docker rm $APP_NAME || true
    docker run -d \\
      --name $APP_NAME \\
      -p $PORT:$PORT \\
      --restart unless-stopped \\
      $APP_NAME:$VERSION
ENDSSH

  echo "Deployment to $name complete!"
}

# Deploy to all web servers
${[for s in web_servers: "deploy_to_server \"${s.name}\" \"${s.ip}\""] | join "\n"}

echo "All deployments complete!"
"""

out.deploy_sh = bash_deploy

## Example 2: Generate Python Health Check Script

python_healthcheck = """
#!/usr/bin/env python3
\"\"\"Health check script for ${app_config.name}\"\"\"

import sys
import requests
from typing import List, Dict

APP_NAME = "${app_config.name}"
HEALTH_PATH = "${app_config.health_check_path}"
TIMEOUT = 5

SERVERS = ${jsonencode([
  (name = s.name, host = s.ip, port = app_config.port)
  for s in servers
  if s.role != "database"
])}

def check_server(server: Dict) -> bool:
    \"\"\"Check if a server is healthy\"\"\"
    url = f"http://{server['host']}:{server['port']}{HEALTH_PATH}"
    try:
        response = requests.get(url, timeout=TIMEOUT)
        if response.status_code == 200:
            print(f"✓ {server['name']} is healthy")
            return True
        else:
            print(f"✗ {server['name']} returned {response.status_code}")
            return False
    except Exception as e:
        print(f"✗ {server['name']} failed: {e}")
        return False

def main():
    print(f"Checking health of {APP_NAME} servers...\\n")

    results = [check_server(server) for server in SERVERS]
    healthy_count = sum(results)
    total_count = len(results)

    print(f"\\nHealthy: {healthy_count}/{total_count}")

    return 0 if all(results) else 1

if __name__ == "__main__":
    sys.exit(main())
"""

out.healthcheck_py = python_healthcheck

## Example 3: Generate Environment-Specific Config Files

for env in environments (
  # Bash environment file
  bash_env_vars = [
    "export ${upper(k)}=\"${v}\""
    for k, v in env_config[env]
  ] | join "\n"

  out.${env}_env_sh = """
#!/bin/bash
# Environment: ${env}
# Generated for ${app_config.name} v${app_config.version}

${bash_env_vars}

# App-specific
export APP_NAME="${app_config.name}"
export APP_VERSION="${app_config.version}"
export APP_PORT=${app_config.port}
"""

  # Python config file
  out.${env}_config_py = """
#!/usr/bin/env python3
\"\"\"Configuration for ${env} environment\"\"\"

# Environment: ${env}
APP_NAME = "${app_config.name}"
APP_VERSION = "${app_config.version}"
APP_PORT = ${app_config.port}

# Environment config
${[for k, v in env_config[env]: "${upper(k)} = ${jsonencode(v)}"] | join "\n"}
"""

  # JSON config
  out.${env}_config_json = jsonencode(merge(
    (
      app_name = app_config.name
      app_version = app_config.version
      app_port = app_config.port
    ),
    env_config[env]
  ))
)

## Example 4: Generate Nginx Configuration

nginx_upstreams = [
  "    server ${s.ip}:${app_config.port};"
  for s in web_servers
] | join "\n"

nginx_config = """
# Generated Nginx configuration for ${app_config.name}

upstream ${app_config.name}_backend {
${nginx_upstreams}
}

server {
    listen 80;
    server_name ${app_config.name}.example.com;

    location / {
        proxy_pass http://${app_config.name}_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    location ${app_config.health_check_path} {
        proxy_pass http://${app_config.name}_backend${app_config.health_check_path};
        access_log off;
    }
}
"""

out.nginx_conf = nginx_config

## Example 5: Generate Docker Compose

docker_services = {
  s.name: (
    image = "${app_config.name}:${app_config.version}"
    ports = ["${app_config.port}:${app_config.port}"]
    environment = (
      NODE_ENV = "production"
      PORT = tostring(app_config.port)
    )
    restart = "unless-stopped"
  )
  for s in web_servers
}

docker_compose = yamlencode((
  version = "3.8"
  services = docker_services
))

out.docker_compose_yml = docker_compose

## Example 6: Generate Systemd Service Files

for server in servers (
  if server.role != "database" (
    out.${server.name}_service = """
[Unit]
Description=${app_config.name} on ${server.name}
After=network.target

[Service]
Type=simple
User=app
WorkingDirectory=/opt/${app_config.name}
ExecStart=/usr/bin/docker run --rm --name ${app_config.name} \\
  -p ${app_config.port}:${app_config.port} \\
  ${app_config.name}:${app_config.version}
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
"""
  )
)

## Example 7: Generate SQL Migration Script

migrations = (
  (
    version = "001"
    description = "Create users table"
    up = """
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255),
    created_at TIMESTAMP DEFAULT NOW()
);
"""
    down = "DROP TABLE users;"
  ),
  (
    version = "002"
    description = "Create posts table"
    up = """
CREATE TABLE posts (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    title VARCHAR(255) NOT NULL,
    content TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);
"""
    down = "DROP TABLE posts;"
  )
)

migration_up = """
-- Migration UP script
-- Generated for ${app_config.name} database

BEGIN;

${[for m in migrations: """
-- Version ${m.version}: ${m.description}
${m.up}
"""] | join "\n"}

COMMIT;
"""

migration_down = """
-- Migration DOWN script
-- Generated for ${app_config.name} database

BEGIN;

${[for m in reverse(migrations): """
-- Rollback ${m.version}: ${m.description}
${m.down}
"""] | join "\n"}

COMMIT;
"""

out.migrate_up_sql = migration_up
out.migrate_down_sql = migration_down

## Example 8: Generate Kubernetes Manifests (as YAML)

k8s_deployment = yamlencode((
  apiVersion = "apps/v1"
  kind = "Deployment"
  metadata = (
    name = app_config.name
    labels = (
      app = app_config.name
      version = app_config.version
    )
  )
  spec = (
    replicas = 3
    selector = (
      matchLabels = (app = app_config.name)
    )
    template = (
      metadata = (
        labels = (
          app = app_config.name
          version = app_config.version
        )
      )
      spec = (
        containers = [
          (
            name = app_config.name
            image = "${app_config.name}:${app_config.version}"
            ports = [
              (containerPort = app_config.port)
            ]
            livenessProbe = (
              httpGet = (
                path = app_config.health_check_path
                port = app_config.port
              )
              initialDelaySeconds = 30
              periodSeconds = 10
            )
          )
        ]
      )
    )
  )
))

out.deployment_yaml = k8s_deployment

## Example 9: Generate Ansible Playbook

ansible_playbook = yamlencode((
  name = "Deploy ${app_config.name}"
  hosts = "webservers"
  become = true
  vars = (
    app_name = app_config.name
    app_version = app_config.version
    app_port = app_config.port
  )
  tasks = [
    (
      name = "Pull Docker image"
      docker_image = (
        name = "{{ app_name }}:{{ app_version }}"
        source = "pull"
      )
    ),
    (
      name = "Stop existing container"
      docker_container = (
        name = "{{ app_name }}"
        state = "absent"
      )
      ignore_errors = true
    ),
    (
      name = "Start new container"
      docker_container = (
        name = "{{ app_name }}"
        image = "{{ app_name }}:{{ app_version }}"
        state = "started"
        restart_policy = "unless-stopped"
        ports = ["{{ app_port }}:{{ app_port }}"]
      )
    )
  ]
))

out.deploy_playbook_yml = ansible_playbook

## Example 10: Generate Makefile

makefile_targets = """
# Generated Makefile for ${app_config.name}

APP_NAME := ${app_config.name}
VERSION := ${app_config.version}
PORT := ${app_config.port}

.PHONY: help build push deploy clean

help:
\t@echo "Available targets:"
\t@echo "  build  - Build Docker image"
\t@echo "  push   - Push to registry"
\t@echo "  deploy - Deploy to servers"
\t@echo "  clean  - Clean up"

build:
\t@echo "Building $(APP_NAME):$(VERSION)..."
\tdocker build -t $(APP_NAME):$(VERSION) .
\tdocker tag $(APP_NAME):$(VERSION) $(APP_NAME):latest

push:
\t@echo "Pushing $(APP_NAME):$(VERSION)..."
\tdocker push $(APP_NAME):$(VERSION)
\tdocker push $(APP_NAME):latest

deploy:
\t@echo "Deploying to servers..."
${[for s in web_servers: "\t@echo \"Deploying to ${s.name}...\"\n\t@ssh ${s.ip} 'docker pull $(APP_NAME):$(VERSION) && docker restart $(APP_NAME)'"] | join "\n"}
\t@echo "Deployment complete!"

clean:
\t@echo "Cleaning up..."
\tdocker rmi $(APP_NAME):$(VERSION) $(APP_NAME):latest || true
"""

out.Makefile = makefile_targets

## Summary

# This example demonstrates generating scripts in:
# - Bash (deployment, environment setup)
# - Python (health checks, configuration)
# - Nginx configuration
# - Docker Compose
# - Systemd service files
# - SQL migrations
# - Kubernetes YAML
# - Ansible playbooks
# - Makefiles

# All generated from a single JCL configuration file!
