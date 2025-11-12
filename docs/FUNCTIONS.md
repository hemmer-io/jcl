# JCL Built-in Functions Reference

JCL provides a comprehensive standard library of built-in functions for data manipulation, encoding/decoding, string operations, and more.

## Table of Contents

- [String Functions](#string-functions)
- [Encoding Functions](#encoding-functions)
- [Collection Functions](#collection-functions)
- [Numeric Functions](#numeric-functions)
- [Type Conversion](#type-conversion)
- [Hash Functions](#hash-functions)
- [Date/Time Functions](#datetime-functions)
- [Filesystem Functions](#filesystem-functions)
- [Template Functions](#template-functions)
- [Utility Functions](#utility-functions)

## String Functions

### upper(string) -> string
Converts a string to uppercase.

```
upper("hello")  # "HELLO"
```

### lower(string) -> string
Converts a string to lowercase.

```
lower("WORLD")  # "world"
```

### trim(string) -> string
Removes leading and trailing whitespace.

```
trim("  hello  ")  # "hello"
```

### trimprefix(string, prefix) -> string
Removes a prefix from a string.

```
trimprefix("hello world", "hello ")  # "world"
```

### trimsuffix(string, suffix) -> string
Removes a suffix from a string.

```
trimsuffix("hello world", " world")  # "hello"
```

### replace(string, old, new) -> string
Replaces all occurrences of old with new.

```
replace("hello world", "world", "everyone")  # "hello everyone"
```

### split(string, separator) -> list
Splits a string by separator.

```
split("a,b,c", ",")  # ["a", "b", "c"]
```

### join(list, separator) -> string
Joins list elements with separator.

```
join(["a", "b", "c"], "-")  # "a-b-c"
```

### substr(string, start, length) -> string
Extracts a substring.

```
substr("hello world", 0, 5)  # "hello"
substr("hello world", 6, 5)  # "world"
```

### strlen(string) -> int
Returns the length of a string.

```
strlen("hello")  # 5
```

### format(format_string, args...) -> string
Printf-style string formatting.

```
format("Hello, %s!", "World")  # "Hello, World!"
```

## Encoding Functions

### jsonencode(value) -> string
Encodes a value as JSON.

```
jsonencode((name=myapp version=1.0))  # '{"name":"myapp","version":"1.0"}'
```

### jsondecode(string) -> value
Decodes a JSON string.

```
jsondecode('{"name":"myapp"}')  # (name=myapp)
```

### yamlencode(value) -> string
Encodes a value as YAML.

```
yamlencode((name=myapp version=1.0))
# name: myapp
# version: 1.0
```

### yamldecode(string) -> value
Decodes a YAML string.

```
yamldecode("name: myapp\nversion: 1.0")  # (name=myapp version=1.0)
```

### tomlencode(value) -> string
Encodes a value as TOML.

```
tomlencode((name=myapp version=1.0))
# name = "myapp"
# version = "1.0"
```

### tomldecode(string) -> value
Decodes a TOML string.

```
tomldecode('name = "myapp"\nversion = "1.0"')  # (name=myapp version=1.0)
```

### base64encode(string) -> string
Encodes a string as Base64.

```
base64encode("hello")  # "aGVsbG8="
```

### base64decode(string) -> string
Decodes a Base64 string.

```
base64decode("aGVsbG8=")  # "hello"
```

### urlencode(string) -> string
URL-encodes a string.

```
urlencode("hello world")  # "hello%20world"
```

### urldecode(string) -> string
URL-decodes a string.

```
urldecode("hello%20world")  # "hello world"
```

## Collection Functions

### length(collection) -> int
Returns the length of a string, list, or map.

```
length("hello")         # 5
length([1, 2, 3])       # 3
length((a=1 b=2))       # 2
```

### contains(collection, value) -> bool
Checks if a collection contains a value.

```
contains([1, 2, 3], 2)        # true
contains("hello", "ell")      # true
```

### keys(map) -> list
Returns the keys of a map as a list.

```
keys((name=myapp version=1.0))  # ["name", "version"]
```

### values(map) -> list
Returns the values of a map as a list.

```
values((name=myapp version=1.0))  # ["myapp", "1.0"]
```

### merge(map1, map2, ...) -> map
Merges multiple maps together.

```
merge((a=1), (b=2), (c=3))  # (a=1 b=2 c=3)
```

### lookup(map, key, default?) -> value
Looks up a key in a map, returns default if not found.

```
lookup((name=myapp), "name")      # "myapp"
lookup((name=myapp), "version", "1.0")  # "1.0"
```

### reverse(list) -> list
Reverses a list.

```
reverse([1, 2, 3])  # [3, 2, 1]
```

### sort(list) -> list
Sorts a list.

```
sort([3, 1, 2])          # [1, 2, 3]
sort(["c", "a", "b"])    # ["a", "b", "c"]
```

### distinct(list) -> list
Removes duplicates from a list.

```
distinct([1, 2, 2, 3, 1])  # [1, 2, 3]
```

### flatten(list) -> list
Flattens a nested list by one level.

```
flatten([[1, 2], [3, 4]])  # [1, 2, 3, 4]
```

### compact(list) -> list
Removes null values from a list.

```
compact([1, null, 2, null, 3])  # [1, 2, 3]
```

## Numeric Functions

### min(numbers...) -> number
Returns the minimum value.

```
min(1, 2, 3)      # 1
min([1, 2, 3])    # 1
```

### max(numbers...) -> number
Returns the maximum value.

```
max(1, 2, 3)      # 3
max([1, 2, 3])    # 3
```

### sum(list) -> number
Returns the sum of numbers in a list.

```
sum([1, 2, 3, 4])  # 10
```

### avg(list) -> number
Returns the average of numbers in a list.

```
avg([1, 2, 3, 4])  # 2.5
```

### abs(number) -> number
Returns the absolute value.

```
abs(-5)  # 5
```

### ceil(number) -> int
Rounds up to nearest integer.

```
ceil(3.2)  # 4
```

### floor(number) -> int
Rounds down to nearest integer.

```
floor(3.8)  # 3
```

### round(number) -> int
Rounds to nearest integer.

```
round(3.5)  # 4
round(3.4)  # 3
```

## Type Conversion

### tostring(value) -> string
Converts a value to a string.

```
tostring(123)    # "123"
tostring(true)   # "true"
```

### tonumber(string) -> number
Converts a string to a number.

```
tonumber("123")    # 123
tonumber("3.14")   # 3.14
```

### tobool(value) -> bool
Converts a value to a boolean.

```
tobool("true")   # true
tobool(1)        # true
tobool(0)        # false
```

### tolist(value) -> list
Converts a value to a list.

```
tolist((a=1 b=2))  # [1, 2]
```

### tomap(list) -> map
Converts a list to a map.

```
tomap([["a", 1], ["b", 2]])  # (a=1 b=2)
```

## Hash Functions

### md5(string) -> string
Computes MD5 hash.

```
md5("hello")  # "5d41402abc4b2a76b9719d911017c592"
```

### sha1(string) -> string
Computes SHA-1 hash.

```
sha1("hello")  # "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d"
```

### sha256(string) -> string
Computes SHA-256 hash.

```
sha256("hello")  # "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
```

### sha512(string) -> string
Computes SHA-512 hash.

```
sha512("hello")  # (long hash string)
```

## Date/Time Functions

### timestamp() -> int
Returns the current Unix timestamp.

```
timestamp()  # 1234567890
```

### formatdate(format, timestamp) -> string
Formats a timestamp as a date string.

```
formatdate("2006-01-02", 1234567890)  # "2009-02-13"
```

### timeadd(timestamp, duration) -> int
Adds a duration to a timestamp.

```
timeadd(timestamp(), "1h")   # timestamp + 1 hour
timeadd(timestamp(), "30m")  # timestamp + 30 minutes
```

## Filesystem Functions

### file(path) -> string
Reads a file and returns its contents.

```
file("config.txt")  # contents of config.txt
```

### fileexists(path) -> bool
Checks if a file exists.

```
fileexists("config.txt")  # true or false
```

### dirname(path) -> string
Returns the directory name of a path.

```
dirname("/path/to/file.txt")  # "/path/to"
```

### basename(path) -> string
Returns the file name of a path.

```
basename("/path/to/file.txt")  # "file.txt"
```

### abspath(path) -> string
Returns the absolute path.

```
abspath("./file.txt")  # "/full/path/to/file.txt"
```

## Template Functions

### template(template_string, vars?) -> string
Renders a template string with variables.

```
template("Hello, {{name}}!", (name="World"))  # "Hello, World!"
```

### templatefile(path, vars?) -> string
Renders a template file with variables.

```
templatefile("nginx.conf.tpl", (port=8080 server_name="myapp"))
```

## Utility Functions

### range(start?, end) -> list
Generates a range of numbers.

```
range(5)        # [0, 1, 2, 3, 4]
range(1, 5)     # [1, 2, 3, 4]
range(0, 10, 2) # [0, 2, 4, 6, 8]
```

### zipmap(keys, values) -> map
Creates a map from two lists.

```
zipmap(["a", "b", "c"], [1, 2, 3])  # (a=1 b=2 c=3)
```

### coalesce(values...) -> value
Returns the first non-null value.

```
coalesce(null, null, "default")  # "default"
coalesce("first", "second")      # "first"
```

### try(expression, default?) -> value
Tries to evaluate an expression, returns default on error.

```
try(jsondecode("invalid json"), null)  # null
try(divide(10, 0), 0)                  # 0
```

## Pipeline Operator

JCL supports the pipeline operator `|` to chain function calls:

```
result = data
  | upper
  | split " "
  | sort
  | join "-"

# Equivalent to:
result = join(sort(split(upper(data), " ")), "-")
```

With lambdas:

```
numbers = [1, 2, 3, 4, 5]
  | map x => x * 2
  | filter x => x > 5
  | sum

# Result: 18 (6 + 8 + 10)
```

## Lambda Expressions

Many collection functions accept lambda expressions:

```
# Map
doubled = map([1, 2, 3], x => x * 2)  # [2, 4, 6]

# Filter
evens = filter([1, 2, 3, 4], x => x % 2 == 0)  # [2, 4]

# Reduce
sum = reduce([1, 2, 3], 0, (acc, x) => acc + x)  # 6
```

## Function Composition

Functions can be composed using pipelines:

```
process_data = data
  | trim
  | lower
  | split ","
  | map x => trim(x)
  | filter x => strlen(x) > 0
  | distinct
  | sort
  | join ", "
```

## Error Handling

Use `try` for safe function calls:

```
# Safe JSON parsing
config = try(jsondecode(file("config.json")), (
  name = "default"
  version = "1.0"
))

# Safe lookups with defaults
port = lookup(config, "port", 8080)
```
