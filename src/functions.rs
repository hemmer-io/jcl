//! Built-in functions for JCL
//!
//! JCL provides a rich standard library of functions for data manipulation,
//! encoding/decoding, string operations, and more.

use crate::ast::Value;
use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde_json;
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha512};
use std::collections::HashMap;

/// Function signature
pub type BuiltinFunction = fn(&[Value]) -> Result<Value>;

/// Function registry
pub struct FunctionRegistry {
    functions: HashMap<String, BuiltinFunction>,
}

impl FunctionRegistry {
    /// Create a new function registry with all built-in functions
    pub fn new() -> Self {
        let mut registry = Self {
            functions: HashMap::new(),
        };

        // String functions
        registry.register("upper", fn_upper);
        registry.register("lower", fn_lower);
        registry.register("trim", fn_trim);
        registry.register("trimprefix", fn_trimprefix);
        registry.register("trimsuffix", fn_trimsuffix);
        registry.register("replace", fn_replace);
        registry.register("split", fn_split);
        registry.register("join", fn_join);
        registry.register("format", fn_format);
        registry.register("substr", fn_substr);
        registry.register("strlen", fn_strlen);

        // Encoding functions
        registry.register("base64encode", fn_base64encode);
        registry.register("base64decode", fn_base64decode);
        registry.register("jsonencode", fn_jsonencode);
        registry.register("json", fn_jsonencode); // Alias for jsonencode
        registry.register("jsondecode", fn_jsondecode);
        registry.register("yamlencode", fn_yamlencode);
        registry.register("yamldecode", fn_yamldecode);
        registry.register("tomlencode", fn_tomlencode);
        registry.register("tomldecode", fn_tomldecode);
        registry.register("urlencode", fn_urlencode);
        registry.register("urldecode", fn_urldecode);

        // Collection functions
        registry.register("length", fn_length);
        registry.register("len", fn_length); // Alias for length
        registry.register("contains", fn_contains);
        registry.register("keys", fn_keys);
        registry.register("values", fn_values);
        registry.register("merge", fn_merge);
        registry.register("lookup", fn_lookup);
        registry.register("reverse", fn_reverse);
        registry.register("sort", fn_sort);
        registry.register("slice", fn_slice);
        registry.register("distinct", fn_distinct);
        registry.register("flatten", fn_flatten);
        registry.register("compact", fn_compact);

        // Numeric functions
        registry.register("min", fn_min);
        registry.register("max", fn_max);
        registry.register("sum", fn_sum);
        registry.register("avg", fn_avg);
        registry.register("abs", fn_abs);
        registry.register("ceil", fn_ceil);
        registry.register("floor", fn_floor);
        registry.register("round", fn_round);

        // Type conversion
        registry.register("tostring", fn_tostring);
        registry.register("str", fn_tostring); // Alias for tostring
        registry.register("tonumber", fn_tonumber);
        registry.register("int", fn_tonumber); // Alias for tonumber
        registry.register("float", fn_tonumber); // Alias for tonumber
        registry.register("tobool", fn_tobool);
        registry.register("tolist", fn_tolist);
        registry.register("tomap", fn_tomap);

        // Hash functions
        registry.register("md5", fn_md5);
        registry.register("sha1", fn_sha1);
        registry.register("sha256", fn_sha256);
        registry.register("sha512", fn_sha512);
        registry.register("hash", fn_sha256); // Alias for sha256

        // Date/Time functions
        registry.register("timestamp", fn_timestamp);
        registry.register("formatdate", fn_formatdate);
        registry.register("timeadd", fn_timeadd);

        // Filesystem functions (not available in WASM)
        #[cfg(not(target_arch = "wasm32"))]
        {
            registry.register("file", fn_file);
            registry.register("fileexists", fn_fileexists);
            registry.register("dirname", fn_dirname);
            registry.register("basename", fn_basename);
            registry.register("abspath", fn_abspath);
        }

        // Template functions
        registry.register("template", fn_template);
        #[cfg(not(target_arch = "wasm32"))]
        registry.register("templatefile", fn_templatefile);

        // Utility functions
        registry.register("range", fn_range);
        registry.register("zipmap", fn_zipmap);
        registry.register("coalesce", fn_coalesce);
        registry.register("try", fn_try);

        // Matrix/Cartesian product functions
        registry.register("cartesian", fn_cartesian);
        registry.register("combinations", fn_combinations);
        registry.register("permutations", fn_permutations);
        registry.register("product", fn_product);

        registry
    }

    /// Register a function
    pub fn register(&mut self, name: &str, func: BuiltinFunction) {
        self.functions.insert(name.to_string(), func);
    }

    /// Call a function
    pub fn call(&self, name: &str, args: &[Value]) -> Result<Value> {
        match self.functions.get(name) {
            Some(func) => func(args),
            None => Err(anyhow!("Unknown function: {}", name)),
        }
    }

