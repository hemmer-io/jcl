---
layout: default
title: Built-in Functions Reference
parent: Reference
nav_order: 2
permalink: /reference/functions/
---


JCL provides 80+ built-in functions organized by category. All functions are globally available and can be used in any expression.

## Table of Contents

- [String Functions](#string-functions)
- [List Functions](#list-functions)
- [Map Functions](#map-functions)
- [Set Operations](#set-operations)
- [Encoding Functions](#encoding-functions)
- [Hashing Functions](#hashing-functions)
- [Numeric Functions](#numeric-functions)
- [Type Conversion Functions](#type-conversion-functions)
- [Type Introspection](#type-introspection)
- [Boolean Aggregation](#boolean-aggregation)
- [Date/Time Functions](#datetime-functions)
- [File Functions](#file-functions)
- [Template Functions](#template-functions)
- [Utility Functions](#utility-functions)
- [Higher-Order & Streaming Functions](#higher-order--streaming-functions) ⭐ NEW
- [Advanced Functions](#advanced-functions)

---

## String Functions

### upper

Convert a string to uppercase.

```jcl
upper("hello")  # "HELLO"
upper("World")  # "WORLD"
```

### lower

Convert a string to lowercase.

```jcl
lower("HELLO")  # "hello"
lower("World")  # "world"
```

### trim

Remove leading and trailing whitespace from a string.

```jcl
trim("  hello  ")  # "hello"
trim("\n\tworld\t\n")  # "world"
```

### trimprefix

Remove a prefix from a string if it exists.

```jcl
trimprefix("hello world", "hello ")  # "world"
trimprefix("test", "hello")  # "test"
```

### trimsuffix

Remove a suffix from a string if it exists.

```jcl
trimsuffix("hello.txt", ".txt")  # "hello"
trimsuffix("test", ".txt")  # "test"
```

### replace

Replace all occurrences of a substring with another string.

```jcl
replace("hello world", "world", "JCL")  # "hello JCL"
replace("aaa", "a", "b")  # "bbb"
```

### split

Split a string into a list by a delimiter.

```jcl
split("a,b,c", ",")  # ["a", "b", "c"]
split("hello world", " ")  # ["hello", "world"]
```

### join

Join a list of strings with a delimiter.

```jcl
join(["a", "b", "c"], ",")  # "a,b,c"
join(["hello", "world"], " ")  # "hello world"
```

### format

Format a string with printf-style formatting.

**Format Specifiers:**
- `%s` - String
- `%d`, `%i` - Integer
- `%f` - Float
- `%b` - Boolean
- `%v` - Any value
- `%x` - Hexadecimal (lowercase)
- `%X` - Hexadecimal (uppercase)
- `%o` - Octal
- `%%` - Literal percent sign

```jcl
format("Hello, %s!", "World")  # "Hello, World!"
format("Number: %d", 42)  # "Number: 42"
format("Pi: %f", 3.14159)  # "Pi: 3.14159"
format("100%% complete")  # "100% complete"
```

### substr

Extract a substring from a string.

```jcl
substr("hello world", 0, 5)  # "hello"
substr("hello world", 6, 5)  # "world"
```

### strlen

Get the length of a string.

```jcl
strlen("hello")  # 5
strlen("")  # 0
```

### indent

Add indentation to each line of a string.

```jcl
# Indent all lines by 2 spaces
text = "line1
line2
line3"
indent(text, 2)
# Result:
# "  line1
#   line2
#   line3"

# Don't indent the first line
indent(text, 2, false)
# Result:
# "line1
#   line2
#   line3"
```

**Parameters:**
- `str` (string): The string to indent
- `num_spaces` (int): Number of spaces to add
- `indent_first` (bool, optional): Whether to indent first line (default: true)

### chomp

Remove trailing newlines from a string.

```jcl
chomp("hello
")  # "hello"

chomp("hello


")  # "hello"

chomp("hello")  # "hello" (no change if no trailing newlines)
```

### strrev

Reverse a string.

```jcl
strrev("hello")  # "olleh"
strrev("JCL")  # "LCJ"
strrev("")  # ""
```

### title

Capitalize the first letter of each word.

```jcl
title("hello world")  # "Hello World"
title("the quick brown fox")  # "The Quick Brown Fox"
title("Hello World")  # "Hello World"
```

---

## List Functions

### length

Get the length of a list or map.

```jcl
length([1, 2, 3])  # 3
length([])  # 0
length((a = 1, b = 2))  # 2
```

### contains

Check if a list contains a value or a map contains a key.

```jcl
contains([1, 2, 3], 2)  # true
contains([1, 2, 3], 4)  # false
contains((a = 1, b = 2), "a")  # true
```

### reverse

Reverse a list.

```jcl
reverse([1, 2, 3])  # [3, 2, 1]
reverse(["a", "b", "c"])  # ["c", "b", "a"]
```

### sort

Sort a list in ascending order.

```jcl
sort([3, 1, 2])  # [1, 2, 3]
sort(["c", "a", "b"])  # ["a", "b", "c"]
```

### slice

Extract a slice from a list.

```jcl
slice([1, 2, 3, 4, 5], 1, 3)  # [2, 3]
slice(["a", "b", "c", "d"], 0, 2)  # ["a", "b"]
```

### distinct

Remove duplicate values from a list.

```jcl
distinct([1, 2, 2, 3, 1])  # [1, 2, 3]
distinct(["a", "b", "a"])  # ["a", "b"]
```

### flatten

Flatten nested lists into a single list.

```jcl
flatten([[1, 2], [3, 4]])  # [1, 2, 3, 4]
flatten([[1], [2, [3, 4]]])  # [1, 2, [3, 4]]  (one level only)
```

### compact

Remove null values from a list.

```jcl
compact([1, null, 2, null, 3])  # [1, 2, 3]
compact(["a", null, "b"])  # ["a", "b"]
```

---

## Map Functions

### keys

Get all keys from a map.

```jcl
keys((a = 1, b = 2, c = 3))  # ["a", "b", "c"]
```

### values

Get all values from a map.

```jcl
values((a = 1, b = 2, c = 3))  # [1, 2, 3]
```

### merge

Merge multiple maps into one (later maps override earlier ones).

```jcl
merge((a = 1), (b = 2))  # (a = 1, b = 2)
merge((a = 1, b = 2), (b = 3, c = 4))  # (a = 1, b = 3, c = 4)
```

### lookup

Look up a key in a map with a default value if not found.

```jcl
lookup((a = 1, b = 2), "a", 0)  # 1
lookup((a = 1, b = 2), "c", 0)  # 0
```

---

## Set Operations

### setunion

Return the union of multiple sets (removes duplicates).

```jcl
a = [1, 2, 3]
b = [3, 4, 5]
setunion(a, b)  # [1, 2, 3, 4, 5]

# Works with multiple sets
setunion([1, 2], [2, 3], [3, 4])  # [1, 2, 3, 4]
```

### setintersection

Return the intersection of sets (elements common to all).

```jcl
a = [1, 2, 3, 4]
b = [3, 4, 5, 6]
setintersection(a, b)  # [3, 4]

# Works with multiple sets
setintersection([1, 2, 3], [2, 3, 4], [3, 4, 5])  # [3]
```

### setdifference

Return elements in the first set that are not in the second.

```jcl
a = [1, 2, 3, 4]
b = [3, 4, 5]
setdifference(a, b)  # [1, 2]
```

### setsymmetricdifference

Return elements that are in one set but not both (symmetric difference).

```jcl
a = [1, 2, 3]
b = [2, 3, 4]
setsymmetricdifference(a, b)  # [1, 4]
```

**Use Cases:**
```jcl
# Find unique values across lists
all_values = setunion(list1, list2, list3)

# Find common configuration keys
common_keys = setintersection(keys(config1), keys(config2))

# Find missing dependencies
missing = setdifference(required_deps, installed_deps)

# Find differences between two configurations
changes = setsymmetricdifference(keys(old_config), keys(new_config))
```

---

## Encoding Functions

### base64_encode

Encode a string to Base64.

```jcl
base64_encode("hello")  # "aGVsbG8="
```

### base64_decode

Decode a Base64 string.

```jcl
base64_decode("aGVsbG8=")  # "hello"
```

### jsonencode

Encode a value to JSON.

```jcl
jsonencode((name = "Alice", age = 30))  # '{"name":"Alice","age":30}'
jsonencode([1, 2, 3])  # '[1,2,3]'
```

### jsondecode

Decode JSON to a JCL value.

```jcl
jsondecode('{"name":"Alice"}')  # (name = "Alice")
jsondecode('[1,2,3]')  # [1, 2, 3]
```

### yamlencode

Encode a value to YAML.

```jcl
yamlencode((name = "Alice", age = 30))  # 'name: Alice\nage: 30\n'
```

### yamldecode

Decode YAML to a JCL value.

```jcl
yamldecode('name: Alice\nage: 30')  # (name = "Alice", age = 30)
```

### tomlencode

Encode a value to TOML.

```jcl
tomlencode((name = "Alice", age = 30))  # 'name = "Alice"\nage = 30\n'
```

### tomldecode

Decode TOML to a JCL value.

```jcl
tomldecode('name = "Alice"\nage = 30')  # (name = "Alice", age = 30)
```

### urlencode

URL-encode a string.

```jcl
urlencode("hello world")  # "hello%20world"
urlencode("a+b=c")  # "a%2Bb%3Dc"
```

### urldecode

URL-decode a string.

```jcl
urldecode("hello%20world")  # "hello world"
urldecode("a%2Bb%3Dc")  # "a+b=c"
```

---

## Hashing Functions

### md5

Compute MD5 hash of a string.

```jcl
md5("hello")  # "5d41402abc4b2a76b9719d911017c592"
```

### sha1

Compute SHA-1 hash of a string.

```jcl
sha1("hello")  # "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d"
```

### sha256

Compute SHA-256 hash of a string.

```jcl
sha256("hello")  # "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
```

### sha512

Compute SHA-512 hash of a string.

```jcl
sha512("hello")  # "9b71d224bd62f3785d96d46ad3ea3d73319bfbc2890caadae2dff72519673ca72323c3d99ba5c11d7c7acc6e14b8c5da0c4663475c2e5c3adef46f73bcdec043"
```

---

## Numeric Functions

### min

Find the minimum value in a list of numbers.

```jcl
min([3, 1, 2])  # 1
min([10, 5, 15])  # 5
```

### max

Find the maximum value in a list of numbers.

```jcl
max([3, 1, 2])  # 3
max([10, 5, 15])  # 15
```

### sum

Sum all numbers in a list.

```jcl
sum([1, 2, 3])  # 6
sum([10, 20, 30])  # 60
```

### avg

Calculate the average of numbers in a list.

```jcl
avg([1, 2, 3])  # 2
avg([10, 20, 30])  # 20
```

### abs

Get the absolute value of a number.

```jcl
abs(-5)  # 5
abs(3.14)  # 3.14
```

### ceil

Round a number up to the nearest integer.

```jcl
ceil(3.2)  # 4
ceil(-3.8)  # -3
```

### floor

Round a number down to the nearest integer.

```jcl
floor(3.8)  # 3
floor(-3.2)  # -4
```

### round

Round a number to the nearest integer.

```jcl
round(3.5)  # 4
round(3.4)  # 3
```

### product

Multiply all numbers in a list.

```jcl
product([2, 3, 4])  # 24
product([1, 2, 3])  # 6
```

---

## Type Conversion Functions

### tostring

Convert any value to a string.

```jcl
tostring(42)  # "42"
tostring(true)  # "true"
tostring([1, 2])  # "[1, 2]"
```

### tonumber

Convert a string to a number.

```jcl
tonumber("42")  # 42
tonumber("3.14")  # 3.14
```

### tobool

Convert a value to a boolean.

```jcl
tobool("true")  # true
tobool("false")  # false
tobool(1)  # true
tobool(0)  # false
```

### tolist

Convert a value to a list.

```jcl
tolist((a = 1, b = 2))  # [1, 2]  (values only)
tolist("abc")  # ["a", "b", "c"]
```

### tomap

Convert parallel lists of keys and values to a map.

```jcl
tomap(["a", "b"], [1, 2])  # (a = 1, b = 2)
```

---

## Type Introspection

### typeof

Return the type of a value as a string.

```jcl
typeof(42)  # "int"
typeof(3.14)  # "float"
typeof("hello")  # "string"
typeof(true)  # "bool"
typeof(null)  # "null"
typeof([1, 2, 3])  # "list"
typeof((a = 1, b = 2))  # "map"
typeof(x => x * 2)  # "function"
```

**Use Cases:**
```jcl
# Dynamic type checking
validate_config = config => {
  if typeof(config.port) != "int" then
    error("port must be an integer")
  else
    config
}

# Conditional logic based on type
process_value = value => {
  type = typeof(value)
  when type {
    "string" => upper(value)
    "int" => value * 2
    "list" => length(value)
    else => value
  }
}

# Debug logging
debug = value => {
  print("Value: ${value}, Type: ${typeof(value)}")
}
```

---

## Boolean Aggregation

### alltrue

Return true if all elements in a list are truthy.

```jcl
alltrue([true, true, true])  # true
alltrue([true, false, true])  # false
alltrue([1, 2, 3])  # true (non-zero numbers are truthy)
alltrue([1, 0, 2])  # false (0 is falsy)
alltrue(["a", "b", "c"])  # true (non-empty strings are truthy)
alltrue(["a", "", "c"])  # false (empty string is falsy)
```

**Truthy values:** Non-zero numbers, non-empty strings, non-empty lists, non-empty maps, true
**Falsy values:** 0, 0.0, empty string, empty list, false, null

### anytrue

Return true if any element in a list is truthy.

```jcl
anytrue([false, false, true, false])  # true
anytrue([false, false, false])  # false
anytrue([0, 0, 42])  # true (42 is truthy)
anytrue([0, 0, 0])  # false (all zeros)
anytrue(["", "", "hello"])  # true ("hello" is truthy)
anytrue(["", "", ""])  # false (all empty)
```

**Use Cases:**
```jcl
# Validation - ensure all checks pass
all_valid = alltrue([
  length(password) >= 8,
  contains(password, "@"),
  upper(password) != password
])

# Check if any feature flag is enabled
any_enabled = anytrue([
  config.feature_a_enabled,
  config.feature_b_enabled,
  config.feature_c_enabled
])

# Ensure all required fields are present
all_present = alltrue([
  config.host != null,
  config.port != null,
  config.username != null
])

# Check if any error occurred
has_errors = anytrue([
  length(syntax_errors) > 0,
  length(type_errors) > 0,
  length(runtime_errors) > 0
])
```

---

## Date/Time Functions

### timestamp

Get the current Unix timestamp.

```jcl
timestamp()  # 1699564800  (example)
```

### formatdate

Format a timestamp as a date string.

```jcl
formatdate(timestamp(), "%Y-%m-%d")  # "2024-11-18"
formatdate(timestamp(), "%H:%M:%S")  # "14:30:00"
```

**Common format codes:**
- `%Y` - Year (4 digits)
- `%m` - Month (01-12)
- `%d` - Day (01-31)
- `%H` - Hour (00-23)
- `%M` - Minute (00-59)
- `%S` - Second (00-59)

### timeadd

Add a duration to a timestamp.

```jcl
timeadd(timestamp(), "1h")  # Add 1 hour
timeadd(timestamp(), "30m")  # Add 30 minutes
timeadd(timestamp(), "1d")  # Add 1 day
```

---

## File Functions

### file

Read the contents of a file.

```jcl
file("config.txt")  # Returns file contents
```

### fileexists

Check if a file exists.

```jcl
fileexists("config.txt")  # true or false
```

### dirname

Get the directory part of a path.

```jcl
dirname("/path/to/file.txt")  # "/path/to"
dirname("file.txt")  # "."
```

### basename

Get the filename part of a path.

```jcl
basename("/path/to/file.txt")  # "file.txt"
basename("/path/to/dir/")  # "dir"
```

### abspath

Get the absolute path of a file.

```jcl
abspath("./file.txt")  # "/full/path/to/file.txt"
```

---

## Template Functions

### template

Render a template string with variables.

```jcl
template("Hello, {{name}}!", (name = "World"))  # "Hello, World!"
template("{{x}} + {{y}} = {{z}}", (x = 1, y = 2, z = 3))  # "1 + 2 = 3"
```

### templatefile

Read and render a template file.

```jcl
templatefile("template.txt", (name = "Alice", age = 30))
```

---

## Utility Functions

### range

Generate a range of numbers.

```jcl
range(5)  # [0, 1, 2, 3, 4]
range(1, 5)  # [1, 2, 3, 4]
range(0, 10, 2)  # [0, 2, 4, 6, 8]
```

### zipmap

Create a map from two lists (keys and values).

```jcl
zipmap(["a", "b", "c"], [1, 2, 3])  # (a = 1, b = 2, c = 3)
```

### coalesce

Return the first non-null value.

```jcl
coalesce(null, null, "default")  # "default"
coalesce("value", "default")  # "value"
```

### try

Try an expression and return a default value if it fails.

```jcl
try(1 / 0, 0)  # 0  (catches division by zero)
try(missing_var, "default")  # "default"
```

---

## Higher-Order & Streaming Functions

JCL provides powerful higher-order functions (`map`, `filter`, `reduce`) and streaming functions for memory-efficient lazy evaluation. These functions work polymorphically with both lists (eager evaluation) and streams (lazy evaluation).

### map

Apply a function to each element of a list or stream.

**Signature**: `map(function, list_or_stream) → list_or_stream`

```jcl
# With lists (eager evaluation)
doubled = map(x => x * 2, [1, 2, 3, 4, 5])
# [2, 4, 6, 8, 10]

uppercase = map(s => upper(s), ["hello", "world"])
# ["HELLO", "WORLD"]

# With streams (lazy evaluation)
numbers = stream([1, 2, 3, 4, 5])
doubled_stream = map(x => x * 2, numbers)
result = collect(doubled_stream)
# [2, 4, 6, 8, 10]

# Transparent optimization (automatic streaming)
# Only processes first 3 elements, not all 1000!
first_three = [x * 2 for x in [0..1000]][0:3]
# [0, 2, 4]
```

### filter

Filter elements of a list or stream based on a predicate function.

**Signature**: `filter(predicate, list_or_stream) → list_or_stream`

```jcl
# With lists (eager evaluation)
evens = filter(x => x % 2 == 0, [1, 2, 3, 4, 5, 6])
# [2, 4, 6]

long_names = filter(s => length(s) > 3, ["a", "hello", "world", "x"])
# ["hello", "world"]

# With streams (lazy evaluation)
numbers = stream([1, 2, 3, 4, 5, 6, 7, 8])
evens_stream = filter(x => x % 2 == 0, numbers)
result = collect(evens_stream)
# [2, 4, 6, 8]

# Transparent optimization with filter
# Processes only until we get 5 matching elements
first_five_evens = [x * 2 for x in [1..100] if x % 2 == 0][0:5]
# [4, 8, 12, 16, 20]
```

### reduce

Reduce a list to a single value by repeatedly applying a function.

**Signature**: `reduce(function, list, initial_value) → value`

```jcl
# Sum all numbers
sum = reduce((acc, x) => acc + x, [1, 2, 3, 4, 5], 0)
# 15

# Concatenate strings
joined = reduce((acc, s) => acc + s, ["hello", " ", "world"], "")
# "hello world"

# Find maximum
max_value = reduce((acc, x) => x > acc ? x : acc, [3, 1, 4, 1, 5, 9], 0)
# 9
```

### stream

Create a lazy stream from a list. Streams delay evaluation until values are actually needed.

**Signature**: `stream(list) → stream`

```jcl
# Create a stream (no evaluation yet)
numbers_stream = stream([1, 2, 3, 4, 5])

# Chain transformations (still lazy)
doubled = map(x => x * 2, numbers_stream)
evens = filter(x => x > 5, doubled)

# Materialize when needed
result = collect(evens)
# [6, 8, 10]
```

**When to use streams:**
- Processing large collections where you only need some results
- Chaining multiple transformations
- Memory-constrained environments

### take

Take the first N elements from a stream.

**Signature**: `take(stream, n) → stream`

```jcl
numbers = stream([1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
first_five = take(numbers, 5)
result = collect(first_five)
# [1, 2, 3, 4, 5]

# Useful for limiting results
# Only processes first 10 elements of a 1000-element range
small_sample = collect(take(stream([0..1000]), 10))
# [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
```

### collect

Materialize a stream into a list. This forces evaluation of all pending operations.

**Signature**: `collect(stream) → list`

```jcl
# Create and transform a stream
numbers = stream([1, 2, 3, 4, 5])
doubled = map(x => x * 2, numbers)

# Collect converts stream to list
result = collect(doubled)
# [2, 4, 6, 8, 10]
```

### Transparent Lazy Evaluation

**NEW in v1.1.0**: JCL automatically optimizes list comprehensions with slicing, eliminating the need for explicit `stream()` and `collect()` calls in common cases.

```jcl
# ✨ Automatic optimization - no explicit streaming needed!

# Pattern: [expr for x in list][start:end]
# Only evaluates elements 0-9, not all 1000
first_ten = [x * 2 for x in [0..1000]][0:10]
# [0, 2, 4, 6, 8, 10, 12, 14, 16, 18]

# With filter - stops once we have 5 results
first_five_evens = [x for x in [0..1000] if x % 2 == 0][0:5]
# [0, 2, 4, 6, 8]

# Middle slice - skips first 100, takes next 5
middle_slice = [x * 3 for x in [0..1000]][100:105]
# [300, 303, 306, 309, 312]

# Open-ended slice [start:]
from_500 = [x for x in [0..1000]][500:]
# [500, 501, ..., 1000]
```

**Performance benefits:**
- **Memory**: O(k) instead of O(n) where k = slice size, n = input size
- **Speed**: Only processes what's needed (10x-100x faster for small slices)
- **Transparent**: Works automatically, no code changes needed

**Optimization applies when:**
- ✅ Pattern is `[expr for x in list][start:end]` or `[expr for x in list if cond][start:end]`
- ✅ Single iterator (one `for` clause)
- ✅ Forward slice (no negative indices, no step)

**Falls back to standard evaluation when:**
- ❌ Negative indices (`[expr for x in list][-2:]`)
- ❌ Step parameter (`[expr for x in list][::2]`)
- ❌ Multiple iterators (`[x+y for x in list1 for y in list2]`)

### Performance Comparison

```jcl
# Without optimization (before v1.1.0):
# Evaluates 1,000,000 expressions, creates 1,000,000-element list
huge_list = [expensive_operation(x) for x in [0..1000000]]
result = huge_list[0:10]  # Then throws away 999,990 results

# With transparent optimization (v1.1.0+):
# Evaluates only 10 expressions, creates 10-element list
result = [expensive_operation(x) for x in [0..1000000]][0:10]

# Performance improvement:
# - 100,000x less memory
# - 100,000x faster
# - Completely transparent!
```

---

## Advanced Functions

### cartesian

Generate the Cartesian product of lists.

```jcl
cartesian([1, 2], ["a", "b"])  # [[1, "a"], [1, "b"], [2, "a"], [2, "b"]]
```

### combinations

Generate all combinations of a list (taken k at a time).

```jcl
combinations([1, 2, 3], 2)  # [[1, 2], [1, 3], [2, 3]]
```

### permutations

Generate all permutations of a list (taken k at a time).

```jcl
permutations([1, 2, 3], 2)  # [[1, 2], [1, 3], [2, 1], [2, 3], [3, 1], [3, 2]]
```

---

## Function Categories Summary

- **String (12)**: upper, lower, trim, trimprefix, trimsuffix, replace, split, join, format, substr, strlen
- **List (9)**: length, contains, reverse, sort, slice, distinct, flatten, compact
- **Map (4)**: keys, values, merge, lookup
- **Encoding (10)**: base64_encode, base64_decode, jsonencode, jsondecode, yamlencode, yamldecode, tomlencode, tomldecode, urlencode, urldecode
- **Hashing (4)**: md5, sha1, sha256, sha512
- **Numeric (9)**: min, max, sum, avg, abs, ceil, floor, round, product
- **Type Conversion (5)**: tostring, tonumber, tobool, tolist, tomap
- **Date/Time (3)**: timestamp, formatdate, timeadd
- **File (5)**: file, fileexists, dirname, basename, abspath
- **Template (2)**: template, templatefile
- **Utility (4)**: range, zipmap, coalesce, try
- **Higher-Order & Streaming (6)**: map, filter, reduce, stream, take, collect ⭐ NEW
- **Advanced (3)**: cartesian, combinations, permutations

**Total: 76 built-in functions**

---

## Error Handling

Most functions will return an error if given invalid arguments. Use the `try` function to catch errors:

```jcl
# This will fail
result = tonumber("not a number")  # Error

# This will return default value
result = try(tonumber("not a number"), 0)  # 0
```

## Type Checking

Before using functions, you can check types:

```jcl
is_string("hello")  # true
is_int(42)  # true
is_float(3.14)  # true
is_bool(true)  # true
is_list([1, 2])  # true
is_map((x = 1))  # true
```
