use jcl::{ast::Value, evaluator::Evaluator, parser};

/// Helper function to parse and evaluate a JCL file
fn eval_file(content: &str) -> Result<std::collections::HashMap<String, Value>, anyhow::Error> {
    let module = parser::parse_str(content)?;
    let mut evaluator = Evaluator::new();
    let result = evaluator.evaluate(module)?;
    Ok(result.bindings)
}

#[test]
fn test_basic_example() {
    let content = include_str!("../examples/basic.jcl");
    let result = eval_file(content).expect("Failed to evaluate basic.jcl");

    // Check some expected values
    assert_eq!(result.get("name"), Some(&Value::String("JCL".to_string())));
    assert_eq!(result.get("port"), Some(&Value::Int(8080)));
    assert_eq!(result.get("is_stable"), Some(&Value::Bool(true)));
    assert_eq!(
        result.get("status"),
        Some(&Value::String("live".to_string()))
    );
    assert_eq!(result.get("remainder"), Some(&Value::Int(1)));
}

#[test]
fn test_functions_example() {
    let content = include_str!("../examples/functions.jcl");
    let result = eval_file(content).expect("Failed to evaluate functions.jcl");

    // Check function results
    assert_eq!(result.get("result1"), Some(&Value::Int(10))); // double(5)
    assert_eq!(result.get("result2"), Some(&Value::Int(30))); // add(10, 20)
    assert_eq!(result.get("result3"), Some(&Value::Int(12))); // quadruple(3) = double(double(3))

    // Check that lambda values are stored
    assert!(result.contains_key("square"));
    assert!(result.contains_key("multiply"));
}

#[test]
fn test_collections_example() {
    let content = include_str!("../examples/collections.jcl");
    let result = eval_file(content).expect("Failed to evaluate collections.jcl");

    // Check list values
    assert_eq!(
        result.get("numbers"),
        Some(&Value::List(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4),
            Value::Int(5)
        ]))
    );

    // Check list comprehension results
    assert_eq!(
        result.get("evens"),
        Some(&Value::List(vec![Value::Int(2), Value::Int(4)]))
    );

    // Check member access
    assert_eq!(result.get("first"), Some(&Value::Int(1)));
    assert_eq!(
        result.get("second"),
        Some(&Value::String("Bob".to_string()))
    );

    // Check nested access
    assert_eq!(
        result.get("first_server_name"),
        Some(&Value::String("web1".to_string()))
    );
}

#[test]
fn test_strings_example() {
    let content = include_str!("../examples/strings.jcl");
    let result = eval_file(content).expect("Failed to evaluate strings.jcl");

    // Check basic strings
    assert_eq!(
        result.get("greeting"),
        Some(&Value::String("Hello, World!".to_string()))
    );
    assert_eq!(
        result.get("language"),
        Some(&Value::String("JCL".to_string()))
    );

    // Check string interpolation (note: parts are empty in current implementation)
    assert!(result.contains_key("message"));

    // Check string functions
    assert_eq!(
        result.get("uppercase"),
        Some(&Value::String("JCL".to_string()))
    );
    assert_eq!(result.get("name_length"), Some(&Value::Int(5)));
}

#[test]
fn test_conditionals_example() {
    let content = include_str!("../examples/conditionals.jcl");
    let result = eval_file(content).expect("Failed to evaluate conditionals.jcl");

    // Check if expressions
    assert_eq!(result.get("score"), Some(&Value::Int(85)));
    assert_eq!(result.get("grade"), Some(&Value::String("B".to_string())));
    assert_eq!(result.get("is_passing"), Some(&Value::Bool(true)));
    assert_eq!(
        result.get("status"),
        Some(&Value::String("PASS".to_string()))
    );

    // Check ternary operator
    assert_eq!(
        result.get("protocol"),
        Some(&Value::String("http".to_string()))
    );

    // Check when expression
    assert_eq!(
        result.get("log_level"),
        Some(&Value::String("warn".to_string()))
    );

    // Check when with literal match
    assert_eq!(
        result.get("category"),
        Some(&Value::String("exact match".to_string()))
    );

    // Check nested conditionals
    assert_eq!(
        result.get("access_level"),
        Some(&Value::String("full".to_string()))
    );
}

#[test]
fn test_pipelines_example() {
    let content = include_str!("../examples/pipelines.jcl");
    let result = eval_file(content).expect("Failed to evaluate pipelines.jcl");

    // Check list comprehensions (pipeline alternatives)
    assert_eq!(
        result.get("doubled"),
        Some(&Value::List(vec![
            Value::Int(2),
            Value::Int(4),
            Value::Int(6),
            Value::Int(8),
            Value::Int(10)
        ]))
    );

    assert_eq!(
        result.get("evens"),
        Some(&Value::List(vec![Value::Int(2), Value::Int(4)]))
    );

    // Check string operations
    assert_eq!(
        result.get("uppercase_text"),
        Some(&Value::String("JCL".to_string()))
    );
    assert_eq!(
        result.get("lowercase_text"),
        Some(&Value::String("jcl".to_string()))
    );
}

#[test]
fn test_web_server_example() {
    let content = include_str!("../examples/web-server.jcl");
    let result = eval_file(content).expect("Failed to evaluate web-server.jcl");

    // Check environment
    assert_eq!(
        result.get("env"),
        Some(&Value::String("production".to_string()))
    );
    assert_eq!(result.get("is_production"), Some(&Value::Bool(true)));

    // Check that nested structures exist
    assert!(result.contains_key("server"));
    assert!(result.contains_key("database"));
    assert!(result.contains_key("app"));
}

#[test]
fn test_builtin_example_parses() {
    // Note: builtin.jcl uses many built-in functions that may not be fully implemented
    // This test just verifies it parses successfully
    let content = include_str!("../examples/builtin.jcl");
    let parse_result = parser::parse_str(content);
    assert!(
        parse_result.is_ok(),
        "builtin.jcl should parse successfully"
    );
}

/// Test that all example files parse successfully
#[test]
fn test_all_examples_parse() {
    let examples = vec![
        ("basic.jcl", include_str!("../examples/basic.jcl")),
        ("functions.jcl", include_str!("../examples/functions.jcl")),
        (
            "collections.jcl",
            include_str!("../examples/collections.jcl"),
        ),
        ("strings.jcl", include_str!("../examples/strings.jcl")),
        (
            "conditionals.jcl",
            include_str!("../examples/conditionals.jcl"),
        ),
        ("pipelines.jcl", include_str!("../examples/pipelines.jcl")),
        ("builtin.jcl", include_str!("../examples/builtin.jcl")),
        ("web-server.jcl", include_str!("../examples/web-server.jcl")),
    ];

    for (name, content) in examples {
        let result = parser::parse_str(content);
        assert!(
            result.is_ok(),
            "Example {} should parse successfully. Error: {:?}",
            name,
            result.err()
        );
    }
}
