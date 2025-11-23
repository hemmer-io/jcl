//! Evaluator for JCL - resolves variables, functions, and expressions

use crate::ast::{
    BinaryOperator, Expression, ImportKind, Module, Pattern, SourceSpan, Statement, StringPart,
    UnaryOperator, Value, WhenArm,
};
use crate::functions;
use crate::module_source::ModuleSourceResolver;
use anyhow::{anyhow, Result};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Evaluated module with all expressions resolved
#[derive(Debug)]
pub struct EvaluatedModule {
    pub bindings: HashMap<String, Value>,
}

/// Import trace entry for debugging
#[derive(Debug, Clone)]
pub struct ImportTrace {
    pub importer: Option<PathBuf>,
    pub imported: PathBuf,
    pub kind: String,
    pub cached: bool,
    pub duration_ms: u128,
}

/// Import performance metrics
#[derive(Debug, Clone, Default)]
pub struct ImportMetrics {
    pub total_imports: usize,
    pub cache_hits: usize,
    pub total_time_ms: u128,
    pub traces: Vec<ImportTrace>,
}

impl ImportMetrics {
    pub fn cache_hit_rate(&self) -> f64 {
        if self.total_imports == 0 {
            0.0
        } else {
            (self.cache_hits as f64 / self.total_imports as f64) * 100.0
        }
    }
}

/// Module metadata
#[derive(Debug, Clone)]
pub struct ModuleMetadata {
    pub version: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
}

/// Module interface definition
#[derive(Debug, Clone)]
pub struct ModuleInterface {
    pub inputs: HashMap<String, crate::ast::ModuleInput>,
    pub outputs: HashMap<String, crate::ast::ModuleOutput>,
    pub metadata: Option<ModuleMetadata>,
}

/// Evaluated module outputs
#[derive(Debug, Clone)]
pub struct ModuleOutputs {
    pub outputs: HashMap<String, Value>,
}

/// Evaluator context
pub struct Evaluator {
    pub variables: HashMap<String, Value>,
    pub functions: HashMap<String, Value>,
    /// Lazy variable expressions (not yet evaluated) - uses RefCell for interior mutability
    lazy_vars: RefCell<HashMap<String, Expression>>,
    /// Type annotations for lazy variables
    lazy_type_annotations: RefCell<HashMap<String, crate::ast::Type>>,
    /// Variables currently being evaluated (for cycle detection) - uses RefCell for interior mutability
    evaluating: RefCell<HashSet<String>>,
    /// Current file being evaluated (for relative import resolution)
    current_file: RefCell<Option<PathBuf>>,
    /// Files currently being imported (for circular dependency detection)
    importing: RefCell<HashSet<PathBuf>>,
    /// Cache of already-imported modules to avoid re-evaluation
    import_cache: RefCell<HashMap<PathBuf, HashMap<String, Value>>>,
    /// Import tracing enabled (for debugging)
    pub trace_imports: bool,
    /// Import metrics collection
    import_metrics: RefCell<ImportMetrics>,
    /// Module interface cache (path -> interface definition)
    module_interface_cache: RefCell<HashMap<PathBuf, ModuleInterface>>,
    /// Module output cache (path -> evaluated outputs)
    module_output_cache: RefCell<HashMap<PathBuf, ModuleOutputs>>,
    /// Current module inputs (for module.inputs access within a module)
    current_module_inputs: RefCell<Option<HashMap<String, Value>>>,
    /// Modules currently being instantiated (for circular dependency detection)
    instantiating_modules: RefCell<Vec<PathBuf>>,
    /// Module source resolver (for external module sources)
    module_source_resolver: RefCell<ModuleSourceResolver>,
}

