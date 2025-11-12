//! Type system for JCL

use crate::ast::{Type, Value};
use anyhow::{anyhow, Result};
use std::collections::HashMap;

/// Type checker
pub struct TypeChecker {
    variables: HashMap<String, Type>,
}

impl TypeChecker {
    /// Create a new type checker
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    /// Register a variable type
    pub fn register_variable(&mut self, name: String, var_type: Type) {
        self.variables.insert(name, var_type);
    }

    /// Check if a value matches a type
    pub fn check(&self, value: &Value, expected: &Type) -> Result<()> {
        match (value, expected) {
            (Value::String(_), Type::String) => Ok(()),
            (Value::Int(_), Type::Int) => Ok(()),
            (Value::Float(_), Type::Float) => Ok(()),
            (Value::Bool(_), Type::Bool) => Ok(()),
            (Value::Null, _) => Ok(()), // Null is compatible with any type
            (_, Type::Any) => Ok(()),   // Any accepts anything

            (Value::List(items), Type::List(item_type)) => {
                for item in items {
                    self.check(item, item_type)?;
                }
                Ok(())
            }

            (Value::Map(map), Type::Map(key_type, value_type)) => {
                for (k, v) in map {
                    self.check(&Value::String(k.clone()), key_type)?;
                    self.check(v, value_type)?;
                }
                Ok(())
            }

            // Object type checking removed - use Map instead

            _ => Err(anyhow!(
                "Type mismatch: expected {:?}, got {:?}",
                expected,
                value
            )),
        }
    }

    /// Infer the type of a value
    pub fn infer(&self, value: &Value) -> Type {
        match value {
            Value::String(_) => Type::String,
            Value::Int(_) => Type::Int,
            Value::Float(_) => Type::Float,
            Value::Bool(_) => Type::Bool,
            Value::Null => Type::Any,
            Value::List(items) => {
                if items.is_empty() {
                    Type::List(Box::new(Type::Any))
                } else {
                    Type::List(Box::new(self.infer(&items[0])))
                }
            }
            Value::Map(map) => {
                if map.is_empty() {
                    Type::Map(Box::new(Type::String), Box::new(Type::Any))
                } else {
                    let first_value = map.values().next().unwrap();
                    Type::Map(Box::new(Type::String), Box::new(self.infer(first_value)))
                }
            }
            Value::Function { .. } => Type::Any, // Functions have complex types
        }
    }

    /// Check type compatibility (can assign from -> to)
    pub fn is_compatible(&self, from: &Type, to: &Type) -> bool {
        match (from, to) {
            (_, Type::Any) => true,
            (Type::Any, _) => true,
            (a, b) if a == b => true,
            (Type::Int, Type::Float) => true, // Allow int -> float coercion
            (Type::List(a), Type::List(b)) => self.is_compatible(a, b),
            (Type::Map(k1, v1), Type::Map(k2, v2)) => {
                self.is_compatible(k1, k2) && self.is_compatible(v1, v2)
            }
            _ => false,
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_string() {
        let checker = TypeChecker::new();
        let value = Value::String("hello".to_string());
        assert!(checker.check(&value, &Type::String).is_ok());
        assert!(checker.check(&value, &Type::Int).is_err());
    }

    #[test]
    fn test_check_list() {
        let checker = TypeChecker::new();
        let value = Value::List(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
        ]);
        assert!(checker.check(&value, &Type::List(Box::new(Type::Int))).is_ok());
        assert!(checker
            .check(&value, &Type::List(Box::new(Type::String)))
            .is_err());
    }

    #[test]
    fn test_infer_type() {
        let checker = TypeChecker::new();
        assert_eq!(checker.infer(&Value::String("test".to_string())), Type::String);
        assert_eq!(checker.infer(&Value::Int(42)), Type::Int);
        assert_eq!(checker.infer(&Value::Bool(true)), Type::Bool);
    }

    #[test]
    fn test_type_compatibility() {
        let checker = TypeChecker::new();
        assert!(checker.is_compatible(&Type::Int, &Type::Float));
        assert!(checker.is_compatible(&Type::String, &Type::Any));
        assert!(!checker.is_compatible(&Type::String, &Type::Int));
    }
}