    /// Check if a function exists
    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    /// List all function names
    pub fn list_functions(&self) -> Vec<String> {
        self.functions.keys().cloned().collect()
    }
}

impl Default for FunctionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Global function registry using lazy_static
lazy_static::lazy_static! {
    static ref GLOBAL_REGISTRY: FunctionRegistry = FunctionRegistry::new();
}

/// Call a built-in function by name
pub fn call_builtin(name: &str, args: Vec<Value>) -> Result<Value> {
    GLOBAL_REGISTRY.call(name, &args)
}

// =============================================================================
// STRING FUNCTIONS
// =============================================================================

fn fn_upper(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "upper")?;
    let s = as_string(&args[0])?;
    Ok(Value::String(s.to_uppercase()))
}

fn fn_lower(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "lower")?;
    let s = as_string(&args[0])?;
    Ok(Value::String(s.to_lowercase()))
}

fn fn_trim(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "trim")?;
    let s = as_string(&args[0])?;
    Ok(Value::String(s.trim().to_string()))
}

fn fn_trimprefix(args: &[Value]) -> Result<Value> {
    require_args(args, 2, "trimprefix")?;
    let s = as_string(&args[0])?;
    let prefix = as_string(&args[1])?;
    Ok(Value::String(s.trim_start_matches(&prefix).to_string()))
}

fn fn_trimsuffix(args: &[Value]) -> Result<Value> {
    require_args(args, 2, "trimsuffix")?;
    let s = as_string(&args[0])?;
    let suffix = as_string(&args[1])?;
    Ok(Value::String(s.trim_end_matches(&suffix).to_string()))
}

fn fn_replace(args: &[Value]) -> Result<Value> {
    require_args(args, 3, "replace")?;
    let s = as_string(&args[0])?;
    let old = as_string(&args[1])?;
    let new = as_string(&args[2])?;
    Ok(Value::String(s.replace(&old, &new)))
}

fn fn_split(args: &[Value]) -> Result<Value> {
    require_args(args, 2, "split")?;
    let s = as_string(&args[0])?;
    let sep = as_string(&args[1])?;
    let parts: Vec<Value> = s
        .split(&sep)
        .map(|p| Value::String(p.to_string()))
        .collect();
    Ok(Value::List(parts))
}

fn fn_join(args: &[Value]) -> Result<Value> {
    require_args(args, 2, "join")?;
    let list = as_list(&args[0])?;
    let sep = as_string(&args[1])?;
    let strings: Result<Vec<String>> = list.iter().map(as_string).collect();
    Ok(Value::String(strings?.join(&sep)))
}

fn fn_format(args: &[Value]) -> Result<Value> {
    require_args_min(args, 1, "format")?;
    let format_str = as_string(&args[0])?;

    let mut result = String::new();
    let mut chars = format_str.chars().peekable();
    let mut arg_index = 1;

    while let Some(ch) = chars.next() {
        if ch == '%' {
            if let Some(&next_ch) = chars.peek() {
                chars.next(); // consume the format specifier

                match next_ch {
                    '%' => result.push('%'),
                    's' => {
                        // String format
                        if arg_index < args.len() {
                            result.push_str(&format_value_as_string(&args[arg_index])?);
                            arg_index += 1;
                        } else {
                            return Err(anyhow!("format: not enough arguments for format string"));
                        }
                    }
                    'd' | 'i' => {
                        // Integer format
                        if arg_index < args.len() {
                            let val = as_int(&args[arg_index])?;
                            result.push_str(&val.to_string());
                            arg_index += 1;
                        } else {
                            return Err(anyhow!("format: not enough arguments for format string"));
                        }
                    }
                    'f' => {
                        // Float format
                        if arg_index < args.len() {
                            let val = as_number(&args[arg_index])?;
                            result.push_str(&val.to_string());
                            arg_index += 1;
                        } else {
                            return Err(anyhow!("format: not enough arguments for format string"));
                        }
                    }
                    'b' => {
                        // Boolean format
                        if arg_index < args.len() {
                            match &args[arg_index] {
                                Value::Bool(b) => result.push_str(&b.to_string()),
                                _ => return Err(anyhow!("format: expected boolean for %b")),
                            }
                            arg_index += 1;
                        } else {
                            return Err(anyhow!("format: not enough arguments for format string"));
                        }
                    }
                    'v' => {
                        // Value format (any type)
                        if arg_index < args.len() {
                            result.push_str(&format!("{:?}", args[arg_index]));
                            arg_index += 1;
                        } else {
                            return Err(anyhow!("format: not enough arguments for format string"));
                        }
                    }
                    'x' => {
                        // Hexadecimal format
                        if arg_index < args.len() {
                            let val = as_int(&args[arg_index])?;
                            result.push_str(&format!("{:x}", val));
                            arg_index += 1;
                        } else {
                            return Err(anyhow!("format: not enough arguments for format string"));
                        }
                    }
                    'X' => {
                        // Hexadecimal format (uppercase)
                        if arg_index < args.len() {
                            let val = as_int(&args[arg_index])?;
                            result.push_str(&format!("{:X}", val));
                            arg_index += 1;
                        } else {
                            return Err(anyhow!("format: not enough arguments for format string"));
                        }
                    }
                    'o' => {
                        // Octal format
                        if arg_index < args.len() {
                            let val = as_int(&args[arg_index])?;
                            result.push_str(&format!("{:o}", val));
                            arg_index += 1;
                        } else {
                            return Err(anyhow!("format: not enough arguments for format string"));
                        }
                    }
                    _ => {
                        // Unknown format specifier, just include it literally
                        result.push('%');
                        result.push(next_ch);
                    }
                }
            } else {
                result.push('%');
            }
        } else {
            result.push(ch);
        }
    }

    Ok(Value::String(result))
}

