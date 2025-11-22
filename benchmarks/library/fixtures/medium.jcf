# Medium configuration file (~100 lines)
# Represents a typical microservice configuration

app_name = "OrderService"
version = "2.3.1"
environment = "production"

# Database configuration
database = (
    host = "db.example.com",
    port = 5432,
    name = "orders_db",
    username = "app_user",
    max_connections = 20,
    timeout = 30,
    ssl_enabled = true
)

# Redis cache configuration
cache = (
    host = "redis.example.com",
    port = 6379,
    db = 0,
    ttl = 3600,
    max_retries = 3
)

# API configuration
api = (
    host = "0.0.0.0",
    port = 8080,
    base_path = "/api/v1",
    cors_enabled = true,
    cors_origins = ["https://app.example.com", "https://admin.example.com"],
    rate_limit = 1000,
    timeout = 30
)

# Service dependencies
services = (
    user_service = (
        url = "https://users.example.com",
        timeout = 5,
        retries = 3
    ),
    payment_service = (
        url = "https://payments.example.com",
        timeout = 10,
        retries = 5
    ),
    inventory_service = (
        url = "https://inventory.example.com",
        timeout = 5,
        retries = 3
    ),
    notification_service = (
        url = "https://notifications.example.com",
        timeout = 3,
        retries = 2
    )
)

# Feature flags
features = (
    new_checkout = true,
    express_shipping = true,
    gift_wrapping = false,
    international_orders = true,
    bulk_discounts = true
)

# Logging configuration
logging = (
    level = "info",
    format = "json",
    outputs = ["stdout", "file"],
    file_path = "/var/log/orders.log",
    max_size_mb = 100,
    retention_days = 30
)

# Metrics configuration
metrics = (
    enabled = true,
    provider = "prometheus",
    port = 9090,
    path = "/metrics",
    labels = (
        service = "orders",
        environment = "production",
        region = "us-east-1"
    )
)

# Security configuration
security = (
    jwt_secret = "change_me_in_production",
    jwt_expiry = 3600,
    allowed_origins = ["https://app.example.com"],
    csrf_enabled = true,
    rate_limiting = (
        enabled = true,
        requests_per_minute = 100,
        burst_size = 50
    )
)

# Business rules
rules = (
    min_order_amount = 10.0,
    max_order_amount = 10000.0,
    free_shipping_threshold = 50.0,
    tax_rate = 0.08,
    currency = "USD"
)
