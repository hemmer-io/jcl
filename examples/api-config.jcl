# REST API Configuration Example
# Demonstrates configuration for a REST API service

# API metadata
api = (
    name = "example-api",
    version = "v2",
    base_path = "/api/v2",
    description = "Example REST API configuration"
)

# Environment
env = "production"

# API endpoints configuration
endpoints = [
    (
        path = "/users",
        methods = ["GET", "POST"],
        auth_required = true,
        rate_limit = 100,
        cache_ttl = 300
    ),
    (
        path = "/users/{id}",
        methods = ["GET", "PUT", "DELETE"],
        auth_required = true,
        rate_limit = 50,
        cache_ttl = 600
    ),
    (
        path = "/posts",
        methods = ["GET", "POST"],
        auth_required = true,
        rate_limit = 200,
        cache_ttl = 60
    ),
    (
        path = "/posts/{id}",
        methods = ["GET", "PUT", "DELETE"],
        auth_required = true,
        rate_limit = 100,
        cache_ttl = 120
    ),
    (
        path = "/public/health",
        methods = ["GET"],
        auth_required = false,
        rate_limit = 1000,
        cache_ttl = 0
    ),
    (
        path = "/auth/login",
        methods = ["POST"],
        auth_required = false,
        rate_limit = 10,
        cache_ttl = 0
    )
]

# Authentication configuration
auth = (
    type = "jwt",
    secret_key = env("JWT_SECRET"),
    algorithm = "HS256",
    token_expiry = 3600,  # 1 hour
    refresh_token_expiry = 604800,  # 7 days

    # OAuth providers
    oauth = (
        google = (
            enabled = true,
            client_id = env("GOOGLE_CLIENT_ID"),
            client_secret = env("GOOGLE_CLIENT_SECRET"),
            redirect_uri = "https://api.example.com/auth/google/callback"
        ),
        github = (
            enabled = true,
            client_id = env("GITHUB_CLIENT_ID"),
            client_secret = env("GITHUB_CLIENT_SECRET"),
            redirect_uri = "https://api.example.com/auth/github/callback"
        )
    )
)

# Rate limiting configuration per tier
rate_limits = (
    free = (
        requests_per_minute = 60,
        requests_per_hour = 1000,
        requests_per_day = 10000
    ),
    pro = (
        requests_per_minute = 300,
        requests_per_hour = 10000,
        requests_per_day = 100000
    ),
    enterprise = (
        requests_per_minute = 1000,
        requests_per_hour = 50000,
        requests_per_day = 1000000
    )
)

# Response format templates
response_templates = (
    success = (
        status = "success",
        data = null,
        timestamp = null
    ),
    error = (
        status = "error",
        message = null,
        code = null,
        timestamp = null
    ),
    paginated = (
        status = "success",
        data = null,
        pagination = (
            page = null,
            per_page = null,
            total = null,
            total_pages = null
        ),
        timestamp = null
    )
)

# API documentation configuration
documentation = (
    enabled = true,
    title = "${api.name} Documentation",
    description = api.description,
    version = api.version,

    servers = [
        (
            url = "https://api.example.com${api.base_path}",
            description = "Production server"
        ),
        (
            url = "https://staging-api.example.com${api.base_path}",
            description = "Staging server"
        )
    ],

    contact = (
        name = "API Support",
        email = "api-support@example.com",
        url = "https://example.com/support"
    )
)

# Generate OpenAPI-style endpoint summaries
endpoint_summaries = for endpoint in endpoints do (
    path = "${api.base_path}${endpoint.path}",
    methods = join(endpoint.methods, ", "),
    auth = if endpoint.auth_required then "Required" else "Not Required",
    rate_limit = format("%d requests/min", endpoint.rate_limit)
)

# Security headers
security_headers = (
    "X-Content-Type-Options" = "nosniff",
    "X-Frame-Options" = "DENY",
    "X-XSS-Protection" = "1; mode=block",
    "Strict-Transport-Security" = "max-age=31536000; includeSubDomains",
    "Content-Security-Policy" = "default-src 'self'",
    "Referrer-Policy" = "strict-origin-when-cross-origin"
)

# Middleware pipeline (order matters)
middleware = [
    "request_logger",
    "cors_handler",
    "security_headers",
    "rate_limiter",
    "auth_validator",
    "request_parser",
    "response_formatter",
    "error_handler"
]

# Health check configuration
health_check = (
    enabled = true,
    endpoint = "/api/v2/public/health",
    checks = [
        (name = "database", timeout = 5000),
        (name = "cache", timeout = 2000),
        (name = "external_api", timeout = 3000)
    ]
)

# API metrics to collect
metrics = [
    "request_count",
    "request_duration",
    "response_status_codes",
    "auth_failures",
    "rate_limit_hits",
    "endpoint_usage"
]

# Export configuration summary
config_summary = (
    api_name = api.name,
    version = api.version,
    total_endpoints = length(endpoints),
    authenticated_endpoints = length(
        for e in endpoints if e.auth_required do e
    ),
    public_endpoints = length(
        for e in endpoints if !e.auth_required do e
    ),
    middleware_count = length(middleware),
    oauth_providers = length(keys(auth.oauth))
)