/// Helper function to format a value as a string for %s
fn format_value_as_string(val: &Value) -> Result<String> {
    match val {
        Value::String(s) => Ok(s.clone()),
        Value::Int(i) => Ok(i.to_string()),
        Value::Float(f) => Ok(f.to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        Value::Null => Ok("null".to_string()),
        Value::List(items) => {
            let formatted: Vec<String> = items
                .iter()
                .map(format_value_as_string)
                .collect::<Result<Vec<_>>>()?;
            Ok(format!("[{}]", formatted.join(", ")))
        }
        Value::Map(map) => {
            let formatted: Vec<String> = map
                .iter()
                .map(|(k, v)| Ok(format!("{} = {}", k, format_value_as_string(v)?)))
                .collect::<Result<Vec<_>>>()?;
            Ok(format!("({})", formatted.join(", ")))
        }
        Value::Function { .. } => Ok("<function>".to_string()),
    }
}

fn fn_substr(args: &[Value]) -> Result<Value> {
    require_args(args, 3, "substr")?;
    let s = as_string(&args[0])?;
    let start = as_int(&args[1])? as usize;
    let length = as_int(&args[2])? as usize;

    let chars: Vec<char> = s.chars().collect();
    let end = (start + length).min(chars.len());
    let result: String = chars[start..end].iter().collect();

    Ok(Value::String(result))
}

fn fn_strlen(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "strlen")?;
    let s = as_string(&args[0])?;
    Ok(Value::Int(s.len() as i64))
}

// =============================================================================
// ENCODING FUNCTIONS
// =============================================================================

fn fn_base64encode(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "base64encode")?;
    let s = as_string(&args[0])?;
    Ok(Value::String(STANDARD.encode(s)))
}

fn fn_base64decode(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "base64decode")?;
    let s = as_string(&args[0])?;
    let decoded = STANDARD.decode(s)?;
    Ok(Value::String(String::from_utf8(decoded)?))
}

fn fn_jsonencode(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "jsonencode")?;
    let json = serde_json::to_string(&args[0])?;
    Ok(Value::String(json))
}

fn fn_jsondecode(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "jsondecode")?;
    let s = as_string(&args[0])?;
    let value: Value = serde_json::from_str(&s)?;
    Ok(value)
}

fn fn_yamlencode(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "yamlencode")?;
    let yaml = serde_yaml::to_string(&args[0])?;
    Ok(Value::String(yaml))
}

fn fn_yamldecode(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "yamldecode")?;
    let s = as_string(&args[0])?;
    let value: Value = serde_yaml::from_str(&s)?;
    Ok(value)
}

fn fn_tomlencode(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "tomlencode")?;
    let toml = toml::to_string(&args[0])?;
    Ok(Value::String(toml))
}

fn fn_tomldecode(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "tomldecode")?;
    let s = as_string(&args[0])?;
    let value: Value = toml::from_str(&s)?;
    Ok(value)
}

fn fn_urlencode(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "urlencode")?;
    let s = as_string(&args[0])?;
    Ok(Value::String(urlencoding::encode(&s).to_string()))
}

fn fn_urldecode(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "urldecode")?;
    let s = as_string(&args[0])?;
    Ok(Value::String(urlencoding::decode(&s)?.to_string()))
}

// =============================================================================
// COLLECTION FUNCTIONS
// =============================================================================

fn fn_length(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "length")?;
    let len = match &args[0] {
        Value::String(s) => s.len(),
        Value::List(l) => l.len(),
        Value::Map(m) => m.len(),
        _ => return Err(anyhow!("length() requires string, list, or map")),
    };
    Ok(Value::Int(len as i64))
}

fn fn_contains(args: &[Value]) -> Result<Value> {
    require_args(args, 2, "contains")?;
    let result = match &args[0] {
        Value::List(list) => list.contains(&args[1]),
        Value::String(s) => {
            let needle = as_string(&args[1])?;
            s.contains(&needle)
        }
        _ => return Err(anyhow!("contains() requires list or string")),
    };
    Ok(Value::Bool(result))
}

