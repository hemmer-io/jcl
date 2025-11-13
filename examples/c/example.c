/**
 * @file example.c
 * @brief Example program demonstrating JCL C API usage
 *
 * Compile:
 *   gcc -o example example.c -I../../include -L../../target/release -ljcl
 *
 * Run (Linux/macOS):
 *   LD_LIBRARY_PATH=../../target/release ./example
 *
 * Run (macOS):
 *   DYLD_LIBRARY_PATH=../../target/release ./example
 */

#include <stdio.h>
#include <stdlib.h>
#include "jcl.h"

void print_separator() {
    printf("\n----------------------------------------\n\n");
}

void example_parse() {
    printf("=== Parse Example ===\n");

    const char* source = "x = 42\ny = x + 1";
    printf("Source:\n%s\n\n", source);

    JclResult result = jcl_parse(source);

    if (result.success) {
        printf("✓ Parse successful\n");
        if (result.value) {
            printf("Result: %s\n", result.value);
        }
    } else {
        printf("✗ Parse failed\n");
        if (result.error) {
            printf("Error: %s\n", result.error);
        }
    }

    jcl_free_result(&result);
}

void example_format() {
    printf("=== Format Example ===\n");

    const char* source = "x=42\ny=x+1";
    printf("Unformatted:\n%s\n\n", source);

    JclResult result = jcl_format(source);

    if (result.success) {
        printf("Formatted:\n%s\n", result.value);
    } else {
        printf("✗ Format failed\n");
        if (result.error) {
            printf("Error: %s\n", result.error);
        }
    }

    jcl_free_result(&result);
}

void example_lint() {
    printf("=== Lint Example ===\n");

    const char* source = "CONSTANT = 42\nunused_var = 10";
    printf("Source:\n%s\n\n", source);

    JclResult result = jcl_lint(source);

    if (result.success) {
        printf("Lint results:\n%s\n", result.value);
    } else {
        printf("✗ Lint failed\n");
        if (result.error) {
            printf("Error: %s\n", result.error);
        }
    }

    jcl_free_result(&result);
}

void example_docs() {
    printf("=== Documentation Example ===\n");

    const char* source =
        "/// Calculates the sum of two numbers\n"
        "fn add(x: int, y: int): int = x + y\n"
        "\n"
        "/// Greets a person by name\n"
        "fn greet(name: string) = \"Hello, \" + name + \"!\"";

    printf("Source:\n%s\n\n", source);

    JclResult result = jcl_generate_docs(source, "example");

    if (result.success) {
        printf("Documentation:\n%s\n", result.value);
    } else {
        printf("✗ Doc generation failed\n");
        if (result.error) {
            printf("Error: %s\n", result.error);
        }
    }

    jcl_free_result(&result);
}

void example_error_handling() {
    printf("=== Error Handling Example ===\n");

    const char* invalid_source = "x = ";
    printf("Invalid source: %s\n\n", invalid_source);

    JclResult result = jcl_parse(invalid_source);

    if (!result.success) {
        printf("✓ Error correctly detected\n");
        if (result.error) {
            printf("Error message: %s\n", result.error);
        }
    } else {
        printf("✗ Error should have been detected\n");
    }

    jcl_free_result(&result);
}

int main() {
    printf("JCL C API Example\n");
    printf("Version: %s\n", jcl_version());
    print_separator();

    // Initialize JCL
    if (jcl_init() != 0) {
        fprintf(stderr, "Failed to initialize JCL\n");
        return 1;
    }

    // Run examples
    example_parse();
    print_separator();

    example_format();
    print_separator();

    example_lint();
    print_separator();

    example_docs();
    print_separator();

    example_error_handling();
    print_separator();

    printf("All examples completed successfully!\n");

    return 0;
}
