# Large configuration file (~1000 lines)
# Represents a complex multi-tenant platform configuration

app_name = "PlatformService"
version = "5.2.0"
environment = "production"
region = "us-east-1"

# Database sharding configuration
databases = [
    (shard_id = 1, host = "db1.example.com", port = 5432, name = "platform_shard_1", weight = 100),
    (shard_id = 2, host = "db2.example.com", port = 5432, name = "platform_shard_2", weight = 100),
    (shard_id = 3, host = "db3.example.com", port = 5432, name = "platform_shard_3", weight = 100),
    (shard_id = 4, host = "db4.example.com", port = 5432, name = "platform_shard_4", weight = 100),
    (shard_id = 5, host = "db5.example.com", port = 5432, name = "platform_shard_5", weight = 100),
    (shard_id = 6, host = "db6.example.com", port = 5432, name = "platform_shard_6", weight = 100),
    (shard_id = 7, host = "db7.example.com", port = 5432, name = "platform_shard_7", weight = 100),
    (shard_id = 8, host = "db8.example.com", port = 5432, name = "platform_shard_8", weight = 100)
]

# Redis cluster configuration
redis_clusters = [
    (cluster_id = 1, master = "redis-m1.example.com", replicas = ["redis-r1a.example.com", "redis-r1b.example.com"], port = 6379),
    (cluster_id = 2, master = "redis-m2.example.com", replicas = ["redis-r2a.example.com", "redis-r2b.example.com"], port = 6379),
    (cluster_id = 3, master = "redis-m3.example.com", replicas = ["redis-r3a.example.com", "redis-r3b.example.com"], port = 6379),
    (cluster_id = 4, master = "redis-m4.example.com", replicas = ["redis-r4a.example.com", "redis-r4b.example.com"], port = 6379)
]

# Microservices registry
services = [
    (name = "auth-service", url = "https://auth.example.com", instances = 10, timeout = 5, retries = 3, circuit_breaker = true),
    (name = "user-service", url = "https://users.example.com", instances = 15, timeout = 5, retries = 3, circuit_breaker = true),
    (name = "payment-service", url = "https://payments.example.com", instances = 20, timeout = 10, retries = 5, circuit_breaker = true),
    (name = "order-service", url = "https://orders.example.com", instances = 12, timeout = 5, retries = 3, circuit_breaker = true),
    (name = "inventory-service", url = "https://inventory.example.com", instances = 8, timeout = 5, retries = 3, circuit_breaker = true),
    (name = "shipping-service", url = "https://shipping.example.com", instances = 6, timeout = 10, retries = 3, circuit_breaker = true),
    (name = "notification-service", url = "https://notifications.example.com", instances = 10, timeout = 3, retries = 2, circuit_breaker = false),
    (name = "analytics-service", url = "https://analytics.example.com", instances = 5, timeout = 15, retries = 1, circuit_breaker = false),
    (name = "search-service", url = "https://search.example.com", instances = 12, timeout = 5, retries = 3, circuit_breaker = true),
    (name = "recommendation-service", url = "https://recommendations.example.com", instances = 8, timeout = 10, retries = 2, circuit_breaker = true)
]

# Tenant configurations
tenants = [
    (id = 1, name = "acme-corp", plan = "enterprise", active = true, max_users = 1000, features = ["sso", "audit", "api", "webhooks"]),
    (id = 2, name = "globex", plan = "enterprise", active = true, max_users = 500, features = ["sso", "audit", "api", "webhooks"]),
    (id = 3, name = "initech", plan = "professional", active = true, max_users = 100, features = ["sso", "api"]),
    (id = 4, name = "umbrella", plan = "enterprise", active = true, max_users = 2000, features = ["sso", "audit", "api", "webhooks", "custom_domain"]),
    (id = 5, name = "wayne-ent", plan = "enterprise", active = true, max_users = 1500, features = ["sso", "audit", "api", "webhooks"]),
    (id = 6, name = "stark-ind", plan = "professional", active = true, max_users = 300, features = ["sso", "api", "webhooks"]),
    (id = 7, name = "oscorp", plan = "starter", active = true, max_users = 50, features = ["api"]),
    (id = 8, name = "lexcorp", plan = "professional", active = true, max_users = 200, features = ["sso", "api"]),
    (id = 9, name = "soylent", plan = "enterprise", active = true, max_users = 800, features = ["sso", "audit", "api", "webhooks"]),
    (id = 10, name = "tyrell", plan = "professional", active = true, max_users = 150, features = ["sso", "api"])
]

