# Conditional Examples
# Demonstrates: if/then/else, when pattern matching, ternary operator

# If expressions
score = 85
grade = if score >= 90 then "A" else if score >= 80 then "B" else "C"

is_passing = score >= 60
status = if is_passing then "PASS" else "FAIL"

# Ternary operator
port = 8080
protocol = port == 443 ? "https" : "http"

# When expressions (pattern matching)
mode = "production"

log_level = when mode (
  "development" => "debug",
  "staging" => "info",
  "production" => "warn",
  _ => "error"
)

# When with literal patterns
value = 75

category = when value (
  0 => "zero",
  75 => "exact match",
  _ => "other"
)

# Nested conditionals
user_role = "admin"
is_authenticated = true

access_level = if is_authenticated then (
  when user_role (
    "admin" => "full",
    "user" => "limited",
    _ => "read-only"
  )
) else "none"