fn fn_keys(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "keys")?;
    let map = as_map(&args[0])?;
    let keys: Vec<Value> = map.keys().map(|k| Value::String(k.clone())).collect();
    Ok(Value::List(keys))
}

fn fn_values(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "values")?;
    let map = as_map(&args[0])?;
    let values: Vec<Value> = map.values().cloned().collect();
    Ok(Value::List(values))
}

fn fn_merge(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::Map(HashMap::new()));
    }

    let mut result = HashMap::new();
    for arg in args {
        let map = as_map(arg)?;
        result.extend(map.clone());
    }
    Ok(Value::Map(result))
}

fn fn_lookup(args: &[Value]) -> Result<Value> {
    require_args_min(args, 2, "lookup")?;
    let map = as_map(&args[0])?;
    let key = as_string(&args[1])?;

    match map.get(&key) {
        Some(value) => Ok(value.clone()),
        None => {
            if args.len() >= 3 {
                Ok(args[2].clone())
            } else {
                Err(anyhow!("Key not found: {}", key))
            }
        }
    }
}

fn fn_reverse(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "reverse")?;
    let mut list = as_list(&args[0])?.clone();
    list.reverse();
    Ok(Value::List(list))
}

fn fn_sort(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "sort")?;
    let mut list = as_list(&args[0])?.clone();
    list.sort_by(|a, b| match (a, b) {
        (Value::String(s1), Value::String(s2)) => s1.cmp(s2),
        (Value::Int(i1), Value::Int(i2)) => i1.cmp(i2),
        (Value::Float(f1), Value::Float(f2)) => {
            f1.partial_cmp(f2).unwrap_or(std::cmp::Ordering::Equal)
        }
        _ => std::cmp::Ordering::Equal,
    });
    Ok(Value::List(list))
}

fn fn_slice(args: &[Value]) -> Result<Value> {
    require_args(args, 3, "slice")?;
    let list = as_list(&args[0])?;
    let start = as_int(&args[1])? as usize;
    let end = as_int(&args[2])? as usize;

    let end = end.min(list.len());
    let start = start.min(end);

    Ok(Value::List(list[start..end].to_vec()))
}

fn fn_distinct(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "distinct")?;
    let list = as_list(&args[0])?;
    let mut seen = Vec::new();
    for item in list {
        if !seen.contains(item) {
            seen.push(item.clone());
        }
    }
    Ok(Value::List(seen))
}

fn fn_flatten(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "flatten")?;
    let list = as_list(&args[0])?;
    let mut result = Vec::new();
    for item in list {
        if let Value::List(nested) = item {
            result.extend(nested.clone());
        } else {
            result.push(item.clone());
        }
    }
    Ok(Value::List(result))
}

fn fn_compact(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "compact")?;
    let list = as_list(&args[0])?;
    let result: Vec<Value> = list
        .iter()
        .filter(|v| !matches!(v, Value::Null))
        .cloned()
        .collect();
    Ok(Value::List(result))
}

// =============================================================================
// NUMERIC FUNCTIONS
// =============================================================================

fn fn_min(args: &[Value]) -> Result<Value> {
    require_args_min(args, 1, "min")?;
    let list = if args.len() == 1 && matches!(&args[0], Value::List(_)) {
        as_list(&args[0])?
    } else {
        args
    };

    let mut min: Option<f64> = None;
    for val in list {
        let num = as_number(val)?;
        min = Some(min.map_or(num, |m| m.min(num)));
    }

    Ok(Value::Float(min.unwrap()))
}

fn fn_max(args: &[Value]) -> Result<Value> {
    require_args_min(args, 1, "max")?;
    let list = if args.len() == 1 && matches!(&args[0], Value::List(_)) {
        as_list(&args[0])?
    } else {
        args
    };

    let mut max: Option<f64> = None;
    for val in list {
        let num = as_number(val)?;
        max = Some(max.map_or(num, |m| m.max(num)));
    }

    Ok(Value::Float(max.unwrap()))
}

fn fn_sum(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "sum")?;
    let list = as_list(&args[0])?;
    let mut sum = 0.0;
    for val in list {
        sum += as_number(val)?;
    }
    Ok(Value::Float(sum))
}

fn fn_avg(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "avg")?;
    let list = as_list(&args[0])?;
    if list.is_empty() {
        return Ok(Value::Float(0.0));
    }
    let sum: f64 = list.iter().map(|v| as_number(v).unwrap_or(0.0)).sum();
    Ok(Value::Float(sum / list.len() as f64))
}

fn fn_abs(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "abs")?;
    let num = as_number(&args[0])?;
    Ok(Value::Float(num.abs()))
}

fn fn_ceil(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "ceil")?;
    let num = as_number(&args[0])?;
    Ok(Value::Int(num.ceil() as i64))
}

fn fn_floor(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "floor")?;
    let num = as_number(&args[0])?;
    Ok(Value::Int(num.floor() as i64))
}

