//! Documentation generator for JCL - extracts API documentation from code

use crate::ast::{Expression, Module, Statement, Type, Value};
use anyhow::Result;

/// Documentation for a function
#[derive(Debug, Clone)]
pub struct FunctionDoc {
    pub name: String,
    pub params: Vec<ParamDoc>,
    pub return_type: Option<Type>,
    pub description: Option<String>,
}

/// Documentation for a parameter
#[derive(Debug, Clone)]
pub struct ParamDoc {
    pub name: String,
    pub param_type: Option<Type>,
    pub default: Option<String>,
}

/// Documentation for a variable/constant
#[derive(Debug, Clone)]
pub struct VariableDoc {
    pub name: String,
    pub value_type: Option<Type>,
    pub value: String,
    pub mutable: bool,
}

/// Documentation for a module
#[derive(Debug, Clone)]
pub struct ModuleDoc {
    pub functions: Vec<FunctionDoc>,
    pub variables: Vec<VariableDoc>,
    pub imports: Vec<String>,
}

/// Generate documentation from a module
pub fn generate(module: &Module) -> Result<ModuleDoc> {
    let mut doc = ModuleDoc {
        functions: Vec::new(),
        variables: Vec::new(),
        imports: Vec::new(),
    };

    for statement in &module.statements {
        match statement {
            Statement::FunctionDef {
                name,
                params,
                return_type,
                body,
                doc_comments,
                ..
            } => {
                // Use doc comments if available, otherwise infer from name
                let description = doc_comments
                    .as_ref()
                    .and_then(|comments| {
                        if comments.is_empty() {
                            None
                        } else {
                            Some(comments.join("\n"))
                        }
                    })
                    .or_else(|| infer_function_description(name, body));

                doc.functions.push(FunctionDoc {
                    name: name.clone(),
                    params: params
                        .iter()
                        .map(|p| ParamDoc {
                            name: p.name.clone(),
                            param_type: p.param_type.clone(),
                            default: p.default.as_ref().map(expr_to_string),
                        })
                        .collect(),
                    return_type: return_type.clone(),
                    description,
                });
            }

            Statement::Assignment {
                name,
                value,
                type_annotation,
                mutable,
                doc_comments: _,
                ..
            } => {
                doc.variables.push(VariableDoc {
                    name: name.clone(),
                    value_type: type_annotation.clone(),
                    value: expr_to_string(value),
                    mutable: *mutable,
                });
            }

            Statement::Import { items, path, .. } => {
                doc.imports
                    .push(format!("import {} from \"{}\"", items.join(", "), path));
            }

            _ => {}
        }
    }

    Ok(doc)
}

