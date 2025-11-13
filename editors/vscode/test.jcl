# JCL Syntax Highlighting Test

# Type annotations
name: string = "JCL Project"
version: int = 2
price: float = 99.99
active: bool = true

# Lambda functions
double = x => x * 2
add = (x, y) => x + y
is_positive = n => n > 0

# Higher-order functions
numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
doubled = map(double, numbers)
evens = filter(x => x % 2 == 0, numbers)
sum = reduce((acc, x) => acc + x, numbers, 0)

# String interpolation
greeting = "Hello, ${name}!"
message = "Version ${version} is ${active ? \"active\" : \"inactive\"}"

# Template rendering
config = (
    port=8080,
    host="localhost",
    ssl=true
)
nginx_conf = template("server { listen {{port}}; server_name {{host}}; }", config)

# Function definition
fn factorial(n: int): int = if n <= 1 then 1 else n * factorial(n - 1)

# Operators
result = 10 + 5 * 2
comparison = result > 20
logical = active and comparison
null_check = config?.ssl ?? false
pipe_result = numbers | map(x => x * 2) | filter(x => x > 10)

# Built-in functions
uppercased = upper("hello world")
json_output = jsonencode(config)
hash_value = sha256("data")
file_content = file("config.txt")
