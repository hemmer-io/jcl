# Built-in Functions Examples
# Demonstrates: string, collection, numeric, and utility functions

# String functions
text = "JCL Configuration Language"
text_upper = upper(text)
text_lower = lower(text)
text_len = len(text)
text_trimmed = trim("  spaces  ")
text_replaced = replace(text, "Configuration", "Config")
text_split = split(text, " ")
words_joined = join(["Hello", "World"], " ")

# Collection functions
numbers = [5, 2, 8, 1, 9, 3]
sorted_nums = sort(numbers)
reversed_nums = reverse(numbers)
num_count = len(numbers)
has_five = contains(numbers, 5)
first_three = slice(numbers, 0, 3)

# Numeric functions
value = -42.7
absolute = abs(value)
rounded = round(42.6)
floored = floor(42.9)
ceiled = ceil(42.1)
maximum = max([1, 5, 3, 9, 2])
minimum = min([1, 5, 3, 9, 2])
total = sum([1, 2, 3, 4, 5])

# Type conversion
str_num = str(42)
int_val = int("123")
float_val = float("3.14")

# Encoding functions
encoded = base64encode("Hello, JCL!")
url_safe = urlencode("hello world?")
json_str = json((name = "JCL", version = 1.0))

# Hashing functions
text_hash = hash("JCL")
md5_hash = md5("password")
sha256_hash = sha256("secret")

# Utility functions
obj_keys = keys((a = 1, b = 2, c = 3))
obj_values = values((a = 1, b = 2, c = 3))
merged = merge((a = 1, b = 2), (b = 3, c = 4))