/// Format documentation as Markdown
pub fn format_markdown(doc: &ModuleDoc, module_name: &str) -> String {
    let mut output = String::new();

    // Header
    output.push_str(&format!("# {}\n\n", module_name));

    // Imports
    if !doc.imports.is_empty() {
        output.push_str("## Imports\n\n");
        for import in &doc.imports {
            output.push_str(&format!("- `{}`\n", import));
        }
        output.push('\n');
    }

    // Variables/Constants
    if !doc.variables.is_empty() {
        output.push_str("## Variables\n\n");
        for var in &doc.variables {
            let mutability = if var.mutable { "mut " } else { "" };
            let type_annotation = var
                .value_type
                .as_ref()
                .map(|t| format!(": {}", type_to_string(t)))
                .unwrap_or_default();

            output.push_str(&format!(
                "### `{}{}{}`\n\n",
                mutability, var.name, type_annotation
            ));
            output.push_str(&format!("**Value:** `{}`\n\n", var.value));
        }
    }

    // Functions
    if !doc.functions.is_empty() {
        output.push_str("## Functions\n\n");
        for func in &doc.functions {
            // Function signature
            let params_str = func
                .params
                .iter()
                .map(|p| {
                    let type_str = p
                        .param_type
                        .as_ref()
                        .map(|t| format!(": {}", type_to_string(t)))
                        .unwrap_or_default();
                    let default_str = p
                        .default
                        .as_ref()
                        .map(|d| format!(" = {}", d))
                        .unwrap_or_default();
                    format!("{}{}{}", p.name, type_str, default_str)
                })
                .collect::<Vec<_>>()
                .join(", ");

            let return_type_str = func
                .return_type
                .as_ref()
                .map(|t| format!(": {}", type_to_string(t)))
                .unwrap_or_default();

            output.push_str(&format!(
                "### `{}({}){}`\n\n",
                func.name, params_str, return_type_str
            ));

            // Description
            if let Some(desc) = &func.description {
                output.push_str(&format!("{}\n\n", desc));
            }

            // Parameters
            if !func.params.is_empty() {
                output.push_str("**Parameters:**\n\n");
                for param in &func.params {
                    let type_str = param
                        .param_type
                        .as_ref()
                        .map(|t| format!(" ({}) ", type_to_string(t)))
                        .unwrap_or_else(|| " ".to_string());
                    let default_str = param
                        .default
                        .as_ref()
                        .map(|d| format!(" (default: `{}`)", d))
                        .unwrap_or_default();
                    output.push_str(&format!("- `{}`{}{}\n", param.name, type_str, default_str));
                }
                output.push('\n');
            }

            // Return type
            if let Some(ret_type) = &func.return_type {
                output.push_str(&format!("**Returns:** `{}`\n\n", type_to_string(ret_type)));
            }
        }
    }

    output
}