# API endpoints configuration
api_endpoints = [
    (path = "/api/v1/users", methods = ["GET", "POST"], auth = true, rate_limit = 1000, cache_ttl = 60),
    (path = "/api/v1/users/:id", methods = ["GET", "PUT", "DELETE"], auth = true, rate_limit = 1000, cache_ttl = 60),
    (path = "/api/v1/orders", methods = ["GET", "POST"], auth = true, rate_limit = 500, cache_ttl = 0),
    (path = "/api/v1/orders/:id", methods = ["GET", "PUT", "DELETE"], auth = true, rate_limit = 500, cache_ttl = 0),
    (path = "/api/v1/products", methods = ["GET"], auth = false, rate_limit = 5000, cache_ttl = 300),
    (path = "/api/v1/products/:id", methods = ["GET"], auth = false, rate_limit = 5000, cache_ttl = 300),
    (path = "/api/v1/cart", methods = ["GET", "POST", "PUT", "DELETE"], auth = true, rate_limit = 1000, cache_ttl = 0),
    (path = "/api/v1/checkout", methods = ["POST"], auth = true, rate_limit = 100, cache_ttl = 0),
    (path = "/api/v1/payments", methods = ["POST"], auth = true, rate_limit = 100, cache_ttl = 0),
    (path = "/api/v1/search", methods = ["GET"], auth = false, rate_limit = 2000, cache_ttl = 120)
]

# Feature flags with rollout percentages
feature_flags = [
    (name = "new_ui", enabled = true, rollout_pct = 100, tenants = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]),
    (name = "advanced_analytics", enabled = true, rollout_pct = 75, tenants = [1, 2, 4, 5, 9]),
    (name = "ai_recommendations", enabled = true, rollout_pct = 50, tenants = [1, 4, 5]),
    (name = "real_time_notifications", enabled = true, rollout_pct = 100, tenants = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]),
    (name = "export_to_csv", enabled = true, rollout_pct = 100, tenants = [1, 2, 3, 4, 5, 6, 8, 9, 10]),
    (name = "custom_reports", enabled = true, rollout_pct = 80, tenants = [1, 2, 4, 5, 6, 9]),
    (name = "api_v2", enabled = true, rollout_pct = 25, tenants = [1, 4]),
    (name = "mobile_app", enabled = true, rollout_pct = 100, tenants = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]),
    (name = "sso_saml", enabled = true, rollout_pct = 100, tenants = [1, 2, 4, 5, 6, 9]),
    (name = "webhooks_v2", enabled = true, rollout_pct = 60, tenants = [1, 2, 4, 5])
]

# Monitoring and alerting rules
alert_rules = [
    (name = "high_error_rate", condition = "error_rate > 5", severity = "critical", channels = ["pagerduty", "slack"]),
    (name = "high_latency", condition = "p95_latency > 1000", severity = "warning", channels = ["slack"]),
    (name = "database_connections", condition = "db_connections > 80", severity = "warning", channels = ["slack"]),
    (name = "memory_usage", condition = "memory_usage > 90", severity = "critical", channels = ["pagerduty", "slack"]),
    (name = "disk_usage", condition = "disk_usage > 85", severity = "warning", channels = ["slack"]),
    (name = "cpu_usage", condition = "cpu_usage > 80", severity = "warning", channels = ["slack"]),
    (name = "request_queue", condition = "queue_depth > 1000", severity = "critical", channels = ["pagerduty"]),
    (name = "failed_payments", condition = "payment_failure_rate > 10", severity = "critical", channels = ["pagerduty", "slack", "email"]),
    (name = "api_downtime", condition = "uptime < 99.9", severity = "critical", channels = ["pagerduty", "slack", "email"]),
    (name = "cache_hit_rate", condition = "cache_hit_rate < 70", severity = "warning", channels = ["slack"])
]

# Load balancer configuration
load_balancers = [
    (name = "api-lb", algorithm = "round_robin", health_check = "/health", targets = ["api-1", "api-2", "api-3", "api-4"]),
    (name = "auth-lb", algorithm = "least_connections", health_check = "/health", targets = ["auth-1", "auth-2", "auth-3"]),
    (name = "payment-lb", algorithm = "ip_hash", health_check = "/health", targets = ["payment-1", "payment-2", "payment-3", "payment-4", "payment-5"]),
    (name = "search-lb", algorithm = "round_robin", health_check = "/health", targets = ["search-1", "search-2", "search-3", "search-4"])
]