fn fn_round(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "round")?;
    let num = as_number(&args[0])?;
    Ok(Value::Int(num.round() as i64))
}

// =============================================================================
// TYPE CONVERSION FUNCTIONS
// =============================================================================

fn fn_tostring(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "tostring")?;
    Ok(Value::String(format!("{:?}", args[0])))
}

fn fn_tonumber(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "tonumber")?;
    let s = as_string(&args[0])?;
    if let Ok(i) = s.parse::<i64>() {
        Ok(Value::Int(i))
    } else if let Ok(f) = s.parse::<f64>() {
        Ok(Value::Float(f))
    } else {
        Err(anyhow!("Cannot convert to number: {}", s))
    }
}

fn fn_tobool(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "tobool")?;
    let result = match &args[0] {
        Value::Bool(b) => *b,
        Value::String(s) => matches!(s.to_lowercase().as_str(), "true" | "yes" | "1"),
        Value::Int(i) => *i != 0,
        _ => false,
    };
    Ok(Value::Bool(result))
}

fn fn_tolist(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "tolist")?;

    match &args[0] {
        Value::List(l) => Ok(Value::List(l.clone())),
        Value::Map(m) => {
            // Convert map to list of [key, value] pairs
            let pairs: Vec<Value> = m
                .iter()
                .map(|(k, v)| Value::List(vec![Value::String(k.clone()), v.clone()]))
                .collect();
            Ok(Value::List(pairs))
        }
        Value::String(s) => {
            // Convert string to list of characters
            let chars: Vec<Value> = s.chars().map(|c| Value::String(c.to_string())).collect();
            Ok(Value::List(chars))
        }
        other => Ok(Value::List(vec![other.clone()])),
    }
}

fn fn_tomap(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "tomap")?;

    match &args[0] {
        Value::Map(m) => Ok(Value::Map(m.clone())),
        Value::List(items) => {
            // Convert list of [key, value] pairs to map
            let mut map = HashMap::new();
            for item in items {
                if let Value::List(pair) = item {
                    if pair.len() == 2 {
                        if let Value::String(key) = &pair[0] {
                            map.insert(key.clone(), pair[1].clone());
                        }
                    }
                }
            }
            Ok(Value::Map(map))
        }
        _ => Err(anyhow!("Cannot convert value to map")),
    }
}

// =============================================================================
// HASH FUNCTIONS
// =============================================================================

fn fn_md5(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "md5")?;
    let s = as_string(&args[0])?;
    Ok(Value::String(format!("{:x}", md5::compute(s))))
}

fn fn_sha1(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "sha1")?;
    let s = as_string(&args[0])?;
    let mut hasher = Sha1::new();
    hasher.update(s.as_bytes());
    let result = hasher.finalize();
    Ok(Value::String(format!("{:x}", result)))
}

fn fn_sha256(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "sha256")?;
    let s = as_string(&args[0])?;
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    let result = hasher.finalize();
    Ok(Value::String(format!("{:x}", result)))
}

fn fn_sha512(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "sha512")?;
    let s = as_string(&args[0])?;
    let mut hasher = Sha512::new();
    hasher.update(s.as_bytes());
    let result = hasher.finalize();
    Ok(Value::String(format!("{:x}", result)))
}

// =============================================================================
// DATE/TIME FUNCTIONS
// =============================================================================

fn fn_timestamp(_args: &[Value]) -> Result<Value> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
    Ok(Value::Int(now.as_secs() as i64))
}

fn fn_formatdate(args: &[Value]) -> Result<Value> {
    require_args_min(args, 2, "formatdate")?;
    let format_str = as_string(&args[0])?;
    let timestamp = as_int(&args[1])?;

    let datetime = chrono::DateTime::from_timestamp(timestamp, 0)
        .ok_or_else(|| anyhow!("Invalid timestamp: {}", timestamp))?;

    Ok(Value::String(datetime.format(&format_str).to_string()))
}

fn fn_timeadd(args: &[Value]) -> Result<Value> {
    require_args(args, 2, "timeadd")?;
    let timestamp = as_int(&args[0])?;
    let seconds_to_add = as_int(&args[1])?;

    let new_timestamp = timestamp + seconds_to_add;
    Ok(Value::Int(new_timestamp))
}

// =============================================================================
// FILESYSTEM FUNCTIONS
// =============================================================================

#[cfg(not(target_arch = "wasm32"))]
fn fn_file(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "file")?;
    let path = as_string(&args[0])?;
    let content = std::fs::read_to_string(path)?;
    Ok(Value::String(content))
}

#[cfg(not(target_arch = "wasm32"))]
fn fn_fileexists(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "fileexists")?;
    let path = as_string(&args[0])?;
    Ok(Value::Bool(std::path::Path::new(&path).exists()))
}

