//! Evaluator for JCL - resolves variables, functions, and expressions

use crate::ast::{
    BinaryOperator, Expression, Module, Pattern, Statement, StringPart, UnaryOperator, Value,
    WhenArm,
};
use crate::functions;
use anyhow::{anyhow, Result};
use std::collections::HashMap;

/// Evaluated module with all expressions resolved
#[derive(Debug)]
pub struct EvaluatedModule {
    pub bindings: HashMap<String, Value>,
}

/// Evaluator context
pub struct Evaluator {
    pub variables: HashMap<String, Value>,
    pub functions: HashMap<String, Value>,
}

impl Evaluator {
    /// Create a new evaluator
    pub fn new() -> Self {
        let mut evaluator = Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
        };
        evaluator.register_builtins();
        evaluator
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
                Statement::Import { .. } => {
                    // Imports are not yet implemented
                    return Err(anyhow!("Imports are not yet implemented"));
                }
                Statement::Expression { expr, .. } => {
                    // Expression statements - evaluate but don't bind
                    self.evaluate_expression(&expr)?;
                }
            }
        }

        Ok(EvaluatedModule { bindings })
    }

    /// Evaluate an expression
    pub fn evaluate_expression(&self, expr: &Expression) -> Result<Value> {
        match expr {
            Expression::Literal { value, .. } => Ok(value.clone()),

            Expression::Variable { name, .. } => {
                // Check variables first, then functions
                if let Some(value) = self.variables.get(name) {
                    Ok(value.clone())
                } else if let Some(func) = self.functions.get(name) {
                    Ok(func.clone())
                } else {
                    Err(anyhow!("Undefined variable: {}", name))
                }
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

            Expression::MemberAccess { object, field, .. } => {
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

            Expression::ListComprehension {
                expr,
                variable,
                iterable,
                condition,
                ..
            } => {
                let iter_value = self.evaluate_expression(iterable)?;
                match iter_value {
                    Value::List(items) => {
                        let mut results = Vec::new();
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
                                let result = scoped_eval.evaluate_expression(expr)?;
                                results.push(result);
                            }
                        }
                        Ok(Value::List(results))
                    }
                    _ => Err(anyhow!("List comprehension requires iterable to be a list")),
                }
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

    /// Clone evaluator with additional variable binding (for scopes)
    fn clone_with_var(&self, var_name: &str, value: Value) -> Self {
        let mut new_eval = Self {
            variables: self.variables.clone(),
            functions: self.functions.clone(),
        };
        new_eval.variables.insert(var_name.to_string(), value);
        new_eval
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
            let result = self.call_user_function(&func_value, &[item.clone()])?;
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

        let mut entries = Vec::new();
        entries.push((
            "name".to_string(),
            Expression::Literal {
                value: Value::String("Alice".to_string()),
                span: None,
            },
        ));
        entries.push((
            "age".to_string(),
            Expression::Literal {
                value: Value::Int(30),
                span: None,
            },
        ));

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
            variable: "x".to_string(),
            iterable: Box::new(Expression::Variable {
                name: "numbers".to_string(),
                span: None,
            }),
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
}