impl Evaluator {
    /// Create a new evaluator
    pub fn new() -> Self {
        let mut evaluator = Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            lazy_vars: RefCell::new(HashMap::new()),
            lazy_type_annotations: RefCell::new(HashMap::new()),
            evaluating: RefCell::new(HashSet::new()),
            current_file: RefCell::new(None),
            importing: RefCell::new(HashSet::new()),
            import_cache: RefCell::new(HashMap::new()),
            trace_imports: false,
            import_metrics: RefCell::new(ImportMetrics::default()),
            module_interface_cache: RefCell::new(HashMap::new()),
            module_output_cache: RefCell::new(HashMap::new()),
            current_module_inputs: RefCell::new(None),
            instantiating_modules: RefCell::new(Vec::new()),
            module_source_resolver: RefCell::new(ModuleSourceResolver::new(None)),
        };
        evaluator.register_builtins();
        evaluator
    }

    /// Set the current file being evaluated (for relative imports)
    pub fn set_current_file<P: AsRef<Path>>(&self, path: P) {
        *self.current_file.borrow_mut() = Some(path.as_ref().to_path_buf());
    }

    /// Enable import tracing for debugging
    pub fn enable_import_tracing(&mut self) {
        self.trace_imports = true;
    }

    /// Get import metrics
    pub fn get_import_metrics(&self) -> ImportMetrics {
        self.import_metrics.borrow().clone()
    }

    /// Print import trace (for debugging)
    pub fn print_import_trace(&self) {
        let metrics = self.import_metrics.borrow();
        if metrics.traces.is_empty() {
            println!("No imports traced.");
            return;
        }

        println!("\n=== Import Trace ===");
        println!(
            "Total imports: {} ({} cached, {:.1}% cache hit rate)",
            metrics.total_imports,
            metrics.cache_hits,
            metrics.cache_hit_rate()
        );
        println!("Total time: {}ms\n", metrics.total_time_ms);

        for (i, trace) in metrics.traces.iter().enumerate() {
            let importer_str = trace
                .importer
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "<root>".to_string());

            let status = if trace.cached { "[CACHED]" } else { "[LOADED]" };

            println!("  {}. {} {}ms", i + 1, status, trace.duration_ms);
            println!("     From: {}", importer_str);
            println!("     Import: {}", trace.imported.display());
            println!("     Kind: {}", trace.kind);
            println!();
        }
    }

    /// Generate import graph in DOT format for visualization
    pub fn generate_import_graph(&self) -> String {
        let metrics = self.import_metrics.borrow();
        let mut dot = String::from("digraph imports {\n");
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [shape=box, style=rounded];\n\n");

        let mut nodes = HashSet::new();
        for trace in &metrics.traces {
            let imported = trace.imported.display().to_string();
            nodes.insert(imported.clone());

            if let Some(importer) = &trace.importer {
                let importer_str = importer.display().to_string();
                nodes.insert(importer_str.clone());

                let style = if trace.cached {
                    " [style=dashed, color=gray]"
                } else {
                    ""
                };

                dot.push_str(&format!(
                    "  \"{}\" -> \"{}\"{};\n",
                    importer_str, imported, style
                ));
            } else {
                // Root import
                dot.push_str(&format!("  \"<root>\" -> \"{}\";\n", imported));
            }
        }

        dot.push_str("}\n");
        dot
    }

    /// Evaluate a module
    pub fn evaluate(&mut self, module: Module) -> Result<EvaluatedModule> {
        let mut bindings = HashMap::new();

        for statement in module.statements {
            match statement {
                Statement::Assignment {
                    name,
                    value,
                    type_annotation,
                    ..
                } => {
                    // Check if this is a lambda/function - these need to be evaluated eagerly
                    // so they can be called immediately
                    let is_function = matches!(value, Expression::Lambda { .. });

                    if is_function {
                        // Evaluate functions eagerly
                        let evaluated_value = self.evaluate_expression(&value)?;

                        // Validate type annotation if present
                        if let Some(expected_type) = type_annotation {
                            let actual_type = evaluated_value.get_type();
                            if !self.type_matches(&actual_type, &expected_type) {
                                return Err(anyhow!(
                                    "Type mismatch for variable '{}': expected {}, got {}",
                                    name,
                                    expected_type,
                                    actual_type
                                ));
                            }
                        }

                        self.variables.insert(name.clone(), evaluated_value.clone());
                        bindings.insert(name, evaluated_value);
                    } else {
                        // Store non-function expressions as lazy - will be evaluated on first access
                        self.lazy_vars
                            .borrow_mut()
                            .insert(name.clone(), value.clone());

                        // Store type annotation for validation during lazy evaluation
                        if let Some(ty) = type_annotation {
                            self.lazy_type_annotations
                                .borrow_mut()
                                .insert(name.clone(), ty);
                        }
                    }
                }
                Statement::FunctionDef {
                    name, params, body, ..
                } => {
                    let func = Value::Function {
                        params,
                        body: Box::new(body),
                    };
                    self.functions.insert(name.clone(), func.clone());
                    bindings.insert(name, func);
                }
                Statement::ForLoop { .. } => {
                    // For loops generate multiple statements - not yet implemented
                    return Err(anyhow!("For loops are not yet implemented in evaluator"));
                }
                Statement::Import { path, kind, .. } => {
                    // Evaluate the import
                    self.evaluate_import(&path, &kind)?;
                }
                Statement::Expression { expr, .. } => {
                    // Expression statements - evaluate but don't bind
                    self.evaluate_expression(&expr)?;
                }
                Statement::ModuleMetadata {
                    version,
                    description,
                    author,
                    license,
                    ..
                } => {
                    // Store module metadata in the interface cache
                    // If interface already exists, update it; otherwise create a new one
                    if let Some(ref current_path) = *self.current_file.borrow() {
                        let mut cache = self.module_interface_cache.borrow_mut();
                        if let Some(interface) = cache.get_mut(current_path) {
                            interface.metadata = Some(ModuleMetadata {
                                version: version.clone(),
                                description: description.clone(),
                                author: author.clone(),
                                license: license.clone(),
                            });
                        } else {
                            // Create minimal interface with just metadata
                            cache.insert(
                                current_path.clone(),
                                ModuleInterface {
                                    inputs: HashMap::new(),
                                    outputs: HashMap::new(),
                                    metadata: Some(ModuleMetadata {
                                        version: version.clone(),
                                        description: description.clone(),
                                        author: author.clone(),
                                        license: license.clone(),
                                    }),
                                },
                            );
                        }
                    }
                }
                Statement::ModuleInterface {
                    inputs, outputs, ..
                } => {
                    // Store the module interface for later validation
                    // This should only appear in module files, not main files
                    if let Some(ref current_path) = *self.current_file.borrow() {
                        let mut cache = self.module_interface_cache.borrow_mut();
                        if let Some(interface) = cache.get_mut(current_path) {
                            // Update existing interface (metadata may have been set first)
                            interface.inputs = inputs.clone();
                            interface.outputs = outputs.clone();
                        } else {
                            cache.insert(
                                current_path.clone(),
                                ModuleInterface {
                                    inputs: inputs.clone(),
                                    outputs: outputs.clone(),
                                    metadata: None,
                                },
                            );
                        }
                    }
                }
                Statement::ModuleOutputs { outputs, .. } => {
                    // Evaluate module outputs and store them
                    // This should only be called within a module context
                    let mut evaluated_outputs = HashMap::new();
                    for (name, expr) in outputs {
                        let value = self.evaluate_expression(&expr)?;
                        evaluated_outputs.insert(name.clone(), value);
                    }

                    // Store in the module outputs for the current file
                    if let Some(ref current_path) = *self.current_file.borrow() {
                        self.module_output_cache.borrow_mut().insert(
                            current_path.clone(),
                            ModuleOutputs {
                                outputs: evaluated_outputs,
                            },
                        );
                    }
                }
                Statement::ModuleInstance {
                    module_type,
                    instance_name,
                    source,
                    when,
                    count,
                    for_each,
                    inputs: input_exprs,
                    ..
                } => {
                    // Check when condition if present
                    if let Some(when_expr) = when {
                        let condition_value = self.evaluate_expression(&when_expr)?;
                        let should_instantiate = match condition_value {
                            Value::Bool(b) => b,
                            _ => {
                                return Err(anyhow!(
                                    "Module 'condition' must evaluate to a boolean, got {:?}",
                                    condition_value
                                ));
                            }
                        };

                        if !should_instantiate {
                            // Skip this module instantiation
                            continue;
                        }
                    }

                    // Handle count/for_each meta-arguments
                    if let Some(count_expr) = count {
                        // Evaluate count and create N instances
                        self.evaluate_module_count(
                            &module_type,
                            &instance_name,
                            &source,
                            &count_expr,
                            &input_exprs,
                            &mut bindings,
                        )?;
                    } else if let Some(for_each_expr) = for_each {
                        // Evaluate for_each and create instances for each element
                        self.evaluate_module_for_each(
                            &module_type,
                            &instance_name,
                            &source,
                            &for_each_expr,
                            &input_exprs,
                            &mut bindings,
                        )?;
                    } else {
                        // Single module instance (original behavior)
                        let outputs = self.evaluate_module_instance(&source, &input_exprs)?;

                        // Create nested structure: module.<type>.<instance> = outputs
                        // Get or create the "module" map
                        let module_map = if let Some(Value::Map(m)) = self.variables.get("module") {
                            m.clone()
                        } else {
                            HashMap::new()
                        };

                        // Get or create the module_type map within module
                        let mut module_map = module_map;
                        let type_map =
                            if let Some(Value::Map(m)) = module_map.get(&module_type.to_string()) {
                                m.clone()
                            } else {
                                HashMap::new()
                            };

                        // Insert instance into type map
                        let mut type_map = type_map;
                        type_map.insert(instance_name.clone(), Value::Map(outputs));

                        // Update module map
                        module_map.insert(module_type.clone(), Value::Map(type_map.clone()));

                        // Store the updated module map
                        self.variables
                            .insert("module".to_string(), Value::Map(module_map.clone()));
                        bindings.insert("module".to_string(), Value::Map(module_map));
                    }
                }
            }
        }

        // Force evaluation of all lazy variables for the final bindings
        // Clone the keys to avoid borrow issues
        let lazy_var_names: Vec<String> = self.lazy_vars.borrow().keys().cloned().collect();

        for name in lazy_var_names {
            // Evaluate the lazy variable
            let value = self.evaluate_lazy_var(&name)?;

            // Cache it in variables and add to bindings
            self.variables.insert(name.clone(), value.clone());
            bindings.insert(name.clone(), value);

            // Remove from lazy_vars since it's now evaluated
            self.lazy_vars.borrow_mut().remove(&name);
        }

        Ok(EvaluatedModule { bindings })
    }

    /// Evaluate an expression
    pub fn evaluate_expression(&self, expr: &Expression) -> Result<Value> {
        match expr {
            Expression::Literal { value, .. } => Ok(value.clone()),

            Expression::Variable { name, .. } => {
                // Check already-evaluated variables first
                if let Some(value) = self.variables.get(name) {
                    return Ok(value.clone());
                }

                // Check functions
                if let Some(func) = self.functions.get(name) {
                    return Ok(func.clone());
                }

                // Check if it's a lazy variable that needs evaluation
                if self.lazy_vars.borrow().contains_key(name) {
                    return self.evaluate_lazy_var(name);
                }

                // Variable not found
                Err(anyhow!("Undefined variable: {}", name))
            }

            Expression::List { elements, .. } => {
                let mut values = Vec::new();
                for item in elements {
                    values.push(self.evaluate_expression(item)?);
                }
                Ok(Value::List(values))
            }

            Expression::Map { entries, .. } => {
                let mut map = HashMap::new();
                for (key, value_expr) in entries {
                    let value = self.evaluate_expression(value_expr)?;
                    map.insert(key.clone(), value);
                }
                Ok(Value::Map(map))
            }

            Expression::MemberAccess {
                object,
                field,
                span,
            } => {
                // Special handling for module.inputs
                if let Expression::Variable { name, .. } = &**object {
                    if name == "module" && field == "inputs" {
                        // Return the current module inputs as a map
                        if let Some(ref inputs) = *self.current_module_inputs.borrow() {
                            return Ok(Value::Map(inputs.clone()));
                        } else {
                            return Err(anyhow!(
                                "module.inputs can only be accessed within a module context"
                            ));
                        }
                    }
                }

                // Check if the object expression contains a Splat
                if self.contains_splat(object) {
                    // Evaluate with splat semantics
                    return self.evaluate_splat_access(object, field, span);
                }

                // Normal member access
                let obj_value = self.evaluate_expression(object)?;
                match obj_value {
                    Value::Map(map) => map
                        .get(field)
                        .cloned()
                        .ok_or_else(|| anyhow!("Field not found: {}", field)),
                    _ => Err(anyhow!("Cannot access member on non-map value")),
                }
            }

            Expression::OptionalChain { object, field, .. } => {
                let obj_value = self.evaluate_expression(object)?;
                if obj_value.is_null() {
                    return Ok(Value::Null);
                }
                match obj_value {
                    Value::Map(map) => Ok(map.get(field).cloned().unwrap_or(Value::Null)),
                    _ => Ok(Value::Null),
                }
            }

            Expression::Index { object, index, .. } => {
                // Phase 3A Optimization: If indexing a list comprehension, only evaluate needed item
                // Only optimize for non-negative indices (negative indices need full list length)
                // Only optimize for single-iterator comprehensions (multi-iterator would be complex)
                if let Expression::ListComprehension {
                    expr,
                    iterators,
                    condition,
                    ..
                } = object.as_ref()
                {
                    if iterators.len() == 1 {
                        let (variable, iterable) = &iterators[0];
                        let index_value = self.evaluate_expression(index)?;
                        if let Value::Int(target_idx) = index_value {
                            // Only optimize for non-negative indices
                            if target_idx >= 0 {
                                return self.evaluate_comprehension_at_index(
                                    expr,
                                    variable,
                                    iterable,
                                    condition.as_ref().map(|v| &**v),
                                    target_idx,
                                );
                            }
                        }
                    }
                }

                // Standard evaluation for non-comprehension indexing
                let obj_value = self.evaluate_expression(object)?;
                let index_value = self.evaluate_expression(index)?;

                match obj_value {
                    Value::List(list) => {
                        if let Value::Int(i) = index_value {
                            let idx = if i < 0 {
                                (list.len() as i64 + i) as usize
                            } else {
                                i as usize
                            };
                            list.get(idx)
                                .cloned()
                                .ok_or_else(|| anyhow!("Index out of bounds: {}", i))
                        } else {
                            Err(anyhow!("List index must be an integer"))
                        }
                    }
                    Value::Map(map) => {
                        if let Value::String(key) = index_value {
                            map.get(&key)
                                .cloned()
                                .ok_or_else(|| anyhow!("Key not found: {}", key))
                        } else {
                            Err(anyhow!("Map key must be a string"))
                        }
                    }
                    _ => Err(anyhow!("Cannot index non-list/non-map value")),
                }
            }

            Expression::Slice {
                object,
                start,
                end,
                step,
                ..
            } => {
                let obj_value = self.evaluate_expression(object)?;

                match obj_value {
                    Value::List(list) => {
                        let list_len = list.len() as i64;

                        // Evaluate step first to determine defaults for start/end
                        let step_val = if let Some(st) = step {
                            let val = self.evaluate_expression(st)?;
                            if let Value::Int(i) = val {
                                if i == 0 {
                                    return Err(anyhow!("Slice step cannot be zero"));
                                }
                                i
                            } else {
                                return Err(anyhow!("Slice step must be an integer"));
                            }
                        } else {
                            1
                        };

                        // Evaluate slice parameters with defaults based on step direction
                        let start_idx = if let Some(s) = start {
                            let val = self.evaluate_expression(s)?;
                            if let Value::Int(i) = val {
                                i
                            } else {
                                return Err(anyhow!("Slice start must be an integer"));
                            }
                        } else if step_val < 0 {
                            // For negative step, default start is end of list
                            list_len - 1
                        } else {
                            0
                        };

                        let end_idx = if let Some(e) = end {
                            let val = self.evaluate_expression(e)?;
                            if let Value::Int(i) = val {
                                i
                            } else {
                                return Err(anyhow!("Slice end must be an integer"));
                            }
                        } else if step_val < 0 {
                            // For negative step, default end is before beginning
                            -list_len - 1
                        } else {
                            list_len
                        };

                        // Normalize negative indices
                        let norm_start = if start_idx < 0 {
                            (list_len + start_idx).max(0)
                        } else {
                            start_idx.min(list_len)
                        };

                        let norm_end = if end_idx < 0 {
                            list_len + end_idx
                        } else {
                            end_idx.min(list_len)
                        };

                        // Perform slicing
                        let mut result = Vec::new();

                        if step_val > 0 {
                            // Forward iteration
                            let mut i = norm_start;
                            while i < norm_end {
                                if let Some(val) = list.get(i as usize) {
                                    result.push(val.clone());
                                }
                                i += step_val;
                            }
                        } else {
                            // Backward iteration (reverse)
                            let mut i = norm_start;
                            while i > norm_end {
                                if let Some(val) = list.get(i as usize) {
                                    result.push(val.clone());
                                }
                                i += step_val; // step_val is negative
                            }
                        }

                        Ok(Value::List(result))
                    }
                    _ => Err(anyhow!("Cannot slice non-list value")),
                }
            }

            Expression::Range {
                start,
                end,
                step,
                inclusive,
                ..
            } => {
                // Evaluate start, end, and optional step
                let start_val = self.evaluate_expression(start)?;
                let end_val = self.evaluate_expression(end)?;

                // Support both integer and float ranges
                match (start_val, end_val) {
                    (Value::Int(s), Value::Int(e)) => {
                        let step_val = if let Some(st) = step {
                            let val = self.evaluate_expression(st)?;
                            if let Value::Int(i) = val {
                                if i == 0 {
                                    return Err(anyhow!("Range step cannot be zero"));
                                }
                                i
                            } else {
                                return Err(anyhow!("Range step must be an integer"));
                            }
                        } else {
                            // Default step: 1 if ascending, -1 if descending
                            if s <= e {
                                1
                            } else {
                                -1
                            }
                        };

                        // Generate the range
                        let mut result = Vec::new();
                        let end_adjusted = if *inclusive { e } else { e - step_val.signum() };

                        if step_val > 0 {
                            // Forward iteration
                            let mut i = s;
                            while i <= end_adjusted {
                                result.push(Value::Int(i));
                                i += step_val;
                            }
                        } else {
                            // Backward iteration
                            let mut i = s;
                            while i >= end_adjusted {
                                result.push(Value::Int(i));
                                i += step_val; // step_val is negative
                            }
                        }

                        Ok(Value::List(result))
                    }
                    (Value::Float(s), Value::Float(e)) => {
                        let step_val = if let Some(st) = step {
                            let val = self.evaluate_expression(st)?;
                            if let Value::Float(f) = val {
                                if f == 0.0 {
                                    return Err(anyhow!("Range step cannot be zero"));
                                }
                                f
                            } else {
                                return Err(anyhow!("Range step must be a float"));
                            }
                        } else {
                            // Default step: 1.0 if ascending, -1.0 if descending
                            if s <= e {
                                1.0
                            } else {
                                -1.0
                            }
                        };

                        // Generate the range
                        let mut result = Vec::new();
                        let end_adjusted = if *inclusive { e } else { e - step_val.signum() };

                        if step_val > 0.0 {
                            // Forward iteration
                            let mut i = s;
                            while i <= end_adjusted {
                                result.push(Value::Float(i));
                                i += step_val;
                            }
                        } else {
                            // Backward iteration
                            let mut i = s;
                            while i >= end_adjusted {
                                result.push(Value::Float(i));
                                i += step_val; // step_val is negative
                            }
                        }

                        Ok(Value::List(result))
                    }
                    _ => Err(anyhow!(
                        "Range requires both start and end to be integers or floats"
                    )),
                }
            }

            Expression::FunctionCall { name, args, .. } => self.call_function(name, args),

            Expression::MethodCall {
                object,
                method,
                args,
                ..
            } => {
                // For method calls, prepend object to args
                let obj_value = self.evaluate_expression(object)?;
                let mut all_args = vec![Expression::Literal {
                    value: obj_value,
                    span: None,
                }];
                all_args.extend_from_slice(args);
                self.call_function(method, &all_args)
            }

            Expression::BinaryOp {
                op, left, right, ..
            } => {
                let left_val = self.evaluate_expression(left)?;
                let right_val = self.evaluate_expression(right)?;
                self.evaluate_binary_op(*op, left_val, right_val)
            }

            Expression::UnaryOp { op, operand, .. } => {
                let operand_val = self.evaluate_expression(operand)?;
                self.evaluate_unary_op(*op, operand_val)
            }

            Expression::Ternary {
                condition,
                then_expr,
                else_expr,
                ..
            } => {
                let cond_val = self.evaluate_expression(condition)?;
                if self.is_truthy(&cond_val) {
                    self.evaluate_expression(then_expr)
                } else {
                    self.evaluate_expression(else_expr)
                }
            }

            Expression::If {
                condition,
                then_expr,
                else_expr,
                ..
            } => {
                let cond_val = self.evaluate_expression(condition)?;
                if self.is_truthy(&cond_val) {
                    self.evaluate_expression(then_expr)
                } else if let Some(else_e) = else_expr {
                    self.evaluate_expression(else_e)
                } else {
                    Ok(Value::Null)
                }
            }

            Expression::When { value, arms, .. } => {
                let val = self.evaluate_expression(value)?;
                self.evaluate_when(&val, arms)
            }

            Expression::Lambda { params, body, .. } => Ok(Value::Function {
                params: params.clone(),
                body: body.clone(),
            }),

            Expression::Let { bindings, body, .. } => {
                // Evaluate let expression with local bindings
                // Each binding creates a new scope that's visible to subsequent bindings and the body
                let mut scoped_eval = self.clone_with_var(&bindings[0].0, {
                    self.evaluate_expression(&bindings[0].1)?
                });

                // Process remaining bindings in order, each seeing previous bindings
                for (name, expr) in &bindings[1..] {
                    let value = scoped_eval.evaluate_expression(expr)?;
                    scoped_eval = scoped_eval.clone_with_var(name, value);
                }

                // Evaluate body in the scope with all bindings
                scoped_eval.evaluate_expression(body)
            }

            Expression::ListComprehension {
                expr,
                iterators,
                condition,
                ..
            } => {
                // Recursively evaluate nested iterations
                // [expr for x in list1 for y in list2 if cond]
                // is equivalent to:
                // for x in list1:
                //     for y in list2:
                //         if cond: results.append(expr)

                self.evaluate_comprehension_recursive(
                    expr,
                    iterators,
                    condition.as_ref().map(|c| &**c),
                    0,
                )
            }

            Expression::Pipeline { stages, .. } => {
                if stages.is_empty() {
                    return Ok(Value::Null);
                }

                let mut result = self.evaluate_expression(&stages[0])?;

                for stage in &stages[1..] {
                    // Each stage should be a function call
                    match stage {
                        Expression::FunctionCall { name, args, .. } => {
                            // Prepend result to args
                            let mut all_args = vec![Expression::Literal {
                                value: result,
                                span: None,
                            }];
                            all_args.extend_from_slice(args);
                            result = self.call_function(name, &all_args)?;
                        }
                        Expression::Variable {
                            name: func_name, ..
                        } => {
                            // Simple function with just piped value
                            result = self.call_function(
                                func_name,
                                &[Expression::Literal {
                                    value: result,
                                    span: None,
                                }],
                            )?;
                        }
                        _ => {
                            return Err(anyhow!("Pipeline stage must be a function call"));
                        }
                    }
                }

                Ok(result)
            }

            Expression::Try { expr, default, .. } => match self.evaluate_expression(expr) {
                Ok(val) => Ok(val),
                Err(_) => {
                    if let Some(def) = default {
                        self.evaluate_expression(def)
                    } else {
                        Ok(Value::Null)
                    }
                }
            },

            Expression::InterpolatedString { parts, .. } => {
                let mut result = String::new();
                for part in parts {
                    match part {
                        StringPart::Literal(s) => result.push_str(s),
                        StringPart::Interpolation(expr) => {
                            let val = self.evaluate_expression(expr)?;
                            result.push_str(&val.to_string_repr());
                        }
                    }
                }
                Ok(Value::String(result))
            }

            Expression::Spread { .. } => {
                Err(anyhow!("Spread operator can only be used in collections"))
            }

            Expression::Splat { object, .. } => {
                let obj_value = self.evaluate_expression(object)?;

                // Validate that splat is applied to a list
                match obj_value {
                    Value::List(_) => {
                        // Return the list as-is
                        // The actual splatting happens in MemberAccess evaluation
                        Ok(obj_value)
                    }
                    _ => Err(anyhow!("Splat operator [*] requires a list")),
                }
            }
        }
    }

    /// Check if an expression contains a Splat operator
    fn contains_splat(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Splat { .. } => true,
            Expression::MemberAccess { object, .. } => self.contains_splat(object),
            Expression::Index { object, .. } => self.contains_splat(object),
            _ => false,
        }
    }

    /// Evaluate member access on a splat expression
    fn evaluate_splat_access(
        &self,
        expr: &Expression,
        field: &str,
        _span: &Option<SourceSpan>,
    ) -> Result<Value> {
        match expr {
            Expression::Splat { object, .. } => {
                // Base case: evaluate object and map field access over it
                let obj_value = self.evaluate_expression(object)?;
                match obj_value {
                    Value::List(items) => {
                        let results: Result<Vec<Value>> = items
                            .iter()
                            .map(|item| self.get_field(item, field))
                            .collect();
                        Ok(Value::List(results?))
                    }
                    _ => Err(anyhow!("Splat requires a list")),
                }
            }
            Expression::MemberAccess {
                object,
                field: inner_field,
                ..
            } => {
                // Recursive case: evaluate inner access first, then apply field
                let inner_list = self.evaluate_splat_access(object, inner_field, _span)?;
                match inner_list {
                    Value::List(items) => {
                        let results: Result<Vec<Value>> = items
                            .iter()
                            .map(|item| self.get_field(item, field))
                            .collect();
                        Ok(Value::List(results?))
                    }
                    _ => Err(anyhow!("Expected list from splat")),
                }
            }
            _ => {
                // Shouldn't reach here if contains_splat is correct
                Err(anyhow!("Invalid splat expression"))
            }
        }
    }

    /// Get a field from a value
    fn get_field(&self, value: &Value, field: &str) -> Result<Value> {
        match value {
            Value::Map(map) => map
                .get(field)
                .cloned()
                .ok_or_else(|| anyhow!("Field '{}' not found", field)),
            _ => Err(anyhow!("Cannot access field on non-map value")),
        }
    }

    /// Evaluate binary operations
    fn evaluate_binary_op(&self, op: BinaryOperator, left: Value, right: Value) -> Result<Value> {
        match op {
            BinaryOperator::Add => match (left, right) {
                (Value::Int(l), Value::Int(r)) => Ok(Value::Int(l + r)),
                (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l + r)),
                (Value::Int(l), Value::Float(r)) => Ok(Value::Float(l as f64 + r)),
                (Value::Float(l), Value::Int(r)) => Ok(Value::Float(l + r as f64)),
                _ => Err(anyhow!("Invalid operands for +")),
            },

            BinaryOperator::Subtract => match (left, right) {
                (Value::Int(l), Value::Int(r)) => Ok(Value::Int(l - r)),
                (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l - r)),
                (Value::Int(l), Value::Float(r)) => Ok(Value::Float(l as f64 - r)),
                (Value::Float(l), Value::Int(r)) => Ok(Value::Float(l - r as f64)),
                _ => Err(anyhow!("Invalid operands for -")),
            },

            BinaryOperator::Multiply => match (left, right) {
                (Value::Int(l), Value::Int(r)) => Ok(Value::Int(l * r)),
                (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l * r)),
                (Value::Int(l), Value::Float(r)) => Ok(Value::Float(l as f64 * r)),
                (Value::Float(l), Value::Int(r)) => Ok(Value::Float(l * r as f64)),
                _ => Err(anyhow!("Invalid operands for *")),
            },

            BinaryOperator::Divide => match (left, right) {
                (Value::Int(l), Value::Int(r)) => {
                    if r == 0 {
                        Err(anyhow!("Division by zero"))
                    } else {
                        Ok(Value::Int(l / r))
                    }
                }
                (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l / r)),
                (Value::Int(l), Value::Float(r)) => Ok(Value::Float(l as f64 / r)),
                (Value::Float(l), Value::Int(r)) => Ok(Value::Float(l / r as f64)),
                _ => Err(anyhow!("Invalid operands for /")),
            },

            BinaryOperator::Modulo => match (left, right) {
                (Value::Int(l), Value::Int(r)) => {
                    if r == 0 {
                        Err(anyhow!("Modulo by zero"))
                    } else {
                        Ok(Value::Int(l % r))
                    }
                }
                _ => Err(anyhow!("Modulo requires integer operands")),
            },

            BinaryOperator::Power => match (left, right) {
                (Value::Int(l), Value::Int(r)) => {
                    if r < 0 {
                        Ok(Value::Float((l as f64).powf(r as f64)))
                    } else {
                        Ok(Value::Int(l.pow(r as u32)))
                    }
                }
                (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l.powf(r))),
                (Value::Int(l), Value::Float(r)) => Ok(Value::Float((l as f64).powf(r))),
                (Value::Float(l), Value::Int(r)) => Ok(Value::Float(l.powf(r as f64))),
                _ => Err(anyhow!("Invalid operands for **")),
            },

            BinaryOperator::Equal => Ok(Value::Bool(self.values_equal(&left, &right))),
            BinaryOperator::NotEqual => Ok(Value::Bool(!self.values_equal(&left, &right))),

            BinaryOperator::LessThan => match (left, right) {
                (Value::Int(l), Value::Int(r)) => Ok(Value::Bool(l < r)),
                (Value::Float(l), Value::Float(r)) => Ok(Value::Bool(l < r)),
                (Value::Int(l), Value::Float(r)) => Ok(Value::Bool((l as f64) < r)),
                (Value::Float(l), Value::Int(r)) => Ok(Value::Bool(l < (r as f64))),
                (Value::String(l), Value::String(r)) => Ok(Value::Bool(l < r)),
                _ => Err(anyhow!("Invalid operands for <")),
            },

            BinaryOperator::LessThanOrEqual => match (left, right) {
                (Value::Int(l), Value::Int(r)) => Ok(Value::Bool(l <= r)),
                (Value::Float(l), Value::Float(r)) => Ok(Value::Bool(l <= r)),
                (Value::Int(l), Value::Float(r)) => Ok(Value::Bool((l as f64) <= r)),
                (Value::Float(l), Value::Int(r)) => Ok(Value::Bool(l <= (r as f64))),
                (Value::String(l), Value::String(r)) => Ok(Value::Bool(l <= r)),
                _ => Err(anyhow!("Invalid operands for <=")),
            },

            BinaryOperator::GreaterThan => match (left, right) {
                (Value::Int(l), Value::Int(r)) => Ok(Value::Bool(l > r)),
                (Value::Float(l), Value::Float(r)) => Ok(Value::Bool(l > r)),
                (Value::Int(l), Value::Float(r)) => Ok(Value::Bool((l as f64) > r)),
                (Value::Float(l), Value::Int(r)) => Ok(Value::Bool(l > (r as f64))),
                (Value::String(l), Value::String(r)) => Ok(Value::Bool(l > r)),
                _ => Err(anyhow!("Invalid operands for >")),
            },

            BinaryOperator::GreaterThanOrEqual => match (left, right) {
                (Value::Int(l), Value::Int(r)) => Ok(Value::Bool(l >= r)),
                (Value::Float(l), Value::Float(r)) => Ok(Value::Bool(l >= r)),
                (Value::Int(l), Value::Float(r)) => Ok(Value::Bool((l as f64) >= r)),
                (Value::Float(l), Value::Int(r)) => Ok(Value::Bool(l >= (r as f64))),
                (Value::String(l), Value::String(r)) => Ok(Value::Bool(l >= r)),
                _ => Err(anyhow!("Invalid operands for >=")),
            },

            BinaryOperator::And => {
                let left_truthy = self.is_truthy(&left);
                if !left_truthy {
                    Ok(left)
                } else {
                    Ok(right)
                }
            }

            BinaryOperator::Or => {
                let left_truthy = self.is_truthy(&left);
                if left_truthy {
                    Ok(left)
                } else {
                    Ok(right)
                }
            }

            BinaryOperator::NullCoalesce => {
                if left.is_null() {
                    Ok(right)
                } else {
                    Ok(left)
                }
            }

            BinaryOperator::Concat => match (left, right) {
                (Value::String(l), Value::String(r)) => Ok(Value::String(l + &r)),
                (Value::List(mut l), Value::List(r)) => {
                    l.extend(r);
                    Ok(Value::List(l))
                }
                _ => Err(anyhow!("Invalid operands for ++")),
            },
        }
    }

    /// Evaluate unary operations
    fn evaluate_unary_op(&self, op: UnaryOperator, operand: Value) -> Result<Value> {
        match op {
            UnaryOperator::Not => Ok(Value::Bool(!self.is_truthy(&operand))),
            UnaryOperator::Negate => match operand {
                Value::Int(i) => Ok(Value::Int(-i)),
                Value::Float(f) => Ok(Value::Float(-f)),
                _ => Err(anyhow!("Cannot negate non-numeric value")),
            },
        }
    }

    /// Evaluate when expression (pattern matching)
    fn evaluate_when(&self, value: &Value, arms: &[WhenArm]) -> Result<Value> {
        for arm in arms {
            if self.pattern_matches(&arm.pattern, value)? {
                // Check guard if present
                if let Some(guard) = &arm.guard {
                    let guard_val = self.evaluate_expression(guard)?;
                    if !self.is_truthy(&guard_val) {
                        continue;
                    }
                }
                return self.evaluate_expression(&arm.expr);
            }
        }
        Err(anyhow!("No matching pattern in when expression"))
    }

    /// Check if pattern matches value
    fn pattern_matches(&self, pattern: &Pattern, value: &Value) -> Result<bool> {
        match pattern {
            Pattern::Wildcard => Ok(true),
            Pattern::Literal(lit) => Ok(self.values_equal(lit, value)),
            Pattern::Variable(_) => Ok(true), // Variables always match
            Pattern::Tuple(patterns) => {
                if let Value::List(values) = value {
                    if patterns.len() != values.len() {
                        return Ok(false);
                    }
                    for (pat, val) in patterns.iter().zip(values.iter()) {
                        if !self.pattern_matches(pat, val)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    /// Check if two values are equal
    fn values_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Int(l), Value::Int(r)) => l == r,
            (Value::Float(l), Value::Float(r)) => (l - r).abs() < f64::EPSILON,
            (Value::Int(l), Value::Float(r)) => (*l as f64 - r).abs() < f64::EPSILON,
            (Value::Float(l), Value::Int(r)) => (l - *r as f64).abs() < f64::EPSILON,
            (Value::String(l), Value::String(r)) => l == r,
            (Value::Bool(l), Value::Bool(r)) => l == r,
            (Value::Null, Value::Null) => true,
            (Value::List(l), Value::List(r)) => {
                l.len() == r.len() && l.iter().zip(r.iter()).all(|(a, b)| self.values_equal(a, b))
            }
            _ => false,
        }
    }

    /// Check if a value is truthy
    fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Bool(b) => *b,
            Value::Null => false,
            Value::Int(0) => false,
            Value::String(s) => !s.is_empty(),
            Value::List(l) => !l.is_empty(),
            _ => true,
        }
    }

    /// Phase 3A: Evaluate list comprehension at a specific index
    /// Only evaluates items until the target index is reached
    fn evaluate_comprehension_at_index(
        &self,
        expr: &Expression,
        variable: &str,
        iterable: &Expression,
        condition: Option<&Expression>,
        target_idx: i64,
    ) -> Result<Value> {
        let iter_value = self.evaluate_expression(iterable)?;
        match iter_value {
            Value::List(items) => {
                let mut current_result_idx = 0i64;

                for item in items {
                    // Create new scope with loop variable
                    let scoped_eval = self.clone_with_var(variable, item);

                    // Check condition if present
                    let should_include = if let Some(cond) = condition {
                        let cond_val = scoped_eval.evaluate_expression(cond)?;
                        scoped_eval.is_truthy(&cond_val)
                    } else {
                        true
                    };

                    if should_include {
                        // Check if this is the target index
                        if current_result_idx == target_idx {
                            // Found the target - evaluate and return immediately
                            return scoped_eval.evaluate_expression(expr);
                        }
                        current_result_idx += 1;
                    }
                }

                // Index out of bounds
                Err(anyhow!("Index out of bounds: {}", target_idx))
            }
            _ => Err(anyhow!("List comprehension requires iterable to be a list")),
        }
    }

    /// Clone evaluator with additional variable binding (for scopes)
    fn clone_with_var(&self, var_name: &str, value: Value) -> Self {
        let mut new_eval = Self {
            variables: self.variables.clone(),
            functions: self.functions.clone(),
            lazy_vars: RefCell::new(self.lazy_vars.borrow().clone()),
            lazy_type_annotations: RefCell::new(self.lazy_type_annotations.borrow().clone()),
            evaluating: RefCell::new(HashSet::new()),
            current_file: RefCell::new(self.current_file.borrow().clone()),
            importing: RefCell::new(HashSet::new()),
            import_cache: RefCell::new(self.import_cache.borrow().clone()),
            trace_imports: self.trace_imports,
            import_metrics: RefCell::new(self.import_metrics.borrow().clone()),
            module_interface_cache: RefCell::new(self.module_interface_cache.borrow().clone()),
            module_output_cache: RefCell::new(self.module_output_cache.borrow().clone()),
            current_module_inputs: RefCell::new(self.current_module_inputs.borrow().clone()),
            instantiating_modules: RefCell::new(Vec::new()),
            module_source_resolver: RefCell::new(ModuleSourceResolver::new(None)),
        };
        new_eval.variables.insert(var_name.to_string(), value);
        new_eval
    }

    /// Recursively evaluate list comprehension with multiple for clauses
    /// e.g., [expr for x in list1 for y in list2 if cond]
    fn evaluate_comprehension_recursive(
        &self,
        expr: &Expression,
        iterators: &[(String, Expression)],
        condition: Option<&Expression>,
        depth: usize,
    ) -> Result<Value> {
        if depth >= iterators.len() {
            // Base case: all iterators exhausted, evaluate expression
            // But check condition first
            let should_include = if let Some(cond) = condition {
                let cond_val = self.evaluate_expression(cond)?;
                self.is_truthy(&cond_val)
            } else {
                true
            };

            if should_include {
                Ok(self.evaluate_expression(expr)?)
            } else {
                // Return a special marker for "skip this item"
                // We'll filter these out later
                Ok(Value::Null) // Using Null as a marker - we'll need to handle this differently
            }
        } else {
            // Recursive case: iterate over current iterator
            let (var_name, iterable_expr) = &iterators[depth];
            let iter_value = self.evaluate_expression(iterable_expr)?;

            match iter_value {
                Value::List(items) => {
                    let mut results = Vec::new();

                    for item in items {
                        // Create new scope with current loop variable(s)
                        // Support tuple destructuring: "i, x" binds to multiple variables
                        let scoped_eval = if var_name.contains(", ") {
                            // Tuple destructuring - extract elements from list
                            let var_names: Vec<&str> = var_name.split(", ").collect();

                            // Item must be a list with matching element count
                            let tuple_values = match &item {
                                Value::List(elements) => {
                                    if elements.len() != var_names.len() {
                                        return Err(anyhow!(
                                            "Tuple destructuring mismatch: expected {} elements, got {}",
                                            var_names.len(),
                                            elements.len()
                                        ));
                                    }
                                    elements.clone()
                                }
                                _ => {
                                    return Err(anyhow!(
                                        "Tuple destructuring requires a list with {} elements, got {:?}",
                                        var_names.len(),
                                        item
                                    ));
                                }
                            };

                            // Bind each variable to its corresponding value
                            // Start with first binding, then chain the rest
                            let (first_name, rest_names) = var_names.split_first().unwrap();
                            let (first_value, rest_values) = tuple_values.split_first().unwrap();

                            let mut eval = self.clone_with_var(first_name, first_value.clone());
                            for (name, value) in rest_names.iter().zip(rest_values.iter()) {
                                eval = eval.clone_with_var(name, value.clone());
                            }
                            eval
                        } else {
                            // Single variable binding
                            self.clone_with_var(var_name, item)
                        };

                        // Recursively handle remaining iterators
                        let sub_results = scoped_eval.evaluate_comprehension_recursive(
                            expr,
                            iterators,
                            condition,
                            depth + 1,
                        )?;

                        // Collect results
                        // If the next depth is the base case, we got a single value to push
                        // Otherwise, we got a list of results to extend (flatten)
                        if depth + 1 >= iterators.len() {
                            // Base case returned a single value (could be a List!)
                            if !matches!(sub_results, Value::Null) {
                                results.push(sub_results);
                            }
                        } else {
                            // Recursive case returned a list to flatten
                            match sub_results {
                                Value::List(items) => results.extend(items),
                                Value::Null => {} // Skip marker
                                other => results.push(other),
                            }
                        }
                    }

                    Ok(Value::List(results))
                }
                _ => Err(anyhow!(
                    "List comprehension requires iterable to be a list, got {:?}",
                    iter_value
                )),
            }
        }
    }

    /// Call a function (built-in or user-defined)
    fn call_function(&self, name: &str, args: &[Expression]) -> Result<Value> {
        // Handle higher-order functions (map, filter, reduce) specially
        // These need unevaluated arguments to work with lambdas
        match name {
            "map" => return self.call_map(args),
            "filter" => return self.call_filter(args),
            "reduce" => return self.call_reduce(args),
            _ => {}
        }

        // Evaluate all arguments
        let arg_values: Result<Vec<Value>> = args
            .iter()
            .map(|arg| self.evaluate_expression(arg))
            .collect();
        let arg_values = arg_values?;

        // Check if it's a user-defined function (in functions map)
        if let Some(func) = self.functions.get(name) {
            return self.call_user_function(func, &arg_values);
        }

        // Check if it's a lambda stored in a variable
        if let Some(func) = self.variables.get(name) {
            if matches!(func, Value::Function { .. }) {
                return self.call_user_function(func, &arg_values);
            }
        }

        // Call built-in function
        functions::call_builtin(name, arg_values)
    }

    /// Call a user-defined function
    fn call_user_function(&self, func: &Value, args: &[Value]) -> Result<Value> {
        match func {
            Value::Function { params, body } => {
                if args.len() != params.len() {
                    return Err(anyhow!(
                        "Function expects {} arguments, got {}",
                        params.len(),
                        args.len()
                    ));
                }

                // Create new scope with parameter bindings
                let mut scoped_eval = self.clone_with_var("_", Value::Null);
                for (param, arg) in params.iter().zip(args.iter()) {
                    scoped_eval
                        .variables
                        .insert(param.name.clone(), arg.clone());
                }

                scoped_eval.evaluate_expression(body)
            }
            _ => Err(anyhow!("Value is not a function")),
        }
    }

    /// Higher-order function: map(lambda, list)
    /// Applies the lambda to each element in the list and returns a new list
    fn call_map(&self, args: &[Expression]) -> Result<Value> {
        if args.len() != 2 {
            return Err(anyhow!(
                "map() expects 2 arguments (lambda, list), got {}",
                args.len()
            ));
        }

        // Evaluate the lambda/function
        let func_value = self.evaluate_expression(&args[0])?;
        if !matches!(func_value, Value::Function { .. }) {
            return Err(anyhow!("map() first argument must be a function"));
        }

        // Evaluate the list
        let list_value = self.evaluate_expression(&args[1])?;
        let list = match list_value {
            Value::List(l) => l,
            _ => return Err(anyhow!("map() second argument must be a list")),
        };

        // Apply the function to each element
        let mut results = Vec::new();
        for item in list {
            let result = self.call_user_function(&func_value, &[item])?;
            results.push(result);
        }

        Ok(Value::List(results))
    }

    /// Higher-order function: filter(lambda, list)
    /// Returns a new list containing only elements for which lambda returns true
    fn call_filter(&self, args: &[Expression]) -> Result<Value> {
        if args.len() != 2 {
            return Err(anyhow!(
                "filter() expects 2 arguments (lambda, list), got {}",
                args.len()
            ));
        }

        // Evaluate the lambda/function
        let func_value = self.evaluate_expression(&args[0])?;
        if !matches!(func_value, Value::Function { .. }) {
            return Err(anyhow!("filter() first argument must be a function"));
        }

        // Evaluate the list
        let list_value = self.evaluate_expression(&args[1])?;
        let list = match list_value {
            Value::List(l) => l,
            _ => return Err(anyhow!("filter() second argument must be a list")),
        };

        // Filter elements
        let mut results = Vec::new();
        for item in list {
            let result = self.call_user_function(&func_value, std::slice::from_ref(&item))?;
            if self.is_truthy(&result) {
                results.push(item);
            }
        }

        Ok(Value::List(results))
    }

    /// Higher-order function: reduce(lambda, list, initial)
    /// Reduces the list to a single value by repeatedly applying the lambda
    fn call_reduce(&self, args: &[Expression]) -> Result<Value> {
        if args.len() != 3 {
            return Err(anyhow!(
                "reduce() expects 3 arguments (lambda, list, initial), got {}",
                args.len()
            ));
        }

        // Evaluate the lambda/function
        let func_value = self.evaluate_expression(&args[0])?;
        if !matches!(func_value, Value::Function { .. }) {
            return Err(anyhow!("reduce() first argument must be a function"));
        }

        // Evaluate the list
        let list_value = self.evaluate_expression(&args[1])?;
        let list = match list_value {
            Value::List(l) => l,
            _ => return Err(anyhow!("reduce() second argument must be a list")),
        };

        // Evaluate the initial value
        let mut accumulator = self.evaluate_expression(&args[2])?;

        // Reduce the list
        for item in list {
            accumulator = self.call_user_function(&func_value, &[accumulator, item])?;
        }

        Ok(accumulator)
    }

    /// Check if an actual type matches an expected type
    fn type_matches(&self, actual: &crate::ast::Type, expected: &crate::ast::Type) -> bool {
        use crate::ast::Type;

        match (actual, expected) {
            // Any matches anything
            (_, Type::Any) | (Type::Any, _) => true,

            // Exact matches
            (Type::String, Type::String) => true,
            (Type::Int, Type::Int) => true,
            (Type::Float, Type::Float) => true,
            (Type::Bool, Type::Bool) => true,
            (Type::Null, Type::Null) => true,

            // Int can be used as Float
            (Type::Int, Type::Float) => true,

            // List types must have compatible element types
            (Type::List(actual_elem), Type::List(expected_elem)) => {
                self.type_matches(actual_elem, expected_elem)
            }

            // Map types must have compatible key and value types
            (Type::Map(actual_key, actual_val), Type::Map(expected_key, expected_val)) => {
                self.type_matches(actual_key, expected_key)
                    && self.type_matches(actual_val, expected_val)
            }

            // Function types must have compatible signatures
            (
                Type::Function {
                    params: actual_params,
                    return_type: actual_return,
                },
                Type::Function {
                    params: expected_params,
                    return_type: expected_return,
                },
            ) => {
                actual_params.len() == expected_params.len()
                    && actual_params
                        .iter()
                        .zip(expected_params.iter())
                        .all(|(a, e)| self.type_matches(a, e))
                    && self.type_matches(actual_return, expected_return)
            }

            // Everything else doesn't match
            _ => false,
        }
    }

    /// Evaluate a lazy variable on first access with cycle detection
    /// Returns the evaluated value and caches it for future access
    fn evaluate_lazy_var(&self, name: &str) -> Result<Value> {
        // Check if already in the process of evaluating (cycle detection)
        if self.evaluating.borrow().contains(name) {
            return Err(anyhow!(
                "Circular dependency detected while evaluating variable '{}'",
                name
            ));
        }

        // Check if we have a lazy expression for this variable
        let expr = self
            .lazy_vars
            .borrow()
            .get(name)
            .cloned()
            .ok_or_else(|| anyhow!("Undefined variable: {}", name))?;

        // Get type annotation if present
        let type_annotation = self.lazy_type_annotations.borrow().get(name).cloned();

        // Mark as currently evaluating
        self.evaluating.borrow_mut().insert(name.to_string());

        // Evaluate the expression
        let result = self.evaluate_expression(&expr);

        // Remove from evaluating set
        self.evaluating.borrow_mut().remove(name);

        let value = result?;

        // Validate type annotation if present
        if let Some(expected_type) = type_annotation {
            let actual_type = value.get_type();
            if !self.type_matches(&actual_type, &expected_type) {
                return Err(anyhow!(
                    "Type mismatch for variable '{}': expected {}, got {}",
                    name,
                    expected_type,
                    actual_type
                ));
            }
        }

        Ok(value)
    }

    /// Evaluate an import statement
    fn evaluate_import(&mut self, path: &str, kind: &ImportKind) -> Result<()> {
        use std::time::Instant;

        let start = Instant::now();

        // Resolve the import path relative to the current file
        let resolved_path = self.resolve_import_path(path)?;

        // Check for circular imports
        if self.importing.borrow().contains(&resolved_path) {
            return Err(anyhow!(
                "Circular import detected: {}",
                resolved_path.display()
            ));
        }

        // Check if we've already imported this module (use cache)
        let cached_bindings = self.import_cache.borrow().get(&resolved_path).cloned();

        let is_cached = cached_bindings.is_some();

        // Trace import if enabled
        if self.trace_imports {
            let kind_str = match kind {
                ImportKind::Full { alias: Some(a) } => format!("Full (as {})", a),
                ImportKind::Full { alias: None } => "Full".to_string(),
                ImportKind::Selective { items } => {
                    format!("Selective ({} items)", items.len())
                }
                ImportKind::Wildcard => "Wildcard".to_string(),
            };

            println!(
                "[IMPORT] {} from {}{}",
                kind_str,
                resolved_path.display(),
                if is_cached { " [CACHED]" } else { "" }
            );
        }

        let imported_bindings = if let Some(cached) = cached_bindings {
            cached
        } else {
            // Mark as currently importing
            self.importing.borrow_mut().insert(resolved_path.clone());

            // Save the current file
            let previous_file = self.current_file.borrow().clone();

            // Set the current file to the import path for nested imports
            self.set_current_file(&resolved_path);

            // Parse and evaluate the imported module
            let imported_module = crate::parse_file(&resolved_path).map_err(|e| {
                anyhow!(
                    "Failed to parse imported file '{}': {}",
                    resolved_path.display(),
                    e
                )
            })?;

            let evaluated = self.evaluate(imported_module).map_err(|e| {
                anyhow!(
                    "Failed to evaluate imported file '{}': {}",
                    resolved_path.display(),
                    e
                )
            })?;

            // Restore the previous file
            *self.current_file.borrow_mut() = previous_file;

            // Remove from importing set
            self.importing.borrow_mut().remove(&resolved_path);

            // Cache the result
            self.import_cache
                .borrow_mut()
                .insert(resolved_path.clone(), evaluated.bindings.clone());

            evaluated.bindings
        };

        // Record metrics
        let duration = start.elapsed().as_millis();
        let mut metrics = self.import_metrics.borrow_mut();
        metrics.total_imports += 1;
        if is_cached {
            metrics.cache_hits += 1;
        }
        metrics.total_time_ms += duration;

        let kind_str = match kind {
            ImportKind::Full { alias: Some(a) } => format!("import \"{}\" as {}", path, a),
            ImportKind::Full { alias: None } => format!("import \"{}\"", path),
            ImportKind::Selective { items } => {
                format!("import ({}) from \"{}\"", items.len(), path)
            }
            ImportKind::Wildcard => format!("import * from \"{}\"", path),
        };

        metrics.traces.push(ImportTrace {
            importer: self.current_file.borrow().clone(),
            imported: resolved_path.clone(),
            kind: kind_str,
            cached: is_cached,
            duration_ms: duration,
        });

        // Add imported bindings to the current scope based on import kind
        match kind {
            ImportKind::Full { alias } => {
                if let Some(alias_name) = alias {
                    // Create a namespace map with all imports
                    let namespace_map: HashMap<String, Value> =
                        imported_bindings.into_iter().collect();
                    self.variables
                        .insert(alias_name.clone(), Value::Map(namespace_map));
                } else {
                    // Import all bindings directly into current scope
                    for (name, value) in imported_bindings {
                        self.variables.insert(name, value);
                    }
                }
            }
            ImportKind::Selective { items } => {
                for item in items {
                    let imported_value = imported_bindings.get(&item.name).ok_or_else(|| {
                        anyhow!(
                            "Item '{}' not found in imported module '{}'",
                            item.name,
                            path
                        )
                    })?;

                    let local_name = item.alias.as_ref().unwrap_or(&item.name);
                    self.variables
                        .insert(local_name.clone(), imported_value.clone());
                }
            }
            ImportKind::Wildcard => {
                // Import all bindings directly into current scope
                for (name, value) in imported_bindings {
                    self.variables.insert(name, value);
                }
            }
        }

        Ok(())
    }

    /// Evaluate a module instance with input parameters
    fn evaluate_module_instance(
        &mut self,
        source: &str,
        input_exprs: &HashMap<String, Expression>,
    ) -> Result<HashMap<String, Value>> {
        // Resolve the module path relative to the current file
        let resolved_path = self.resolve_import_path(source)?;

        // Check for circular module dependencies
        {
            let instantiating = self.instantiating_modules.borrow();
            if instantiating.contains(&resolved_path) {
                let cycle: Vec<String> = instantiating
                    .iter()
                    .chain(std::iter::once(&resolved_path))
                    .map(|p| p.display().to_string())
                    .collect();
                return Err(anyhow!(
                    "Circular module dependency detected: {}",
                    cycle.join(" -> ")
                ));
            }
        }

        // Add this module to the instantiation stack
        self.instantiating_modules
            .borrow_mut()
            .push(resolved_path.clone());

        // Call helper function and ensure we pop from stack regardless of result
        let result = self.evaluate_module_instance_impl(&resolved_path, input_exprs);

        // Remove from stack before returning
        self.instantiating_modules.borrow_mut().pop();

        result
    }

    /// Helper function for module instance evaluation (actual implementation)
    fn evaluate_module_instance_impl(
        &mut self,
        resolved_path: &Path,
        input_exprs: &HashMap<String, Expression>,
    ) -> Result<HashMap<String, Value>> {
        // Parse the module file
        let module_ast = crate::parse_file(resolved_path).map_err(|e| {
            anyhow!(
                "Failed to parse module file '{}': {}",
                resolved_path.display(),
                e
            )
        })?;

        // Save the current context (file only - module inputs handled separately)
        let previous_file = self.current_file.borrow().clone();

        // Set the current file to the module path for nested imports
        self.set_current_file(resolved_path);

        // Evaluate input expressions in the CALLER's context
        let mut input_values = HashMap::new();
        for (name, expr) in input_exprs {
            // Restore caller file context for input evaluation
            // Keep module inputs context so nested modules can use module.inputs
            *self.current_file.borrow_mut() = previous_file.clone();
            // DO NOT restore module inputs here - keep the current module's inputs available

            let value = self.evaluate_expression(expr)?;
            input_values.insert(name.clone(), value);

            // Restore module file context
            self.set_current_file(resolved_path);
        }

        // Clear any previous module output cache for this path
        // (since we're evaluating with different inputs)
        self.module_output_cache.borrow_mut().remove(resolved_path);

        // Get the module interface for validation and default values
        let interface = self
            .module_interface_cache
            .borrow()
            .get(resolved_path)
            .cloned();

        // Apply default values for missing inputs
        let mut final_input_values = input_values.clone();
        if let Some(ref iface) = interface {
            for (name, input_def) in &iface.inputs {
                if !final_input_values.contains_key(name) {
                    if let Some(ref default_expr) = input_def.default {
                        // Evaluate default expression in caller context
                        *self.current_file.borrow_mut() = previous_file.clone();
                        let default_value = self.evaluate_expression(default_expr)?;
                        final_input_values.insert(name.clone(), default_value);
                        self.set_current_file(resolved_path);
                    }
                }
            }
        }

        // Validate inputs against the interface if it exists
        if let Some(ref iface) = interface {
            self.validate_module_inputs(&final_input_values, &iface.inputs)?;
        }

        // Save current variable state to isolate module evaluation
        let saved_variables = self.variables.clone();
        let saved_functions = self.functions.clone();
        let saved_module_inputs = self.current_module_inputs.borrow().clone();

        // Set module.inputs for the module context
        *self.current_module_inputs.borrow_mut() = Some(final_input_values.clone());

        // Evaluate the module
        let _evaluated = self.evaluate(module_ast).map_err(|e| {
            anyhow!(
                "Failed to evaluate module file '{}': {}",
                resolved_path.display(),
                e
            )
        })?;

        // Restore module.inputs after evaluation (for nested modules)
        *self.current_module_inputs.borrow_mut() = saved_module_inputs;

        // Restore the previous file
        *self.current_file.borrow_mut() = previous_file;

        // Restore variable state (isolate module variables from parent scope)
        self.variables = saved_variables;
        self.functions = saved_functions;

        // Get the module interface for validation
        let interface = self
            .module_interface_cache
            .borrow()
            .get(resolved_path)
            .cloned();

        // Validate inputs against the interface if it exists
        if let Some(ref iface) = interface {
            self.validate_module_inputs(&input_values, &iface.inputs)?;
        }

        // Get the module outputs
        let outputs = self
            .module_output_cache
            .borrow()
            .get(resolved_path)
            .map(|o| o.outputs.clone())
            .ok_or_else(|| {
                anyhow!(
                    "Module '{}' did not define module.outputs",
                    resolved_path.display()
                )
            })?;

        // Validate outputs against the interface if it exists
        if let Some(ref iface) = interface {
            self.validate_module_outputs(&outputs, &iface.outputs)?;
        }

        Ok(outputs)
    }

    /// Evaluate module with count meta-argument (create N instances)
    fn evaluate_module_count(
        &mut self,
        module_type: &str,
        instance_name: &str,
        source: &str,
        count_expr: &Expression,
        input_exprs: &HashMap<String, Expression>,
        bindings: &mut HashMap<String, Value>,
    ) -> Result<()> {
        // Evaluate count expression
        let count_value = self.evaluate_expression(count_expr)?;
        let count = match count_value {
            Value::Int(n) if n >= 0 => n as usize,
            _ => {
                return Err(anyhow!(
                    "Module 'count' must be a non-negative integer, got {:?}",
                    count_value
                ));
            }
        };

        // Create count instances (stored as a list)
        let mut instances = Vec::new();
        for i in 0..count {
            // Make count.index available during evaluation
            let mut count_map = HashMap::new();
            count_map.insert("index".to_string(), Value::Int(i as i64));
            self.variables
                .insert("count".to_string(), Value::Map(count_map));

            // Evaluate the module instance
            let outputs = self.evaluate_module_instance(source, input_exprs)?;
            instances.push(Value::Map(outputs));

            // Remove count variable
            self.variables.remove("count");
        }

        // Store instances as a list: module.<type>.<instance> = [...]
        let module_map = if let Some(Value::Map(m)) = self.variables.get("module") {
            m.clone()
        } else {
            HashMap::new()
        };

        let mut module_map = module_map;
        let type_map = if let Some(Value::Map(m)) = module_map.get(module_type) {
            m.clone()
        } else {
            HashMap::new()
        };

        let mut type_map = type_map;
        type_map.insert(instance_name.to_string(), Value::List(instances));

        module_map.insert(module_type.to_string(), Value::Map(type_map));

        self.variables
            .insert("module".to_string(), Value::Map(module_map.clone()));
        bindings.insert("module".to_string(), Value::Map(module_map));

        Ok(())
    }

    /// Evaluate module with for_each meta-argument (create instances for each element)
    fn evaluate_module_for_each(
        &mut self,
        module_type: &str,
        instance_name: &str,
        source: &str,
        for_each_expr: &Expression,
        input_exprs: &HashMap<String, Expression>,
        bindings: &mut HashMap<String, Value>,
    ) -> Result<()> {
        // Evaluate for_each expression
        let for_each_value = self.evaluate_expression(for_each_expr)?;

        match for_each_value {
            Value::List(list) => {
                // For lists, create instances with each.value and each.key (index)
                let mut instances = HashMap::new();
                for (index, value) in list.iter().enumerate() {
                    // Make each.key and each.value available
                    let mut each_map = HashMap::new();
                    each_map.insert("key".to_string(), Value::Int(index as i64));
                    each_map.insert("value".to_string(), value.clone());
                    self.variables
                        .insert("each".to_string(), Value::Map(each_map));

                    // Evaluate the module instance
                    let outputs = self.evaluate_module_instance(source, input_exprs)?;
                    instances.insert(index.to_string(), Value::Map(outputs));

                    // Remove each variable
                    self.variables.remove("each");
                }

                // Store instances as a map: module.<type>.<instance> = {...}
                let module_map = if let Some(Value::Map(m)) = self.variables.get("module") {
                    m.clone()
                } else {
                    HashMap::new()
                };

                let mut module_map = module_map;
                let type_map = if let Some(Value::Map(m)) = module_map.get(module_type) {
                    m.clone()
                } else {
                    HashMap::new()
                };

                let mut type_map = type_map;
                type_map.insert(instance_name.to_string(), Value::Map(instances));

                module_map.insert(module_type.to_string(), Value::Map(type_map));

                self.variables
                    .insert("module".to_string(), Value::Map(module_map.clone()));
                bindings.insert("module".to_string(), Value::Map(module_map));

                Ok(())
            }
            Value::Map(map) => {
                // For maps, create instances with each.key and each.value
                let mut instances = HashMap::new();
                for (key, value) in map.iter() {
                    // Make each.key and each.value available
                    let mut each_map = HashMap::new();
                    each_map.insert("key".to_string(), Value::String(key.clone()));
                    each_map.insert("value".to_string(), value.clone());
                    self.variables
                        .insert("each".to_string(), Value::Map(each_map));

                    // Evaluate the module instance
                    let outputs = self.evaluate_module_instance(source, input_exprs)?;
                    instances.insert(key.clone(), Value::Map(outputs));

                    // Remove each variable
                    self.variables.remove("each");
                }

                // Store instances as a map: module.<type>.<instance> = {...}
                let module_map = if let Some(Value::Map(m)) = self.variables.get("module") {
                    m.clone()
                } else {
                    HashMap::new()
                };

                let mut module_map = module_map;
                let type_map = if let Some(Value::Map(m)) = module_map.get(module_type) {
                    m.clone()
                } else {
                    HashMap::new()
                };

                let mut type_map = type_map;
                type_map.insert(instance_name.to_string(), Value::Map(instances));

                module_map.insert(module_type.to_string(), Value::Map(type_map));

                self.variables
                    .insert("module".to_string(), Value::Map(module_map.clone()));
                bindings.insert("module".to_string(), Value::Map(module_map));

                Ok(())
            }
            _ => Err(anyhow!(
                "Module 'for_each' must be a list or map, got {:?}",
                for_each_value
            )),
        }
    }

    /// Validate module inputs against the interface
    fn validate_module_inputs(
        &self,
        provided: &HashMap<String, Value>,
        interface: &HashMap<String, crate::ast::ModuleInput>,
    ) -> Result<()> {
        // Check for required inputs
        for (name, input_def) in interface {
            if input_def.required && !provided.contains_key(name) && input_def.default.is_none() {
                return Err(anyhow!("Required module input '{}' not provided", name));
            }
        }

        // Check for unknown inputs
        for name in provided.keys() {
            if !interface.contains_key(name) {
                return Err(anyhow!("Unknown module input '{}'", name));
            }
        }

        Ok(())
    }

    /// Validate module outputs against the interface
    fn validate_module_outputs(
        &self,
        provided: &HashMap<String, Value>,
        interface: &HashMap<String, crate::ast::ModuleOutput>,
    ) -> Result<()> {
        // Check that all declared outputs are provided
        for name in interface.keys() {
            if !provided.contains_key(name) {
                return Err(anyhow!(
                    "Module output '{}' declared but not provided",
                    name
                ));
            }
        }

        Ok(())
    }

    /// Resolve an import path relative to the current file
    fn resolve_import_path(&self, path: &str) -> Result<PathBuf> {
        // Check if this is an external source (registry, git, http, tarball)
        if path.starts_with("registry::")
            || path.starts_with("git::")
            || path.starts_with("http://")
            || path.starts_with("https://")
        {
            // Use module source resolver for external sources
            let base_dir = if let Some(current) = self.current_file.borrow().clone() {
                current
                    .parent()
                    .map(|p| p.to_path_buf())
                    .ok_or_else(|| anyhow!("Current file has no parent directory"))?
            } else {
                PathBuf::from(".")
            };

            return self
                .module_source_resolver
                .borrow_mut()
                .resolve(path, &base_dir);
        }

        // Local path resolution (existing logic)
        let import_path = Path::new(path);

        // If it's an absolute path, use it directly
        if import_path.is_absolute() {
            return Ok(import_path.to_path_buf());
        }

        // Otherwise, resolve relative to the current file
        if let Some(current) = self.current_file.borrow().as_ref() {
            let base_dir = current
                .parent()
                .ok_or_else(|| anyhow!("Current file has no parent directory"))?;
            let resolved = base_dir.join(import_path);

            // Canonicalize to resolve ".." and "." components
            resolved.canonicalize().or_else(|_| {
                // If canonicalize fails (file might not exist yet), just return the joined path
                Ok(resolved)
            })
        } else {
            // No current file, try to resolve from current working directory
            let cwd = std::env::current_dir()?;
            let resolved = cwd.join(import_path);

            resolved.canonicalize().or_else(|_| Ok(resolved))
        }
    }

    /// Register built-in functions
    fn register_builtins(&mut self) {
        // Built-in functions are implemented in the functions module
        // and called via functions::call_builtin()
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_literal() {
        let evaluator = Evaluator::new();
        let expr = Expression::Literal {
            value: Value::String("hello".to_string()),
            span: None,
        };
        let result = evaluator.evaluate_expression(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::String("hello".to_string()));
    }

    #[test]
    fn test_evaluate_arithmetic() {
        let evaluator = Evaluator::new();

        // Addition
        let expr = Expression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(Expression::Literal {
                value: Value::Int(5),
                span: None,
            }),
            right: Box::new(Expression::Literal {
                value: Value::Int(3),
                span: None,
            }),
            span: None,
        };
        assert_eq!(evaluator.evaluate_expression(&expr).unwrap(), Value::Int(8));

        // Subtraction
        let expr = Expression::BinaryOp {
            op: BinaryOperator::Subtract,
            left: Box::new(Expression::Literal {
                value: Value::Int(10),
                span: None,
            }),
            right: Box::new(Expression::Literal {
                value: Value::Int(4),
                span: None,
            }),
            span: None,
        };
        assert_eq!(evaluator.evaluate_expression(&expr).unwrap(), Value::Int(6));

        // Multiplication
        let expr = Expression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(Expression::Literal {
                value: Value::Int(6),
                span: None,
            }),
            right: Box::new(Expression::Literal {
                value: Value::Int(7),
                span: None,
            }),
            span: None,
        };
        assert_eq!(
            evaluator.evaluate_expression(&expr).unwrap(),
            Value::Int(42)
        );

        // Division
        let expr = Expression::BinaryOp {
            op: BinaryOperator::Divide,
            left: Box::new(Expression::Literal {
                value: Value::Int(20),
                span: None,
            }),
            right: Box::new(Expression::Literal {
                value: Value::Int(4),
                span: None,
            }),
            span: None,
        };
        assert_eq!(evaluator.evaluate_expression(&expr).unwrap(), Value::Int(5));
    }

    #[test]
    fn test_evaluate_comparison() {
        let evaluator = Evaluator::new();

        // Equal
        let expr = Expression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(Expression::Literal {
                value: Value::Int(5),
                span: None,
            }),
            right: Box::new(Expression::Literal {
                value: Value::Int(5),
                span: None,
            }),
            span: None,
        };
        assert_eq!(
            evaluator.evaluate_expression(&expr).unwrap(),
            Value::Bool(true)
        );

        // Not equal
        let expr = Expression::BinaryOp {
            op: BinaryOperator::NotEqual,
            left: Box::new(Expression::Literal {
                value: Value::Int(5),
                span: None,
            }),
            right: Box::new(Expression::Literal {
                value: Value::Int(3),
                span: None,
            }),
            span: None,
        };
        assert_eq!(
            evaluator.evaluate_expression(&expr).unwrap(),
            Value::Bool(true)
        );

        // Less than
        let expr = Expression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(Expression::Literal {
                value: Value::Int(3),
                span: None,
            }),
            right: Box::new(Expression::Literal {
                value: Value::Int(5),
                span: None,
            }),
            span: None,
        };
        assert_eq!(
            evaluator.evaluate_expression(&expr).unwrap(),
            Value::Bool(true)
        );

        // Greater than
        let expr = Expression::BinaryOp {
            op: BinaryOperator::GreaterThan,
            left: Box::new(Expression::Literal {
                value: Value::Int(7),
                span: None,
            }),
            right: Box::new(Expression::Literal {
                value: Value::Int(3),
                span: None,
            }),
            span: None,
        };
        assert_eq!(
            evaluator.evaluate_expression(&expr).unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn test_evaluate_logical() {
        let evaluator = Evaluator::new();

        // AND - true and true
        let expr = Expression::BinaryOp {
            op: BinaryOperator::And,
            left: Box::new(Expression::Literal {
                value: Value::Bool(true),
                span: None,
            }),
            right: Box::new(Expression::Literal {
                value: Value::Bool(true),
                span: None,
            }),
            span: None,
        };
        assert_eq!(
            evaluator.evaluate_expression(&expr).unwrap(),
            Value::Bool(true)
        );

        // AND - false and true
        let expr = Expression::BinaryOp {
            op: BinaryOperator::And,
            left: Box::new(Expression::Literal {
                value: Value::Bool(false),
                span: None,
            }),
            right: Box::new(Expression::Literal {
                value: Value::Bool(true),
                span: None,
            }),
            span: None,
        };
        assert_eq!(
            evaluator.evaluate_expression(&expr).unwrap(),
            Value::Bool(false)
        );

        // OR - false or true
        let expr = Expression::BinaryOp {
            op: BinaryOperator::Or,
            left: Box::new(Expression::Literal {
                value: Value::Bool(false),
                span: None,
            }),
            right: Box::new(Expression::Literal {
                value: Value::Bool(true),
                span: None,
            }),
            span: None,
        };
        assert_eq!(
            evaluator.evaluate_expression(&expr).unwrap(),
            Value::Bool(true)
        );

        // NOT
        let expr = Expression::UnaryOp {
            op: UnaryOperator::Not,
            operand: Box::new(Expression::Literal {
                value: Value::Bool(true),
                span: None,
            }),
            span: None,
        };
        assert_eq!(
            evaluator.evaluate_expression(&expr).unwrap(),
            Value::Bool(false)
        );
    }

    #[test]
    fn test_evaluate_null_coalesce() {
        let evaluator = Evaluator::new();

        // Null ?? value
        let expr = Expression::BinaryOp {
            op: BinaryOperator::NullCoalesce,
            left: Box::new(Expression::Literal {
                value: Value::Null,
                span: None,
            }),
            right: Box::new(Expression::Literal {
                value: Value::Int(42),
                span: None,
            }),
            span: None,
        };
        assert_eq!(
            evaluator.evaluate_expression(&expr).unwrap(),
            Value::Int(42)
        );

        // value ?? other
        let expr = Expression::BinaryOp {
            op: BinaryOperator::NullCoalesce,
            left: Box::new(Expression::Literal {
                value: Value::Int(10),
                span: None,
            }),
            right: Box::new(Expression::Literal {
                value: Value::Int(42),
                span: None,
            }),
            span: None,
        };
        assert_eq!(
            evaluator.evaluate_expression(&expr).unwrap(),
            Value::Int(10)
        );
    }

    #[test]
    fn test_evaluate_ternary() {
        let evaluator = Evaluator::new();

        // true ? "yes" : "no"
        let expr = Expression::Ternary {
            condition: Box::new(Expression::Literal {
                value: Value::Bool(true),
                span: None,
            }),
            then_expr: Box::new(Expression::Literal {
                value: Value::String("yes".to_string()),
                span: None,
            }),
            else_expr: Box::new(Expression::Literal {
                value: Value::String("no".to_string()),
                span: None,
            }),
            span: None,
        };
        assert_eq!(
            evaluator.evaluate_expression(&expr).unwrap(),
            Value::String("yes".to_string())
        );

        // false ? "yes" : "no"
        let expr = Expression::Ternary {
            condition: Box::new(Expression::Literal {
                value: Value::Bool(false),
                span: None,
            }),
            then_expr: Box::new(Expression::Literal {
                value: Value::String("yes".to_string()),
                span: None,
            }),
            else_expr: Box::new(Expression::Literal {
                value: Value::String("no".to_string()),
                span: None,
            }),
            span: None,
        };
        assert_eq!(
            evaluator.evaluate_expression(&expr).unwrap(),
            Value::String("no".to_string())
        );
    }

    #[test]
    fn test_evaluate_if_expression() {
        let evaluator = Evaluator::new();

        // if true then "yes" else "no"
        let expr = Expression::If {
            condition: Box::new(Expression::Literal {
                value: Value::Bool(true),
                span: None,
            }),
            then_expr: Box::new(Expression::Literal {
                value: Value::String("yes".to_string()),
                span: None,
            }),
            else_expr: Some(Box::new(Expression::Literal {
                value: Value::String("no".to_string()),
                span: None,
            })),
            span: None,
        };
        assert_eq!(
            evaluator.evaluate_expression(&expr).unwrap(),
            Value::String("yes".to_string())
        );

        // if false then "yes" (no else)
        let expr = Expression::If {
            condition: Box::new(Expression::Literal {
                value: Value::Bool(false),
                span: None,
            }),
            then_expr: Box::new(Expression::Literal {
                value: Value::String("yes".to_string()),
                span: None,
            }),
            else_expr: None,
            span: None,
        };
        assert_eq!(evaluator.evaluate_expression(&expr).unwrap(), Value::Null);
    }

    #[test]
    fn test_evaluate_list() {
        let evaluator = Evaluator::new();

        let expr = Expression::List {
            elements: vec![
                Expression::Literal {
                    value: Value::Int(1),
                    span: None,
                },
                Expression::Literal {
                    value: Value::Int(2),
                    span: None,
                },
                Expression::Literal {
                    value: Value::Int(3),
                    span: None,
                },
            ],
            span: None,
        };

        let result = evaluator.evaluate_expression(&expr).unwrap();
        assert_eq!(
            result,
            Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
        );
    }

    #[test]
    fn test_evaluate_map() {
        let evaluator = Evaluator::new();

        let entries = vec![
            (
                "name".to_string(),
                Expression::Literal {
                    value: Value::String("Alice".to_string()),
                    span: None,
                },
            ),
            (
                "age".to_string(),
                Expression::Literal {
                    value: Value::Int(30),
                    span: None,
                },
            ),
        ];

        let expr = Expression::Map {
            entries,
            span: None,
        };
        let result = evaluator.evaluate_expression(&expr).unwrap();

        if let Value::Map(map) = result {
            assert_eq!(map.get("name"), Some(&Value::String("Alice".to_string())));
            assert_eq!(map.get("age"), Some(&Value::Int(30)));
        } else {
            panic!("Expected Map value");
        }
    }

    #[test]
    fn test_evaluate_member_access() {
        let mut evaluator = Evaluator::new();

        // Create a map variable
        let mut map = HashMap::new();
        map.insert("name".to_string(), Value::String("Bob".to_string()));
        evaluator
            .variables
            .insert("person".to_string(), Value::Map(map));

        // person.name
        let expr = Expression::MemberAccess {
            object: Box::new(Expression::Variable {
                name: "person".to_string(),
                span: None,
            }),
            field: "name".to_string(),
            span: None,
        };

        assert_eq!(
            evaluator.evaluate_expression(&expr).unwrap(),
            Value::String("Bob".to_string())
        );
    }

    #[test]
    fn test_evaluate_index_access() {
        let mut evaluator = Evaluator::new();

        // Create a list variable
        let list = Value::List(vec![Value::Int(10), Value::Int(20), Value::Int(30)]);
        evaluator.variables.insert("numbers".to_string(), list);

        // numbers[1]
        let expr = Expression::Index {
            object: Box::new(Expression::Variable {
                name: "numbers".to_string(),
                span: None,
            }),
            index: Box::new(Expression::Literal {
                value: Value::Int(1),
                span: None,
            }),
            span: None,
        };

        assert_eq!(
            evaluator.evaluate_expression(&expr).unwrap(),
            Value::Int(20)
        );
    }

    #[test]
    fn test_evaluate_string_interpolation() {
        let mut evaluator = Evaluator::new();

        evaluator
            .variables
            .insert("name".to_string(), Value::String("World".to_string()));
        evaluator
            .variables
            .insert("count".to_string(), Value::Int(42));

        // "Hello ${name}, count: ${count}"
        let expr = Expression::InterpolatedString {
            parts: vec![
                StringPart::Literal("Hello ".to_string()),
                StringPart::Interpolation(Box::new(Expression::Variable {
                    name: "name".to_string(),
                    span: None,
                })),
                StringPart::Literal(", count: ".to_string()),
                StringPart::Interpolation(Box::new(Expression::Variable {
                    name: "count".to_string(),
                    span: None,
                })),
            ],
            span: None,
        };

        assert_eq!(
            evaluator.evaluate_expression(&expr).unwrap(),
            Value::String("Hello World, count: 42".to_string())
        );
    }

    #[test]
    fn test_evaluate_list_comprehension() {
        let mut evaluator = Evaluator::new();

        // Set up list [1, 2, 3, 4, 5]
        evaluator.variables.insert(
            "numbers".to_string(),
            Value::List(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
                Value::Int(4),
                Value::Int(5),
            ]),
        );

        // [x * 2 for x in numbers if x > 2]
        let expr = Expression::ListComprehension {
            expr: Box::new(Expression::BinaryOp {
                op: BinaryOperator::Multiply,
                left: Box::new(Expression::Variable {
                    name: "x".to_string(),
                    span: None,
                }),
                right: Box::new(Expression::Literal {
                    value: Value::Int(2),
                    span: None,
                }),
                span: None,
            }),
            iterators: vec![(
                "x".to_string(),
                Expression::Variable {
                    name: "numbers".to_string(),
                    span: None,
                },
            )],
            condition: Some(Box::new(Expression::BinaryOp {
                op: BinaryOperator::GreaterThan,
                left: Box::new(Expression::Variable {
                    name: "x".to_string(),
                    span: None,
                }),
                right: Box::new(Expression::Literal {
                    value: Value::Int(2),
                    span: None,
                }),
                span: None,
            })),
            span: None,
        };

        let result = evaluator.evaluate_expression(&expr).unwrap();
        assert_eq!(
            result,
            Value::List(vec![Value::Int(6), Value::Int(8), Value::Int(10)])
        );
    }

    #[test]
    fn test_evaluate_try_expression() {
        let evaluator = Evaluator::new();

        // try undefined_var else 42
        let expr = Expression::Try {
            expr: Box::new(Expression::Variable {
                name: "undefined".to_string(),
                span: None,
            }),
            default: Some(Box::new(Expression::Literal {
                value: Value::Int(42),
                span: None,
            })),
            span: None,
        };

        assert_eq!(
            evaluator.evaluate_expression(&expr).unwrap(),
            Value::Int(42)
        );

        // try valid_literal
        let expr = Expression::Try {
            expr: Box::new(Expression::Literal {
                value: Value::Int(100),
                span: None,
            }),
            default: None,
            span: None,
        };

        assert_eq!(
            evaluator.evaluate_expression(&expr).unwrap(),
            Value::Int(100)
        );
    }

    #[test]
    fn test_map_function() {
        let input = r#"
            numbers = [1, 2, 3, 4, 5]
            doubled = map(x => x * 2, numbers)
        "#;
        let module = crate::parse_str(input).unwrap();
        let mut evaluator = Evaluator::new();
        let result = evaluator.evaluate(module).unwrap();

        assert_eq!(
            result.bindings.get("doubled").unwrap(),
            &Value::List(vec![
                Value::Int(2),
                Value::Int(4),
                Value::Int(6),
                Value::Int(8),
                Value::Int(10),
            ])
        );
    }

    #[test]
    fn test_filter_function() {
        let input = r#"
            numbers = [1, 2, 3, 4, 5, 6]
            evens = filter(x => x % 2 == 0, numbers)
        "#;
        let module = crate::parse_str(input).unwrap();
        let mut evaluator = Evaluator::new();
        let result = evaluator.evaluate(module).unwrap();

        assert_eq!(
            result.bindings.get("evens").unwrap(),
            &Value::List(vec![Value::Int(2), Value::Int(4), Value::Int(6),])
        );
    }

    #[test]
    fn test_reduce_function() {
        let input = r#"
            numbers = [1, 2, 3, 4, 5]
            sum = reduce((acc, x) => acc + x, numbers, 0)
        "#;
        let module = crate::parse_str(input).unwrap();
        let mut evaluator = Evaluator::new();
        let result = evaluator.evaluate(module).unwrap();

        assert_eq!(result.bindings.get("sum").unwrap(), &Value::Int(15));
    }

    #[test]
    fn test_higher_order_with_lambda_variables() {
        let input = r#"
            double = x => x * 2
            numbers = [1, 2, 3]
            doubled = map(double, numbers)
        "#;
        let module = crate::parse_str(input).unwrap();
        let mut evaluator = Evaluator::new();
        let result = evaluator.evaluate(module).unwrap();

        assert_eq!(
            result.bindings.get("doubled").unwrap(),
            &Value::List(vec![Value::Int(2), Value::Int(4), Value::Int(6),])
        );
    }

    #[test]
    fn test_lambda_variable_call() {
        let input = r#"
            double = x => x * 2
            result = double(5)
        "#;
        let module = crate::parse_str(input).unwrap();
        let mut evaluator = Evaluator::new();
        let result = evaluator.evaluate(module).unwrap();

        assert_eq!(result.bindings.get("result").unwrap(), &Value::Int(10));
    }

    #[test]
    fn test_type_validation_success() {
        let input = r#"
            name: string = "John"
            age: int = 25
            price: float = 19.99
        "#;
        let module = crate::parse_str(input).unwrap();
        let mut evaluator = Evaluator::new();
        let result = evaluator.evaluate(module);

        assert!(result.is_ok());
    }

    #[test]
    fn test_type_validation_failure() {
        let input = r#"
            count: string = 42
        "#;
        let module = crate::parse_str(input).unwrap();
        let mut evaluator = Evaluator::new();
        let result = evaluator.evaluate(module);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Type mismatch"));
    }

    #[test]
    fn test_type_validation_int_to_float() {
        let input = r#"
            price: float = 42
        "#;
        let module = crate::parse_str(input).unwrap();
        let mut evaluator = Evaluator::new();
        let result = evaluator.evaluate(module);

        // Int should be allowed where Float is expected
        assert!(result.is_ok());
    }

    // Phase 2 Lazy Evaluation Tests

    #[test]
    fn test_lazy_variable_evaluation() {
        // Test that variables are evaluated lazily (on first access)
        let input = r#"
            x = 1 + 1
            y = x + 1
            z = y * 2
        "#;
        let module = crate::parse_str(input).unwrap();
        let mut evaluator = Evaluator::new();
        let result = evaluator.evaluate(module).unwrap();

        // All variables should be evaluated by the end
        assert_eq!(result.bindings.get("x").unwrap(), &Value::Int(2));
        assert_eq!(result.bindings.get("y").unwrap(), &Value::Int(3));
        assert_eq!(result.bindings.get("z").unwrap(), &Value::Int(6));
    }

    #[test]
    fn test_lazy_variable_dependency_chain() {
        // Test that dependent variables are evaluated correctly
        let input = r#"
            a = 10
            b = a * 2
            c = b + a
            result = c - 5
        "#;
        let module = crate::parse_str(input).unwrap();
        let mut evaluator = Evaluator::new();
        let result = evaluator.evaluate(module).unwrap();

        assert_eq!(result.bindings.get("a").unwrap(), &Value::Int(10));
        assert_eq!(result.bindings.get("b").unwrap(), &Value::Int(20));
        assert_eq!(result.bindings.get("c").unwrap(), &Value::Int(30));
        assert_eq!(result.bindings.get("result").unwrap(), &Value::Int(25));
    }

    #[test]
    fn test_lazy_variable_circular_dependency() {
        // Test that circular dependencies are detected
        let input = r#"
            x = y + 1
            y = x + 1
        "#;
        let module = crate::parse_str(input).unwrap();
        let mut evaluator = Evaluator::new();
        let result = evaluator.evaluate(module);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Circular dependency"));
    }

    #[test]
    fn test_lazy_variable_self_reference() {
        // Test that self-referential variables are detected
        let input = r#"
            x = x + 1
        "#;
        let module = crate::parse_str(input).unwrap();
        let mut evaluator = Evaluator::new();
        let result = evaluator.evaluate(module);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Circular dependency"));
    }

    #[test]
    fn test_lazy_variable_with_list_comprehension() {
        // Test lazy evaluation with list comprehensions
        let input = r#"
            numbers = [1, 2, 3, 4, 5]
            doubled = [x * 2 for x in numbers]
            sum = doubled[0] + doubled[1]
        "#;
        let module = crate::parse_str(input).unwrap();
        let mut evaluator = Evaluator::new();
        let result = evaluator.evaluate(module).unwrap();

        assert_eq!(
            result.bindings.get("doubled").unwrap(),
            &Value::List(vec![
                Value::Int(2),
                Value::Int(4),
                Value::Int(6),
                Value::Int(8),
                Value::Int(10)
            ])
        );
        assert_eq!(result.bindings.get("sum").unwrap(), &Value::Int(6));
    }

    #[test]
    fn test_lazy_variable_unused_error() {
        // Test that unused variables with errors don't prevent evaluation
        // if they're never accessed
        let input = r#"
            valid = 42
            unused = 1 / 0
        "#;
        let module = crate::parse_str(input).unwrap();
        let mut evaluator = Evaluator::new();
        let result = evaluator.evaluate(module);

        // This will fail because we force-evaluate all variables at the end
        // In a true lazy system, this might pass if 'unused' was never accessed
        assert!(result.is_err());
    }

    #[test]
    fn test_lazy_variable_type_annotation_validation() {
        // Test that type annotations are validated during lazy evaluation
        let input = r#"
            value: int = 1 + 1
            result = value * 2
        "#;
        let module = crate::parse_str(input).unwrap();
        let mut evaluator = Evaluator::new();
        let result = evaluator.evaluate(module).unwrap();

        assert_eq!(result.bindings.get("value").unwrap(), &Value::Int(2));
        assert_eq!(result.bindings.get("result").unwrap(), &Value::Int(4));
    }

    // Phase 3A: List comprehension index optimization tests
    #[test]
    fn test_list_comprehension_index_first() {
        // Test accessing first element only evaluates first item
        let input = r#"
            numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
            first = [x * 2 for x in numbers][0]
        "#;
        let module = crate::parse_str(input).unwrap();
        let mut evaluator = Evaluator::new();
        let result = evaluator.evaluate(module).unwrap();

        assert_eq!(result.bindings.get("first").unwrap(), &Value::Int(2));
    }

    #[test]
    fn test_list_comprehension_index_middle() {
        // Test accessing middle element only evaluates up to that index
        let input = r#"
            numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
            fifth = [x * 3 for x in numbers][4]
        "#;
        let module = crate::parse_str(input).unwrap();
        let mut evaluator = Evaluator::new();
        let result = evaluator.evaluate(module).unwrap();

        assert_eq!(result.bindings.get("fifth").unwrap(), &Value::Int(15));
    }

    #[test]
    fn test_list_comprehension_index_with_filter() {
        // Test filtered comprehension with index
        let input = r#"
            numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
            filtered_third = [x for x in numbers if x > 3][2]
        "#;
        let module = crate::parse_str(input).unwrap();
        let mut evaluator = Evaluator::new();
        let result = evaluator.evaluate(module).unwrap();

        // Items > 3: [4, 5, 6, 7, 8, 9, 10]
        // Index 2: 6
        assert_eq!(
            result.bindings.get("filtered_third").unwrap(),
            &Value::Int(6)
        );
    }

    #[test]
    fn test_list_comprehension_index_out_of_bounds() {
        // Test that indexing beyond comprehension size fails gracefully
        let input = r#"
            numbers = [1, 2, 3]
            invalid = [x * 2 for x in numbers][10]
        "#;
        let module = crate::parse_str(input).unwrap();
        let mut evaluator = Evaluator::new();
        let result = evaluator.evaluate(module);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Index out of bounds"));
    }

    #[test]
    fn test_list_comprehension_index_negative() {
        // Test that negative indices work (they should NOT hit optimization)
        let input = r#"
            numbers = [1, 2, 3, 4, 5]
            last = [x * 2 for x in numbers][-1]
        "#;
        let module = crate::parse_str(input).unwrap();
        let mut evaluator = Evaluator::new();
        let result = evaluator.evaluate(module).unwrap();

        assert_eq!(result.bindings.get("last").unwrap(), &Value::Int(10));
    }

    #[test]
    fn test_list_comprehension_index_complex_expression() {
        // Test complex expression in comprehension with index access
        let input = r#"
            numbers = [1, 2, 3, 4, 5]
            result = [x * 2 + 1 for x in numbers][2]
        "#;
        let module = crate::parse_str(input).unwrap();
        let mut evaluator = Evaluator::new();
        let result = evaluator.evaluate(module).unwrap();

        // [3, 5, 7, 9, 11][2] = 7
        assert_eq!(result.bindings.get("result").unwrap(), &Value::Int(7));
    }

    #[test]
    fn test_list_comprehension_index_string_elements() {
        // Test comprehension with string elements and index
        let input = r#"
            words = ["hello", "world", "test"]
            second = [upper(w) for w in words][1]
        "#;
        let module = crate::parse_str(input).unwrap();
        let mut evaluator = Evaluator::new();
        let result = evaluator.evaluate(module).unwrap();

        assert_eq!(
            result.bindings.get("second").unwrap(),
            &Value::String("WORLD".to_string())
        );
    }

    #[test]
    fn test_list_comprehension_normal_evaluation_still_works() {
        // Ensure normal (non-indexed) comprehensions still work correctly
        let input = r#"
            numbers = [1, 2, 3, 4, 5]
            doubled = [x * 2 for x in numbers]
            sum = doubled[0] + doubled[4]
        "#;
        let module = crate::parse_str(input).unwrap();
        let mut evaluator = Evaluator::new();
        let result = evaluator.evaluate(module).unwrap();

        assert_eq!(
            result.bindings.get("doubled").unwrap(),
            &Value::List(vec![
                Value::Int(2),
                Value::Int(4),
                Value::Int(6),
                Value::Int(8),
                Value::Int(10)
            ])
        );
        assert_eq!(result.bindings.get("sum").unwrap(), &Value::Int(12));
    }

    #[test]
    fn test_evaluate_slice_basic() {
        let evaluator = Evaluator::new();

        // Create a list [1, 2, 3, 4, 5]
        let list = Expression::List {
            elements: vec![
                Expression::Literal {
                    value: Value::Int(1),
                    span: None,
                },
                Expression::Literal {
                    value: Value::Int(2),
                    span: None,
                },
                Expression::Literal {
                    value: Value::Int(3),
                    span: None,
                },
                Expression::Literal {
                    value: Value::Int(4),
                    span: None,
                },
                Expression::Literal {
                    value: Value::Int(5),
                    span: None,
                },
            ],
            span: None,
        };

        // Test [1:3] - should return [2, 3]
        let slice = Expression::Slice {
            object: Box::new(list.clone()),
            start: Some(Box::new(Expression::Literal {
                value: Value::Int(1),
                span: None,
            })),
            end: Some(Box::new(Expression::Literal {
                value: Value::Int(3),
                span: None,
            })),
            step: None,
            span: None,
        };

        let result = evaluator.evaluate_expression(&slice).unwrap();
        assert_eq!(result, Value::List(vec![Value::Int(2), Value::Int(3)]));
    }

    #[test]
    fn test_evaluate_slice_full_copy() {
        let evaluator = Evaluator::new();

        let list = Expression::List {
            elements: vec![
                Expression::Literal {
                    value: Value::Int(1),
                    span: None,
                },
                Expression::Literal {
                    value: Value::Int(2),
                    span: None,
                },
                Expression::Literal {
                    value: Value::Int(3),
                    span: None,
                },
            ],
            span: None,
        };

        // Test [:] - full copy
        let slice = Expression::Slice {
            object: Box::new(list),
            start: None,
            end: None,
            step: None,
            span: None,
        };

        let result = evaluator.evaluate_expression(&slice).unwrap();
        assert_eq!(
            result,
            Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
        );
    }

    #[test]
    fn test_evaluate_slice_negative_indices() {
        let evaluator = Evaluator::new();

        let list = Expression::List {
            elements: vec![
                Expression::Literal {
                    value: Value::Int(1),
                    span: None,
                },
                Expression::Literal {
                    value: Value::Int(2),
                    span: None,
                },
                Expression::Literal {
                    value: Value::Int(3),
                    span: None,
                },
                Expression::Literal {
                    value: Value::Int(4),
                    span: None,
                },
                Expression::Literal {
                    value: Value::Int(5),
                    span: None,
                },
            ],
            span: None,
        };

        // Test [-2:] - last two elements
        let slice = Expression::Slice {
            object: Box::new(list),
            start: Some(Box::new(Expression::Literal {
                value: Value::Int(-2),
                span: None,
            })),
            end: None,
            step: None,
            span: None,
        };

        let result = evaluator.evaluate_expression(&slice).unwrap();
        assert_eq!(result, Value::List(vec![Value::Int(4), Value::Int(5)]));
    }

    #[test]
    fn test_evaluate_slice_with_step() {
        let evaluator = Evaluator::new();

        let list = Expression::List {
            elements: vec![
                Expression::Literal {
                    value: Value::Int(1),
                    span: None,
                },
                Expression::Literal {
                    value: Value::Int(2),
                    span: None,
                },
                Expression::Literal {
                    value: Value::Int(3),
                    span: None,
                },
                Expression::Literal {
                    value: Value::Int(4),
                    span: None,
                },
                Expression::Literal {
                    value: Value::Int(5),
                    span: None,
                },
            ],
            span: None,
        };

        // Test [::2] - every other element
        let slice = Expression::Slice {
            object: Box::new(list),
            start: None,
            end: None,
            step: Some(Box::new(Expression::Literal {
                value: Value::Int(2),
                span: None,
            })),
            span: None,
        };

        let result = evaluator.evaluate_expression(&slice).unwrap();
        assert_eq!(
            result,
            Value::List(vec![Value::Int(1), Value::Int(3), Value::Int(5)])
        );
    }

    #[test]
    fn test_evaluate_slice_reverse() {
        let evaluator = Evaluator::new();

        let list = Expression::List {
            elements: vec![
                Expression::Literal {
                    value: Value::Int(1),
                    span: None,
                },
                Expression::Literal {
                    value: Value::Int(2),
                    span: None,
                },
                Expression::Literal {
                    value: Value::Int(3),
                    span: None,
                },
                Expression::Literal {
                    value: Value::Int(4),
                    span: None,
                },
                Expression::Literal {
                    value: Value::Int(5),
                    span: None,
                },
            ],
            span: None,
        };

        // Test [::-1] - reverse
        let slice = Expression::Slice {
            object: Box::new(list),
            start: None,
            end: None,
            step: Some(Box::new(Expression::Literal {
                value: Value::Int(-1),
                span: None,
            })),
            span: None,
        };

        let result = evaluator.evaluate_expression(&slice).unwrap();
        assert_eq!(
            result,
            Value::List(vec![
                Value::Int(5),
                Value::Int(4),
                Value::Int(3),
                Value::Int(2),
                Value::Int(1)
            ])
        );
    }

    #[test]
    fn test_evaluate_slice_empty() {
        let evaluator = Evaluator::new();

        let list = Expression::List {
            elements: vec![
                Expression::Literal {
                    value: Value::Int(1),
                    span: None,
                },
                Expression::Literal {
                    value: Value::Int(2),
                    span: None,
                },
                Expression::Literal {
                    value: Value::Int(3),
                    span: None,
                },
            ],
            span: None,
        };

        // Test [3:1] - backwards range with positive step (empty)
        let slice = Expression::Slice {
            object: Box::new(list),
            start: Some(Box::new(Expression::Literal {
                value: Value::Int(3),
                span: None,
            })),
            end: Some(Box::new(Expression::Literal {
                value: Value::Int(1),
                span: None,
            })),
            step: None,
            span: None,
        };

        let result = evaluator.evaluate_expression(&slice).unwrap();
        assert_eq!(result, Value::List(vec![]));
    }

    #[test]
    fn test_evaluate_slice_error_zero_step() {
        let evaluator = Evaluator::new();

        let list = Expression::List {
            elements: vec![Expression::Literal {
                value: Value::Int(1),
                span: None,
            }],
            span: None,
        };

        // Test [::0] - zero step (error)
        let slice = Expression::Slice {
            object: Box::new(list),
            start: None,
            end: None,
            step: Some(Box::new(Expression::Literal {
                value: Value::Int(0),
                span: None,
            })),
            span: None,
        };

        let result = evaluator.evaluate_expression(&slice);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("step cannot be zero"));
    }

    #[test]
    fn test_evaluate_slice_error_non_list() {
        let evaluator = Evaluator::new();

        // Try to slice a non-list (error)
        let slice = Expression::Slice {
            object: Box::new(Expression::Literal {
                value: Value::Int(42),
                span: None,
            }),
            start: None,
            end: None,
            step: None,
            span: None,
        };

        let result = evaluator.evaluate_expression(&slice);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Cannot slice non-list"));
    }

    #[test]
    fn test_evaluate_multi_for_comprehension() {
        let evaluator = Evaluator::new();

        // [x + y for x in [1, 2] for y in [10, 20]]
        // Expected: [11, 21, 12, 22]
        let expr = Expression::ListComprehension {
            expr: Box::new(Expression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(Expression::Variable {
                    name: "x".to_string(),
                    span: None,
                }),
                right: Box::new(Expression::Variable {
                    name: "y".to_string(),
                    span: None,
                }),
                span: None,
            }),
            iterators: vec![
                (
                    "x".to_string(),
                    Expression::List {
                        elements: vec![
                            Expression::Literal {
                                value: Value::Int(1),
                                span: None,
                            },
                            Expression::Literal {
                                value: Value::Int(2),
                                span: None,
                            },
                        ],
                        span: None,
                    },
                ),
                (
                    "y".to_string(),
                    Expression::List {
                        elements: vec![
                            Expression::Literal {
                                value: Value::Int(10),
                                span: None,
                            },
                            Expression::Literal {
                                value: Value::Int(20),
                                span: None,
                            },
                        ],
                        span: None,
                    },
                ),
            ],
            condition: None,
            span: None,
        };

        let result = evaluator.evaluate_expression(&expr).unwrap();
        assert_eq!(
            result,
            Value::List(vec![
                Value::Int(11),
                Value::Int(21),
                Value::Int(12),
                Value::Int(22)
            ])
        );
    }

    #[test]
    fn test_evaluate_nested_comprehension() {
        let evaluator = Evaluator::new();

        // [[i * j for j in [1, 2, 3]] for i in [1, 2, 3]]
        // Expected: [[1, 2, 3], [2, 4, 6], [3, 6, 9]]
        let inner_comp = Expression::ListComprehension {
            expr: Box::new(Expression::BinaryOp {
                op: BinaryOperator::Multiply,
                left: Box::new(Expression::Variable {
                    name: "i".to_string(),
                    span: None,
                }),
                right: Box::new(Expression::Variable {
                    name: "j".to_string(),
                    span: None,
                }),
                span: None,
            }),
            iterators: vec![(
                "j".to_string(),
                Expression::List {
                    elements: vec![
                        Expression::Literal {
                            value: Value::Int(1),
                            span: None,
                        },
                        Expression::Literal {
                            value: Value::Int(2),
                            span: None,
                        },
                        Expression::Literal {
                            value: Value::Int(3),
                            span: None,
                        },
                    ],
                    span: None,
                },
            )],
            condition: None,
            span: None,
        };

        let outer_comp = Expression::ListComprehension {
            expr: Box::new(inner_comp),
            iterators: vec![(
                "i".to_string(),
                Expression::List {
                    elements: vec![
                        Expression::Literal {
                            value: Value::Int(1),
                            span: None,
                        },
                        Expression::Literal {
                            value: Value::Int(2),
                            span: None,
                        },
                        Expression::Literal {
                            value: Value::Int(3),
                            span: None,
                        },
                    ],
                    span: None,
                },
            )],
            condition: None,
            span: None,
        };

        let result = evaluator.evaluate_expression(&outer_comp).unwrap();
        assert_eq!(
            result,
            Value::List(vec![
                Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]),
                Value::List(vec![Value::Int(2), Value::Int(4), Value::Int(6)]),
                Value::List(vec![Value::Int(3), Value::Int(6), Value::Int(9)])
            ])
        );
    }

    #[test]
    fn test_evaluate_flattening_comprehension() {
        let mut evaluator = Evaluator::new();

        // Set up nested_list = [[1, 2], [3, 4], [5, 6]]
        evaluator.variables.insert(
            "nested_list".to_string(),
            Value::List(vec![
                Value::List(vec![Value::Int(1), Value::Int(2)]),
                Value::List(vec![Value::Int(3), Value::Int(4)]),
                Value::List(vec![Value::Int(5), Value::Int(6)]),
            ]),
        );

        // [num for sublist in nested_list for num in sublist]
        // Expected: [1, 2, 3, 4, 5, 6]
        let expr = Expression::ListComprehension {
            expr: Box::new(Expression::Variable {
                name: "num".to_string(),
                span: None,
            }),
            iterators: vec![
                (
                    "sublist".to_string(),
                    Expression::Variable {
                        name: "nested_list".to_string(),
                        span: None,
                    },
                ),
                (
                    "num".to_string(),
                    Expression::Variable {
                        name: "sublist".to_string(),
                        span: None,
                    },
                ),
            ],
            condition: None,
            span: None,
        };

        let result = evaluator.evaluate_expression(&expr).unwrap();
        assert_eq!(
            result,
            Value::List(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
                Value::Int(4),
                Value::Int(5),
                Value::Int(6)
            ])
        );
    }

    #[test]
    fn test_evaluate_multi_for_with_condition() {
        let evaluator = Evaluator::new();

        // [x + y for x in [1, 2, 3] for y in [10, 20, 30] if x + y > 20]
        // Expected: [21, 31, 22, 32, 23, 33]
        let expr = Expression::ListComprehension {
            expr: Box::new(Expression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(Expression::Variable {
                    name: "x".to_string(),
                    span: None,
                }),
                right: Box::new(Expression::Variable {
                    name: "y".to_string(),
                    span: None,
                }),
                span: None,
            }),
            iterators: vec![
                (
                    "x".to_string(),
                    Expression::List {
                        elements: vec![
                            Expression::Literal {
                                value: Value::Int(1),
                                span: None,
                            },
                            Expression::Literal {
                                value: Value::Int(2),
                                span: None,
                            },
                            Expression::Literal {
                                value: Value::Int(3),
                                span: None,
                            },
                        ],
                        span: None,
                    },
                ),
                (
                    "y".to_string(),
                    Expression::List {
                        elements: vec![
                            Expression::Literal {
                                value: Value::Int(10),
                                span: None,
                            },
                            Expression::Literal {
                                value: Value::Int(20),
                                span: None,
                            },
                            Expression::Literal {
                                value: Value::Int(30),
                                span: None,
                            },
                        ],
                        span: None,
                    },
                ),
            ],
            condition: Some(Box::new(Expression::BinaryOp {
                op: BinaryOperator::GreaterThan,
                left: Box::new(Expression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(Expression::Variable {
                        name: "x".to_string(),
                        span: None,
                    }),
                    right: Box::new(Expression::Variable {
                        name: "y".to_string(),
                        span: None,
                    }),
                    span: None,
                }),
                right: Box::new(Expression::Literal {
                    value: Value::Int(20),
                    span: None,
                }),
                span: None,
            })),
            span: None,
        };

        let result = evaluator.evaluate_expression(&expr).unwrap();
        assert_eq!(
            result,
            Value::List(vec![
                Value::Int(21),
                Value::Int(31),
                Value::Int(22),
                Value::Int(32),
                Value::Int(23),
                Value::Int(33)
            ])
        );
    }

    #[test]
    fn test_range_inclusive() {
        let evaluator = Evaluator::new();

        // [0..5] should produce [0, 1, 2, 3, 4, 5]
        let expr = Expression::Range {
            start: Box::new(Expression::Literal {
                value: Value::Int(0),
                span: None,
            }),
            end: Box::new(Expression::Literal {
                value: Value::Int(5),
                span: None,
            }),
            step: None,
            inclusive: true,
            span: None,
        };

        let result = evaluator.evaluate_expression(&expr).unwrap();
        assert_eq!(
            result,
            Value::List(vec![
                Value::Int(0),
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
                Value::Int(4),
                Value::Int(5),
            ])
        );
    }

    #[test]
    fn test_range_exclusive() {
        let evaluator = Evaluator::new();

        // [0..<5] should produce [0, 1, 2, 3, 4]
        let expr = Expression::Range {
            start: Box::new(Expression::Literal {
                value: Value::Int(0),
                span: None,
            }),
            end: Box::new(Expression::Literal {
                value: Value::Int(5),
                span: None,
            }),
            step: None,
            inclusive: false,
            span: None,
        };

        let result = evaluator.evaluate_expression(&expr).unwrap();
        assert_eq!(
            result,
            Value::List(vec![
                Value::Int(0),
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
                Value::Int(4),
            ])
        );
    }

    #[test]
    fn test_range_with_step() {
        let evaluator = Evaluator::new();

        // [0..10:2] should produce [0, 2, 4, 6, 8, 10]
        let expr = Expression::Range {
            start: Box::new(Expression::Literal {
                value: Value::Int(0),
                span: None,
            }),
            end: Box::new(Expression::Literal {
                value: Value::Int(10),
                span: None,
            }),
            step: Some(Box::new(Expression::Literal {
                value: Value::Int(2),
                span: None,
            })),
            inclusive: true,
            span: None,
        };

        let result = evaluator.evaluate_expression(&expr).unwrap();
        assert_eq!(
            result,
            Value::List(vec![
                Value::Int(0),
                Value::Int(2),
                Value::Int(4),
                Value::Int(6),
                Value::Int(8),
                Value::Int(10),
            ])
        );
    }

    #[test]
    fn test_range_descending() {
        let evaluator = Evaluator::new();

        // [10..0:-1] should produce [10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0]
        let expr = Expression::Range {
            start: Box::new(Expression::Literal {
                value: Value::Int(10),
                span: None,
            }),
            end: Box::new(Expression::Literal {
                value: Value::Int(0),
                span: None,
            }),
            step: Some(Box::new(Expression::Literal {
                value: Value::Int(-1),
                span: None,
            })),
            inclusive: true,
            span: None,
        };

        let result = evaluator.evaluate_expression(&expr).unwrap();
        assert_eq!(
            result,
            Value::List(vec![
                Value::Int(10),
                Value::Int(9),
                Value::Int(8),
                Value::Int(7),
                Value::Int(6),
                Value::Int(5),
                Value::Int(4),
                Value::Int(3),
                Value::Int(2),
                Value::Int(1),
                Value::Int(0),
            ])
        );
    }

    #[test]
    fn test_range_float() {
        let evaluator = Evaluator::new();

        // [0.0..2.0:0.5] should produce [0.0, 0.5, 1.0, 1.5, 2.0]
        let expr = Expression::Range {
            start: Box::new(Expression::Literal {
                value: Value::Float(0.0),
                span: None,
            }),
            end: Box::new(Expression::Literal {
                value: Value::Float(2.0),
                span: None,
            }),
            step: Some(Box::new(Expression::Literal {
                value: Value::Float(0.5),
                span: None,
            })),
            inclusive: true,
            span: None,
        };

        let result = evaluator.evaluate_expression(&expr).unwrap();
        assert_eq!(
            result,
            Value::List(vec![
                Value::Float(0.0),
                Value::Float(0.5),
                Value::Float(1.0),
                Value::Float(1.5),
                Value::Float(2.0),
            ])
        );
    }

    #[test]
    fn test_splat_basic() {
        let evaluator = Evaluator::new();

        // Create users list with maps
        let users = Expression::List {
            elements: vec![
                Expression::Map {
                    entries: vec![
                        (
                            "name".to_string(),
                            Expression::Literal {
                                value: Value::String("Alice".to_string()),
                                span: None,
                            },
                        ),
                        (
                            "age".to_string(),
                            Expression::Literal {
                                value: Value::Int(30),
                                span: None,
                            },
                        ),
                    ],
                    span: None,
                },
                Expression::Map {
                    entries: vec![
                        (
                            "name".to_string(),
                            Expression::Literal {
                                value: Value::String("Bob".to_string()),
                                span: None,
                            },
                        ),
                        (
                            "age".to_string(),
                            Expression::Literal {
                                value: Value::Int(25),
                                span: None,
                            },
                        ),
                    ],
                    span: None,
                },
            ],
            span: None,
        };

        // users[*].name
        let expr = Expression::MemberAccess {
            object: Box::new(Expression::Splat {
                object: Box::new(users),
                span: None,
            }),
            field: "name".to_string(),
            span: None,
        };

        let result = evaluator.evaluate_expression(&expr).unwrap();
        assert_eq!(
            result,
            Value::List(vec![
                Value::String("Alice".to_string()),
                Value::String("Bob".to_string()),
            ])
        );
    }

    #[test]
    fn test_splat_nested_access() {
        let evaluator = Evaluator::new();

        // Create orders with nested customer data
        let orders = Expression::List {
            elements: vec![
                Expression::Map {
                    entries: vec![(
                        "customer".to_string(),
                        Expression::Map {
                            entries: vec![(
                                "email".to_string(),
                                Expression::Literal {
                                    value: Value::String("alice@example.com".to_string()),
                                    span: None,
                                },
                            )],
                            span: None,
                        },
                    )],
                    span: None,
                },
                Expression::Map {
                    entries: vec![(
                        "customer".to_string(),
                        Expression::Map {
                            entries: vec![(
                                "email".to_string(),
                                Expression::Literal {
                                    value: Value::String("bob@example.com".to_string()),
                                    span: None,
                                },
                            )],
                            span: None,
                        },
                    )],
                    span: None,
                },
            ],
            span: None,
        };

        // orders[*].customer.email
        let expr = Expression::MemberAccess {
            object: Box::new(Expression::MemberAccess {
                object: Box::new(Expression::Splat {
                    object: Box::new(orders),
                    span: None,
                }),
                field: "customer".to_string(),
                span: None,
            }),
            field: "email".to_string(),
            span: None,
        };

        let result = evaluator.evaluate_expression(&expr).unwrap();
        assert_eq!(
            result,
            Value::List(vec![
                Value::String("alice@example.com".to_string()),
                Value::String("bob@example.com".to_string()),
            ])
        );
    }

    #[test]
    fn test_splat_empty_list() {
        let evaluator = Evaluator::new();

        // Empty list
        let empty_list = Expression::List {
            elements: vec![],
            span: None,
        };

        // [][*].name should return []
        let expr = Expression::MemberAccess {
            object: Box::new(Expression::Splat {
                object: Box::new(empty_list),
                span: None,
            }),
            field: "name".to_string(),
            span: None,
        };

        let result = evaluator.evaluate_expression(&expr).unwrap();
        assert_eq!(result, Value::List(vec![]));
    }

    #[test]
    fn test_splat_on_non_list() {
        let evaluator = Evaluator::new();

        // Try to splat on an integer (should error)
        let expr = Expression::Splat {
            object: Box::new(Expression::Literal {
                value: Value::Int(42),
                span: None,
            }),
            span: None,
        };

        let result = evaluator.evaluate_expression(&expr);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Splat operator [*] requires a list"));
    }

    #[test]
    fn test_splat_missing_field() {
        let evaluator = Evaluator::new();

        // Create list with maps that don't have the requested field
        let users = Expression::List {
            elements: vec![Expression::Map {
                entries: vec![(
                    "name".to_string(),
                    Expression::Literal {
                        value: Value::String("Alice".to_string()),
                        span: None,
                    },
                )],
                span: None,
            }],
            span: None,
        };

        // users[*].missing should error
        let expr = Expression::MemberAccess {
            object: Box::new(Expression::Splat {
                object: Box::new(users),
                span: None,
            }),
            field: "missing".to_string(),
            span: None,
        };

        let result = evaluator.evaluate_expression(&expr);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Field 'missing' not found"));
    }
}