#[cfg(not(target_arch = "wasm32"))]
fn fn_dirname(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "dirname")?;
    let path = as_string(&args[0])?;
    let parent = std::path::Path::new(&path)
        .parent()
        .and_then(|p| p.to_str())
        .unwrap_or("");
    Ok(Value::String(parent.to_string()))
}

#[cfg(not(target_arch = "wasm32"))]
fn fn_basename(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "basename")?;
    let path = as_string(&args[0])?;
    let name = std::path::Path::new(&path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    Ok(Value::String(name.to_string()))
}

#[cfg(not(target_arch = "wasm32"))]
fn fn_abspath(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "abspath")?;
    let path = as_string(&args[0])?;
    let abs = std::fs::canonicalize(path)?;
    Ok(Value::String(abs.to_string_lossy().to_string()))
}

// =============================================================================
// TEMPLATE FUNCTIONS
// =============================================================================

fn fn_template(args: &[Value]) -> Result<Value> {
    require_args(args, 2, "template")?;
    let template_str = as_string(&args[0])?;
    let vars = as_map(&args[1])?;

    // Convert JCL map to JSON value for Handlebars
    let json_vars = value_to_serde_json(&Value::Map(vars.clone()))?;

    // Render template with Handlebars
    let mut handlebars = handlebars::Handlebars::new();
    handlebars.set_strict_mode(true);

    let rendered = handlebars
        .render_template(&template_str, &json_vars)
        .map_err(|e| anyhow!("Template rendering error: {}", e))?;

    Ok(Value::String(rendered))
}

#[cfg(not(target_arch = "wasm32"))]
fn fn_templatefile(args: &[Value]) -> Result<Value> {
    require_args(args, 2, "templatefile")?;
    let path = as_string(&args[0])?;
    let vars = as_map(&args[1])?;

    // Read template from file
    let template_str = std::fs::read_to_string(&path)
        .map_err(|e| anyhow!("Failed to read template file '{}': {}", path, e))?;

    // Convert JCL map to JSON value for Handlebars
    let json_vars = value_to_serde_json(&Value::Map(vars.clone()))?;

    // Render template with Handlebars
    let mut handlebars = handlebars::Handlebars::new();
    handlebars.set_strict_mode(true);

    let rendered = handlebars
        .render_template(&template_str, &json_vars)
        .map_err(|e| anyhow!("Template rendering error: {}", e))?;

    Ok(Value::String(rendered))
}

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

fn fn_range(args: &[Value]) -> Result<Value> {
    let (start, end) = match args.len() {
        1 => (0, as_int(&args[0])?),
        2 => (as_int(&args[0])?, as_int(&args[1])?),
        _ => return Err(anyhow!("range() requires 1 or 2 arguments")),
    };

    let result: Vec<Value> = (start..end).map(Value::Int).collect();
    Ok(Value::List(result))
}

fn fn_zipmap(args: &[Value]) -> Result<Value> {
    require_args(args, 2, "zipmap")?;
    let keys = as_list(&args[0])?;
    let values = as_list(&args[1])?;

    let mut result = HashMap::new();
    for (k, v) in keys.iter().zip(values.iter()) {
        let key = as_string(k)?;
        result.insert(key, v.clone());
    }

    Ok(Value::Map(result))
}

fn fn_coalesce(args: &[Value]) -> Result<Value> {
    for arg in args {
        if !matches!(arg, Value::Null) {
            return Ok(arg.clone());
        }
    }
    Ok(Value::Null)
}

fn fn_try(args: &[Value]) -> Result<Value> {
    require_args_min(args, 1, "try")?;
    // Note: try() error handling is implemented in the evaluator (Expression::Try)
    // This function is not directly called - the parser generates Expression::Try AST nodes
    // which are handled by evaluator.rs lines 285-296
    Ok(args[0].clone())
}

// =============================================================================
// MATRIX / CARTESIAN PRODUCT FUNCTIONS
// =============================================================================

fn fn_cartesian(args: &[Value]) -> Result<Value> {
    require_args_min(args, 2, "cartesian")?;

    // Get all lists
    let mut lists: Vec<Vec<Value>> = Vec::new();
    for arg in args {
        lists.push(as_list(arg)?.clone());
    }

    // Compute Cartesian product
    let mut result: Vec<Value> = vec![Value::List(vec![])];

    for list in lists {
        let mut new_result = Vec::new();
        for existing in &result {
            let existing_list = match existing {
                Value::List(l) => l,
                _ => continue,
            };
            for item in &list {
                let mut new_combo = existing_list.clone();
                new_combo.push(item.clone());
                new_result.push(Value::List(new_combo));
            }
        }
        result = new_result;
    }

    Ok(Value::List(result))
}

fn fn_product(args: &[Value]) -> Result<Value> {
    // Alias for cartesian
    fn_cartesian(args)
}

fn fn_combinations(args: &[Value]) -> Result<Value> {
    require_args(args, 2, "combinations")?;
    let list = as_list(&args[0])?;
    let n = as_int(&args[1])? as usize;

    let result = generate_combinations(list, n);
    Ok(Value::List(result))
}

