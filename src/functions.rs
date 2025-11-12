//! Built-in functions for JCL
//!
//! JCL provides a rich standard library of functions for data manipulation,
//! encoding/decoding, string operations, and more.

use crate::ast::Value;
use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde_json;
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
        registry.register("jsondecode", fn_jsondecode);
        registry.register("yamlencode", fn_yamlencode);
        registry.register("yamldecode", fn_yamldecode);
        registry.register("tomlencode", fn_tomlencode);
        registry.register("tomldecode", fn_tomldecode);
        registry.register("urlencode", fn_urlencode);
        registry.register("urldecode", fn_urldecode);

        // Collection functions
        registry.register("length", fn_length);
        registry.register("contains", fn_contains);
        registry.register("keys", fn_keys);
        registry.register("values", fn_values);
        registry.register("merge", fn_merge);
        registry.register("lookup", fn_lookup);
        registry.register("reverse", fn_reverse);
        registry.register("sort", fn_sort);
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
        registry.register("tonumber", fn_tonumber);
        registry.register("tobool", fn_tobool);
        registry.register("tolist", fn_tolist);
        registry.register("tomap", fn_tomap);

        // Hash functions
        registry.register("md5", fn_md5);
        registry.register("sha1", fn_sha1);
        registry.register("sha256", fn_sha256);
        registry.register("sha512", fn_sha512);

        // Date/Time functions
        registry.register("timestamp", fn_timestamp);
        registry.register("formatdate", fn_formatdate);
        registry.register("timeadd", fn_timeadd);

        // Filesystem functions
        registry.register("file", fn_file);
        registry.register("fileexists", fn_fileexists);
        registry.register("dirname", fn_dirname);
        registry.register("basename", fn_basename);
        registry.register("abspath", fn_abspath);

        // Template functions
        registry.register("template", fn_template);
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

fn fn_format(_args: &[Value]) -> Result<Value> {
    // TODO: Implement printf-style formatting
    Ok(Value::String("TODO: format".to_string()))
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
    list.sort_by(|a, b| {
        match (a, b) {
            (Value::String(s1), Value::String(s2)) => s1.cmp(s2),
            (Value::Int(i1), Value::Int(i2)) => i1.cmp(i2),
            (Value::Float(f1), Value::Float(f2)) => {
                f1.partial_cmp(f2).unwrap_or(std::cmp::Ordering::Equal)
            }
            _ => std::cmp::Ordering::Equal,
        }
    });
    Ok(Value::List(list))
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

fn fn_tolist(_args: &[Value]) -> Result<Value> {
    // TODO: Implement
    Ok(Value::List(vec![]))
}

fn fn_tomap(_args: &[Value]) -> Result<Value> {
    // TODO: Implement
    Ok(Value::Map(HashMap::new()))
}

// =============================================================================
// HASH FUNCTIONS
// =============================================================================

fn fn_md5(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "md5")?;
    let s = as_string(&args[0])?;
    Ok(Value::String(format!("{:x}", md5::compute(s))))
}

fn fn_sha1(_args: &[Value]) -> Result<Value> {
    // TODO: Implement with sha1 crate
    Ok(Value::String("TODO: sha1".to_string()))
}

fn fn_sha256(_args: &[Value]) -> Result<Value> {
    // TODO: Implement with sha2 crate
    Ok(Value::String("TODO: sha256".to_string()))
}

fn fn_sha512(_args: &[Value]) -> Result<Value> {
    // TODO: Implement with sha2 crate
    Ok(Value::String("TODO: sha512".to_string()))
}

// =============================================================================
// DATE/TIME FUNCTIONS
// =============================================================================

fn fn_timestamp(_args: &[Value]) -> Result<Value> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
    Ok(Value::Int(now.as_secs() as i64))
}

fn fn_formatdate(_args: &[Value]) -> Result<Value> {
    // TODO: Implement with chrono
    Ok(Value::String("TODO: formatdate".to_string()))
}

fn fn_timeadd(_args: &[Value]) -> Result<Value> {
    // TODO: Implement
    Ok(Value::String("TODO: timeadd".to_string()))
}

// =============================================================================
// FILESYSTEM FUNCTIONS
// =============================================================================

fn fn_file(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "file")?;
    let path = as_string(&args[0])?;
    let content = std::fs::read_to_string(path)?;
    Ok(Value::String(content))
}

fn fn_fileexists(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "fileexists")?;
    let path = as_string(&args[0])?;
    Ok(Value::Bool(std::path::Path::new(&path).exists()))
}

fn fn_dirname(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "dirname")?;
    let path = as_string(&args[0])?;
    let parent = std::path::Path::new(&path)
        .parent()
        .and_then(|p| p.to_str())
        .unwrap_or("");
    Ok(Value::String(parent.to_string()))
}

fn fn_basename(args: &[Value]) -> Result<Value> {
    require_args(args, 1, "basename")?;
    let path = as_string(&args[0])?;
    let name = std::path::Path::new(&path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    Ok(Value::String(name.to_string()))
}

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
    require_args_min(args, 1, "template")?;
    let template = as_string(&args[0])?;

    // TODO: Implement template rendering with handlebars
    Ok(Value::String(template))
}

fn fn_templatefile(args: &[Value]) -> Result<Value> {
    require_args_min(args, 1, "templatefile")?;
    let path = as_string(&args[0])?;
    let template = std::fs::read_to_string(path)?;

    // TODO: Implement template rendering
    Ok(Value::String(template))
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
    // TODO: Implement proper try/catch for expressions
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
}
