# JCL Script Generation Example
# Demonstrates generating scripts from JCL configuration

# Configuration Data

app_config = (
  name = "myapp",
  version = "2.1.0",
  port = 8080,
  health_check_path = "/health"
)

servers = [
  (name = "web-1", ip = "10.0.1.10", role = "web"),
  (name = "web-2", ip = "10.0.1.11", role = "web"),
  (name = "api-1", ip = "10.0.2.10", role = "api")
]

# Example 1: Generate Bash Deployment Script

web_servers = [s for s in servers if s.role == "web"]

bash_deploy = """
#!/bin/bash
set -euo pipefail

# Configuration
APP_NAME="${app_config.name}"
VERSION="${app_config.version}"
PORT=${app_config.port}

# Servers
${join(["SERVER_${upper(replace(s.name, "-", "_"))}=\"${s.ip}\"" for s in web_servers], "\n")}

# Deploy to all web servers
${join(["echo \"Deploying to ${s.name}...\"" for s in web_servers], "\n")}

echo "All deployments complete!"
"""

deploy = bash_deploy

# Example 2: Generate Python Health Check Script

non_db_servers = [s for s in servers if s.role != "database"]

python_healthcheck = """
#!/usr/bin/env python3
'''Health check script for ${app_config.name}'''

import requests

APP_NAME = "${app_config.name}"
HEALTH_PATH = "${app_config.health_check_path}"

servers = ${jsonencode([
  (name = s.name, host = s.ip, port = app_config.port)
  for s in non_db_servers
])}

for server in servers:
    url = f"http://{server['host']}:{server['port']}{HEALTH_PATH}"
    print(f"Checking {server['name']}...")
"""

healthcheck = python_healthcheck

# Example 3: Generate Nginx Configuration

nginx_upstreams = join([
  "    server ${s.ip}:${app_config.port};"
  for s in web_servers
], "\n")

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
    }

    location ${app_config.health_check_path} {
        proxy_pass http://${app_config.name}_backend${app_config.health_check_path};
        access_log off;
    }
}
"""

nginx = nginx_config

# Example 4: Generate Docker Compose

docker_compose = yamlencode((
  version = "3.8",
  services = (
    web1 = (
      image = "${app_config.name}:${app_config.version}",
      ports = ["${app_config.port}:${app_config.port}"],
      restart = "unless-stopped"
    ),
    web2 = (
      image = "${app_config.name}:${app_config.version}",
      ports = ["${app_config.port}:${app_config.port}"],
      restart = "unless-stopped"
    )
  )
))

docker_compose = docker_compose

# Example 5: Generate Makefile

makefile_targets = """
# Generated Makefile for ${app_config.name}

APP_NAME := ${app_config.name}
VERSION := ${app_config.version}
PORT := ${app_config.port}

.PHONY: help build deploy

help:
\t@echo "Available targets:"
\t@echo "  build  - Build Docker image"
\t@echo "  deploy - Deploy to servers"

build:
\t@echo "Building $(APP_NAME):$(VERSION)..."
\tdocker build -t $(APP_NAME):$(VERSION) .

deploy:
\t@echo "Deploying to servers..."
${join(["\t@echo \"Deploying to ${s.name}...\"" for s in web_servers], "\n")}
\t@echo "Deployment complete!"
"""

Makefile = makefile_targets

# Summary

# This example demonstrates generating scripts in:
# - Bash (deployment)
# - Python (health checks)
# - Nginx configuration
# - Docker Compose (YAML)
# - Makefiles
#
# All generated from a single JCL configuration file!
