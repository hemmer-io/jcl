# Web Application Configuration Example
# This example shows a complete web application configuration

# Environment-specific settings
environment = "production"
region = "us-west-2"

# Application metadata
app = (
    name = "my-web-app",
    version = "2.1.0",
    description = "Example web application with JCL configuration"
)

# Server configuration with environment-based values
server = (
    host = "0.0.0.0",
    port = if environment == "production" then 80 else 8080,
    workers = if environment == "production" then 8 else 2,
    timeout = 30,
    max_connections = if environment == "production" then 1000 else 100,

    # TLS settings
    tls = (
        enabled = environment == "production",
        cert_path = "/etc/ssl/certs/app.crt",
        key_path = "/etc/ssl/private/app.key"
    )
)

# Database configuration
database = (
    # Connection string with interpolation
    host = "db.${environment}.${region}.example.com",
    port = 5432,
    name = "${app.name}_${environment}",
    username = "app_user",

    # Pool settings based on environment
    pool = (
        min_size = if environment == "production" then 10 else 2,
        max_size = if environment == "production" then 50 else 10,
        timeout = 30
    ),

    # SSL for production only
    ssl_mode = if environment == "production" then "require" else "prefer"
)

# Cache configuration
cache = (
    type = "redis",
    host = "cache.${environment}.${region}.example.com",
    port = 6379,
    ttl = if environment == "production" then 3600 else 300,
    max_memory = if environment == "production" then "2gb" else "512mb"
)

# Feature flags
features = (
    new_ui = true,
    beta_api = environment != "production",
    analytics = true,
    debug_mode = environment != "production",
    rate_limiting = environment == "production",
    maintenance_mode = false
)

# CORS configuration
cors = (
    enabled = true,
    allowed_origins = if environment == "production" then [
        "https://example.com",
        "https://www.example.com",
        "https://app.example.com"
    ] else [
        "http://localhost:3000",
        "http://localhost:8080"
    ],
    allowed_methods = ["GET", "POST", "PUT", "DELETE", "OPTIONS"],
    allowed_headers = ["Content-Type", "Authorization"],
    max_age = 3600
)

# Logging configuration
logging = (
    level = if features.debug_mode then "debug" else "info",
    format = "json",
    output = if environment == "production" then "stdout" else "file",
    file_path = "./logs/${app.name}.log",

    # Structured logging fields
    fields = (
        app = app.name,
        version = app.version,
        environment = environment,
        region = region
    )
)

# Monitoring and metrics
monitoring = (
    enabled = environment == "production",
    endpoint = "https://metrics.example.com/push",
    interval = 60,

    metrics = [
        "http_requests_total",
        "http_request_duration_seconds",
        "database_connections_active",
        "cache_hit_rate"
    ]
)

# Build connection strings using functions
db_connection_string = format(
    "postgresql://%s@%s:%d/%s?sslmode=%s",
    database.username,
    database.host,
    database.port,
    database.name,
    database.ssl_mode
)

cache_connection_string = format(
    "redis://%s:%d",
    cache.host,
    cache.port
)

# Generate unique deployment ID
deployment_id = sha256(format(
    "%s-%s-%d",
    app.name,
    app.version,
    timestamp()
))

# Export configuration summary
summary = (
    app = app.name,
    version = app.version,
    environment = environment,
    deployment_id = deployment_id,
    server_url = format("http://%s:%d", server.host, server.port),
    database_url = db_connection_string,
    features_enabled = length(keys(features))
)