# Message queue configuration
queues = [
    (name = "orders", type = "kafka", topic = "orders", partitions = 12, replication = 3, retention_hours = 168),
    (name = "payments", type = "kafka", topic = "payments", partitions = 8, replication = 3, retention_hours = 168),
    (name = "notifications", type = "rabbitmq", exchange = "notifications", routing_key = "notify.*", durable = true),
    (name = "analytics", type = "kafka", topic = "analytics", partitions = 4, replication = 2, retention_hours = 720),
    (name = "audit_logs", type = "kafka", topic = "audit", partitions = 16, replication = 3, retention_hours = 2160)
]

# Storage buckets configuration
storage_buckets = [
    (name = "user-uploads", provider = "s3", region = "us-east-1", versioning = true, encryption = true, lifecycle_days = 365),
    (name = "invoices", provider = "s3", region = "us-east-1", versioning = true, encryption = true, lifecycle_days = 2555),
    (name = "reports", provider = "s3", region = "us-east-1", versioning = false, encryption = true, lifecycle_days = 90),
    (name = "backups", provider = "s3", region = "us-west-2", versioning = true, encryption = true, lifecycle_days = 730),
    (name = "static-assets", provider = "cloudfront", region = "global", versioning = false, encryption = false, lifecycle_days = 0)
]

# Security policies
security_policies = [
    (name = "password_policy", min_length = 12, require_uppercase = true, require_lowercase = true, require_digits = true, require_special = true, max_age_days = 90),
    (name = "session_policy", max_duration_hours = 8, idle_timeout_minutes = 30, concurrent_sessions = 3),
    (name = "api_key_policy", rotation_days = 90, max_keys_per_user = 5, require_ip_whitelist = false),
    (name = "mfa_policy", required_for = ["admin", "finance"], grace_period_days = 30, backup_codes = 10)
]

# Compliance settings
compliance = (
    gdpr = (
        enabled = true,
        data_retention_days = 730,
        right_to_erasure = true,
        consent_required = true,
        cookie_banner = true
    ),
    hipaa = (
        enabled = true,
        audit_logging = true,
        encryption_at_rest = true,
        encryption_in_transit = true,
        access_controls = true
    ),
    pci_dss = (
        enabled = true,
        tokenization = true,
        secure_transmission = true,
        access_logging = true,
        quarterly_scans = true
    ),
    soc2 = (
        enabled = true,
        continuous_monitoring = true,
        incident_response = true,
        access_reviews = true,
        vendor_management = true
    )
)

# Backup and disaster recovery
backup_config = (
    database_backups = (
        enabled = true,
        frequency = "daily",
        retention_days = 30,
        point_in_time_recovery = true,
        cross_region_replication = true,
        destination_region = "us-west-2"
    ),
    file_backups = (
        enabled = true,
        frequency = "hourly",
        retention_days = 7,
        incremental = true
    ),
    disaster_recovery = (
        enabled = true,
        rto_minutes = 60,
        rpo_minutes = 15,
        failover_region = "us-west-2",
        automated_failover = true
    )
)

# Logging destinations
logging_destinations = [
    (name = "cloudwatch", type = "aws", region = "us-east-1", log_group = "/aws/platform/prod", retention_days = 30),
    (name = "elasticsearch", type = "elasticsearch", url = "https://logs.example.com", index_pattern = "platform-logs-*"),
    (name = "datadog", type = "datadog", api_key = "change_me", service = "platform", environment = "production"),
    (name = "splunk", type = "splunk", url = "https://splunk.example.com", token = "change_me", index = "main")
]

# Cost allocation tags
cost_tags = (
    environment = "production",
    team = "platform",
    project = "core",
    cost_center = "engineering",
    billing_code = "PLAT-001"
)

# Network configuration
network = (
    vpc_cidr = "10.0.0.0/16",
    subnets = [
        (name = "public-1", cidr = "10.0.1.0/24", az = "us-east-1a", public = true),
        (name = "public-2", cidr = "10.0.2.0/24", az = "us-east-1b", public = true),
        (name = "private-1", cidr = "10.0.11.0/24", az = "us-east-1a", public = false),
        (name = "private-2", cidr = "10.0.12.0/24", az = "us-east-1b", public = false),
        (name = "database-1", cidr = "10.0.21.0/24", az = "us-east-1a", public = false),
        (name = "database-2", cidr = "10.0.22.0/24", az = "us-east-1b", public = false)
    ],
    nat_gateways = ["nat-1a", "nat-1b"],
    internet_gateway = "igw-main",
    route_tables = ["rt-public", "rt-private-1a", "rt-private-1b"]
)

