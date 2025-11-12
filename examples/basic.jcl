# Basic JCL Examples
# Demonstrates: variables, types, arithmetic, comparisons

# Simple variable assignments
name = "JCL"
version = 1.0
is_stable = true
config_path = null

# Arithmetic expressions
port = 8000 + 80
timeout = 30 * 1000
half = 100 / 2
remainder = 10 % 3

# Comparisons and boolean logic
is_production = version > 0.5
needs_update = version < 2.0 and not is_stable
status = if is_production then "live" else "dev"

# Null coalescing
effective_port = config_path ?? port
fallback_name = null ?? "default" ?? "backup"