fn generate_combinations(list: &[Value], n: usize) -> Vec<Value> {
    if n == 0 {
        return vec![Value::List(vec![])];
    }
    if list.is_empty() {
        return vec![];
    }

    let mut result = Vec::new();

    // Include first element
    for rest in generate_combinations(&list[1..], n - 1) {
        if let Value::List(mut rest_list) = rest {
            let mut combo = vec![list[0].clone()];
            combo.append(&mut rest_list);
            result.push(Value::List(combo));
        }
    }

    // Exclude first element
    result.extend(generate_combinations(&list[1..], n));

    result
}

fn fn_permutations(args: &[Value]) -> Result<Value> {
    require_args(args, 2, "permutations")?;
    let list = as_list(&args[0])?;
    let n = as_int(&args[1])? as usize;

    let result = generate_permutations(list, n);
    Ok(Value::List(result))
}

fn generate_permutations(list: &[Value], n: usize) -> Vec<Value> {
    if n == 0 {
        return vec![Value::List(vec![])];
    }
    if list.is_empty() {
        return vec![];
    }

    let mut result = Vec::new();

    for (i, item) in list.iter().enumerate() {
        let mut remaining = list.to_vec();
        remaining.remove(i);

        for rest in generate_permutations(&remaining, n - 1) {
            if let Value::List(mut rest_list) = rest {
                let mut perm = vec![item.clone()];
                perm.append(&mut rest_list);
                result.push(Value::List(perm));
            }
        }
    }

    result
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

fn require_args(args: &[Value], expected: usize, name: &str) -> Result<()> {
    if args.len() != expected {
        return Err(anyhow!(
            "{}() requires {} argument(s), got {}",
            name,
            expected,
            args.len()
        ));
    }
    Ok(())
}

fn require_args_min(args: &[Value], min: usize, name: &str) -> Result<()> {
    if args.len() < min {
        return Err(anyhow!(
            "{}() requires at least {} argument(s), got {}",
            name,
            min,
            args.len()
        ));
    }
    Ok(())
}

fn as_string(value: &Value) -> Result<String> {
    match value {
        Value::String(s) => Ok(s.clone()),
        _ => Err(anyhow!("Expected string, got {:?}", value)),
    }
}

fn as_int(value: &Value) -> Result<i64> {
    match value {
        Value::Int(i) => Ok(*i),
        _ => Err(anyhow!("Expected int, got {:?}", value)),
    }
}

fn as_number(value: &Value) -> Result<f64> {
    match value {
        Value::Int(i) => Ok(*i as f64),
        Value::Float(f) => Ok(*f),
        _ => Err(anyhow!("Expected number, got {:?}", value)),
    }
}

fn as_list(value: &Value) -> Result<&Vec<Value>> {
    match value {
        Value::List(l) => Ok(l),
        _ => Err(anyhow!("Expected list, got {:?}", value)),
    }
}

fn as_map(value: &Value) -> Result<&HashMap<String, Value>> {
    match value {
        Value::Map(m) => Ok(m),
        _ => Err(anyhow!("Expected map, got {:?}", value)),
    }
}

/// Convert JCL Value to serde_json::Value for template rendering
fn value_to_serde_json(value: &Value) -> Result<serde_json::Value> {
    match value {
        Value::String(s) => Ok(serde_json::Value::String(s.clone())),
        Value::Int(i) => Ok(serde_json::Value::Number((*i).into())),
        Value::Float(f) => serde_json::Number::from_f64(*f)
            .map(serde_json::Value::Number)
            .ok_or_else(|| anyhow!("Invalid float value for JSON")),
        Value::Bool(b) => Ok(serde_json::Value::Bool(*b)),
        Value::Null => Ok(serde_json::Value::Null),
        Value::List(items) => {
            let json_items: Result<Vec<serde_json::Value>> =
                items.iter().map(value_to_serde_json).collect();
            Ok(serde_json::Value::Array(json_items?))
        }
        Value::Map(map) => {
            let mut json_map = serde_json::Map::new();
            for (key, val) in map {
                json_map.insert(key.clone(), value_to_serde_json(val)?);
            }
            Ok(serde_json::Value::Object(json_map))
        }
        Value::Function { .. } => Err(anyhow!(
            "Cannot convert function to JSON for template rendering"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upper() {
        let result = fn_upper(&[Value::String("hello".to_string())]).unwrap();
        assert_eq!(result, Value::String("HELLO".to_string()));
    }

    #[test]
    fn test_length() {
        let result = fn_length(&[Value::String("hello".to_string())]).unwrap();
        assert_eq!(result, Value::Int(5));

        let result = fn_length(&[Value::List(vec![Value::Int(1), Value::Int(2)])]).unwrap();
        assert_eq!(result, Value::Int(2));
    }

    #[test]
    fn test_merge() {
        let mut map1 = HashMap::new();
        map1.insert("a".to_string(), Value::Int(1));

        let mut map2 = HashMap::new();
        map2.insert("b".to_string(), Value::Int(2));

        let result = fn_merge(&[Value::Map(map1), Value::Map(map2)]).unwrap();

        if let Value::Map(m) = result {
            assert_eq!(m.len(), 2);
            assert_eq!(m.get("a"), Some(&Value::Int(1)));
            assert_eq!(m.get("b"), Some(&Value::Int(2)));
        } else {
            panic!("Expected map");
        }
    }

    #[test]
    fn test_format_basic() {
        // String formatting
        let result = fn_format(&[
            Value::String("Hello, %s!".to_string()),
            Value::String("World".to_string()),
        ])
        .unwrap();
        assert_eq!(result, Value::String("Hello, World!".to_string()));
    }

    #[test]
    fn test_format_integer() {
        // Integer formatting
        let result = fn_format(&[Value::String("Count: %d".to_string()), Value::Int(42)]).unwrap();
        assert_eq!(result, Value::String("Count: 42".to_string()));
    }

    #[test]
    fn test_format_float() {
        // Float formatting
        let result =
            fn_format(&[Value::String("Value: %f".to_string()), Value::Float(42.5)]).unwrap();
        assert_eq!(result, Value::String("Value: 42.5".to_string()));
    }

    #[test]
    fn test_format_hex() {
        // Hexadecimal formatting
        let result = fn_format(&[Value::String("Hex: %x".to_string()), Value::Int(255)]).unwrap();
        assert_eq!(result, Value::String("Hex: ff".to_string()));
    }

    #[test]
    fn test_format_percent_escape() {
        // Percent sign escaping - %% in format string should produce single %
        let result = fn_format(&[Value::String("100%% complete".to_string())]).unwrap();
        assert_eq!(result, Value::String("100% complete".to_string()));
    }

    #[test]
    fn test_format_multiple() {
        // Multiple arguments
        let result = fn_format(&[
            Value::String("%s is %d years old".to_string()),
            Value::String("Alice".to_string()),
            Value::Int(30),
        ])
        .unwrap();
        assert_eq!(result, Value::String("Alice is 30 years old".to_string()));
    }

    #[test]
    fn test_format_boolean() {
        // Boolean formatting
        let result =
            fn_format(&[Value::String("Active: %b".to_string()), Value::Bool(true)]).unwrap();
        assert_eq!(result, Value::String("Active: true".to_string()));
    }

    #[test]
    fn test_range() {
        let result = fn_range(&[Value::Int(5)]).unwrap();
        if let Value::List(l) = result {
            assert_eq!(l.len(), 5);
            assert_eq!(l[0], Value::Int(0));
            assert_eq!(l[4], Value::Int(4));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_template() {
        let template_str = "Hello, {{name}}! Age: {{age}}.";
        let vars = vec![
            ("name".to_string(), Value::String("Alice".to_string())),
            ("age".to_string(), Value::Int(30)),
        ]
        .into_iter()
        .collect();

        let result =
            fn_template(&[Value::String(template_str.to_string()), Value::Map(vars)]).unwrap();

        assert_eq!(result, Value::String("Hello, Alice! Age: 30.".to_string()));
    }

    #[test]
    fn test_template_with_list() {
        let template_str = "Items: {{#each items}}{{this}}{{#unless @last}}, {{/unless}}{{/each}}";
        let vars = vec![(
            "items".to_string(),
            Value::List(vec![
                Value::String("apple".to_string()),
                Value::String("banana".to_string()),
                Value::String("cherry".to_string()),
            ]),
        )]
        .into_iter()
        .collect();

        let result =
            fn_template(&[Value::String(template_str.to_string()), Value::Map(vars)]).unwrap();

        assert_eq!(
            result,
            Value::String("Items: apple, banana, cherry".to_string())
        );
    }

    #[test]
    fn test_template_with_conditional() {
        let template_str = "Status: {{#if active}}Active{{else}}Inactive{{/if}}";
        let vars = vec![("active".to_string(), Value::Bool(true))]
            .into_iter()
            .collect();

        let result =
            fn_template(&[Value::String(template_str.to_string()), Value::Map(vars)]).unwrap();

        assert_eq!(result, Value::String("Status: Active".to_string()));
    }

    #[test]
    fn test_templatefile() {
        // Create temporary template file
        let temp_file = "/tmp/jcl_test_template_unique.txt";
        std::fs::write(temp_file, "Hello, {{name}}!").unwrap();

        let vars = vec![("name".to_string(), Value::String("World".to_string()))]
            .into_iter()
            .collect();

        let result =
            fn_templatefile(&[Value::String(temp_file.to_string()), Value::Map(vars)]).unwrap();

        assert_eq!(result, Value::String("Hello, World!".to_string()));

        // Cleanup
        std::fs::remove_file(temp_file).ok();
    }
}