/// Convert an expression to a string representation
fn expr_to_string(expr: &Expression) -> String {
    match expr {
        Expression::Literal { value, .. } => value_to_string(value),
        Expression::Variable { name, .. } => name.clone(),
        Expression::List { elements, .. } => {
            let items_str = elements
                .iter()
                .map(expr_to_string)
                .collect::<Vec<_>>()
                .join(", ");
            format!("[{}]", items_str)
        }
        Expression::Map { entries, .. } => {
            let entries_str = entries
                .iter()
                .map(|(k, v)| format!("{} = {}", k, expr_to_string(v)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("({})", entries_str)
        }
        Expression::Lambda { params, body, .. } => {
            let params_str = params
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            format!("({}) => {}", params_str, expr_to_string(body))
        }
        Expression::FunctionCall { name, args, .. } => {
            let args_str = args
                .iter()
                .map(expr_to_string)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}({})", name, args_str)
        }
        Expression::BinaryOp {
            left, right, op, ..
        } => {
            format!(
                "{} {:?} {}",
                expr_to_string(left),
                op,
                expr_to_string(right)
            )
        }
        Expression::If {
            condition,
            then_expr,
            else_expr,
            ..
        } => {
            let else_str = else_expr
                .as_ref()
                .map(|e| format!(" else {}", expr_to_string(e)))
                .unwrap_or_default();
            format!(
                "if {} then {}{}",
                expr_to_string(condition),
                expr_to_string(then_expr),
                else_str
            )
        }
        Expression::MemberAccess { object, field, .. } => {
            format!("{}.{}", expr_to_string(object), field)
        }
        _ => "<expr>".to_string(),
    }
}

/// Convert a value to a string representation
fn value_to_string(val: &Value) -> String {
    match val {
        Value::String(s) => format!("\"{}\"", s),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        Value::List(items) => {
            let items_str = items
                .iter()
                .map(value_to_string)
                .collect::<Vec<_>>()
                .join(", ");
            format!("[{}]", items_str)
        }
        Value::Map(map) => {
            let entries_str = map
                .iter()
                .map(|(k, v)| format!("{} = {}", k, value_to_string(v)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("({})", entries_str)
        }
        Value::Function { params, .. } => {
            let params_str = params
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            format!("fn({})", params_str)
        }
    }
}

/// Convert a type to a string representation
fn type_to_string(typ: &Type) -> String {
    match typ {
        Type::String => "string".to_string(),
        Type::Int => "int".to_string(),
        Type::Float => "float".to_string(),
        Type::Bool => "bool".to_string(),
        Type::List(inner) => format!("list<{}>", type_to_string(inner)),
        Type::Map(key, value) => format!("map<{}, {}>", type_to_string(key), type_to_string(value)),
        Type::Function {
            params,
            return_type,
        } => {
            let params_str = params
                .iter()
                .map(type_to_string)
                .collect::<Vec<_>>()
                .join(", ");
            format!("({}) -> {}", params_str, type_to_string(return_type))
        }
        Type::Any => "any".to_string(),
        Type::Null => "null".to_string(),
    }
}

/// Infer a basic description from function name and body
fn infer_function_description(name: &str, _body: &Expression) -> Option<String> {
    // Try to infer purpose from name
    let description = if let Some(rest) = name.strip_prefix("get_") {
        Some(format!("Gets {}", rest.replace('_', " ")))
    } else if let Some(rest) = name.strip_prefix("set_") {
        Some(format!("Sets {}", rest.replace('_', " ")))
    } else if let Some(rest) = name
        .strip_prefix("is_")
        .or_else(|| name.strip_prefix("has_"))
    {
        Some(format!("Checks if {}", rest.replace('_', " ")))
    } else if let Some(rest) = name.strip_prefix("create_") {
        Some(format!("Creates {}", rest.replace('_', " ")))
    } else if let Some(rest) = name
        .strip_prefix("delete_")
        .or_else(|| name.strip_prefix("remove_"))
    {
        Some(format!("Deletes {}", rest.replace('_', " ")))
    } else if let Some(rest) = name.strip_prefix("validate_") {
        Some(format!("Validates {}", rest.replace('_', " ")))
    } else if let Some(rest) = name
        .strip_prefix("calculate_")
        .or_else(|| name.strip_prefix("compute_"))
    {
        Some(format!("Calculates {}", rest.replace('_', " ")))
    } else if name == "add" || name == "sum" {
        Some("Adds values together".to_string())
    } else if name == "subtract" || name == "sub" {
        Some("Subtracts values".to_string())
    } else if name == "multiply" || name == "mul" {
        Some("Multiplies values".to_string())
    } else if name == "divide" || name == "div" {
        Some("Divides values".to_string())
    } else {
        None
    };

    description
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    #[test]
    fn test_generate_function_docs() {
        let input = r#"
            fn add(x: int, y: int): int = x + y
            fn greet(name: string) = "Hello, " + name
        "#;
        let module = parser::parse_str(input).unwrap();
        let doc = generate(&module).unwrap();

        assert_eq!(doc.functions.len(), 2);
        assert_eq!(doc.functions[0].name, "add");
        assert_eq!(doc.functions[0].params.len(), 2);
        assert_eq!(doc.functions[1].name, "greet");
    }

    #[test]
    fn test_generate_variable_docs() {
        let input = r#"
            port: int = 8080
            host = "localhost"
        "#;
        let module = parser::parse_str(input).unwrap();
        let doc = generate(&module).unwrap();

        assert_eq!(doc.variables.len(), 2);
        assert_eq!(doc.variables[0].name, "port");
        assert_eq!(doc.variables[1].name, "host");
    }

    #[test]
    fn test_markdown_formatting() {
        let input = r#"
            fn calculate_total(price: float, tax: float): float = price * (1.0 + tax)
        "#;
        let module = parser::parse_str(input).unwrap();
        let doc = generate(&module).unwrap();
        let markdown = format_markdown(&doc, "test");

        assert!(markdown.contains("# test"));
        assert!(markdown.contains("calculate_total"));
        assert!(markdown.contains("price"));
        assert!(markdown.contains("tax"));
        assert!(markdown.contains("float"));
    }
}
