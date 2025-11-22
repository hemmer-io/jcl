# Collection Examples
# Demonstrates: lists, maps, comprehensions, member access

# Lists
numbers = [1, 2, 3, 4, 5]
names = ["Alice", "Bob", "Charlie"]
mixed = [1, "two", true, null]
empty = []

# Maps (using parentheses)
server = (
  host = "localhost",
  port = 8080,
  ssl = true
)

database = (
  type: "postgres",
  host: "db.example.com",
  port: 5432,
  name: "myapp"
)

# List comprehensions
evens = [x for x in numbers if x % 2 == 0]
doubled = [x * 2 for x in numbers]
squares = [x * x for x in numbers if x > 2]

# Member access
db_host = database.host
db_port = database.port
server_url = server.host

# Index access
first = numbers[0]
second = names[1]

# Nested collections
config = (
  servers = [
    (name = "web1", port = 8080),
    (name = "web2", port = 8081)
  ],
  database = database
)

first_server = config.servers[0]
first_server_name = first_server.name