# Auto-scaling configuration
autoscaling_groups = [
    (name = "api-asg", min_size = 4, max_size = 20, desired = 8, scale_up_threshold = 70, scale_down_threshold = 30),
    (name = "worker-asg", min_size = 2, max_size = 10, desired = 4, scale_up_threshold = 80, scale_down_threshold = 20),
    (name = "batch-asg", min_size = 1, max_size = 5, desired = 2, scale_up_threshold = 75, scale_down_threshold = 25)
]

# Container orchestration
kubernetes = (
    cluster_name = "platform-prod",
    version = "1.27",
    node_pools = [
        (name = "general", machine_type = "n2-standard-4", min_nodes = 3, max_nodes = 10, disk_size_gb = 100),
        (name = "memory-optimized", machine_type = "n2-highmem-8", min_nodes = 2, max_nodes = 6, disk_size_gb = 200),
        (name = "compute-optimized", machine_type = "n2-highcpu-16", min_nodes = 1, max_nodes = 4, disk_size_gb = 100)
    ],
    ingress_controller = "nginx",
    cert_manager = true,
    service_mesh = "istio",
    monitoring = "prometheus"
)

# CI/CD pipelines
pipelines = [
    (name = "main", trigger = "push", branches = ["main"], stages = ["test", "build", "deploy"], auto_deploy = true),
    (name = "feature", trigger = "pull_request", branches = ["feature/*"], stages = ["test", "build"], auto_deploy = false),
    (name = "release", trigger = "tag", branches = ["v*"], stages = ["test", "build", "deploy", "smoke_test"], auto_deploy = true),
    (name = "hotfix", trigger = "push", branches = ["hotfix/*"], stages = ["test", "build", "deploy"], auto_deploy = true)
]

# SLA definitions
slas = [
    (service = "api", availability = 99.95, latency_p95_ms = 200, latency_p99_ms = 500),
    (service = "auth", availability = 99.99, latency_p95_ms = 100, latency_p99_ms = 300),
    (service = "payment", availability = 99.99, latency_p95_ms = 500, latency_p99_ms = 1000),
    (service = "search", availability = 99.9, latency_p95_ms = 300, latency_p99_ms = 800)
]

# Rate limiting tiers
rate_limit_tiers = [
    (tier = "free", requests_per_hour = 100, burst = 20, cost_per_request = 0.0),
    (tier = "basic", requests_per_hour = 1000, burst = 100, cost_per_request = 0.001),
    (tier = "professional", requests_per_hour = 10000, burst = 500, cost_per_request = 0.0005),
    (tier = "enterprise", requests_per_hour = 100000, burst = 2000, cost_per_request = 0.0001)
]

# Geographic regions
regions = [
    (code = "us-east-1", name = "US East (N. Virginia)", active = true, primary = true),
    (code = "us-west-2", name = "US West (Oregon)", active = true, primary = false),
    (code = "eu-west-1", name = "EU (Ireland)", active = true, primary = false),
    (code = "ap-southeast-1", name = "Asia Pacific (Singapore)", active = true, primary = false),
    (code = "ap-northeast-1", name = "Asia Pacific (Tokyo)", active = false, primary = false)
]

# Third-party integrations
integrations = [
    (name = "stripe", type = "payment", api_key = "sk_live_change_me", webhook_secret = "whsec_change_me", enabled = true),
    (name = "sendgrid", type = "email", api_key = "SG.change_me", from_email = "noreply@example.com", enabled = true),
    (name = "twilio", type = "sms", account_sid = "AC_change_me", auth_token = "change_me", phone_number = "+15555555555", enabled = true),
    (name = "slack", type = "notifications", webhook_url = "https://hooks.slack.com/services/change/me", channel = "#alerts", enabled = true),
    (name = "pagerduty", type = "alerting", integration_key = "change_me", enabled = true),
    (name = "datadog", type = "monitoring", api_key = "change_me", app_key = "change_me", enabled = true)
]

# Job scheduling
scheduled_jobs = [
    (name = "daily_reports", schedule = "0 6 * * *", command = "generate_reports", timeout_minutes = 60, retry_count = 3),
    (name = "data_cleanup", schedule = "0 2 * * *", command = "cleanup_old_data", timeout_minutes = 120, retry_count = 1),
    (name = "backup_verification", schedule = "0 4 * * *", command = "verify_backups", timeout_minutes = 30, retry_count = 2),
    (name = "cache_warmup", schedule = "*/15 * * * *", command = "warm_cache", timeout_minutes = 5, retry_count = 0),
    (name = "health_check", schedule = "*/5 * * * *", command = "run_health_checks", timeout_minutes = 2, retry_count = 0)
]
