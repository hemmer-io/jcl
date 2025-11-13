/**
 * @file jcl.h
 * @brief C API for JCL (Jack-of-All Configuration Language)
 *
 * This header provides a C-compatible API for embedding JCL in other languages.
 *
 * ## Basic Usage
 *
 * ```c
 * #include <jcl.h>
 * #include <stdio.h>
 *
 * int main() {
 *     jcl_init();
 *
 *     const char* source = "x = 42\ny = x * 2";
 *     JclResult result = jcl_parse(source);
 *
 *     if (result.success) {
 *         printf("Parse successful: %s\n", result.value);
 *     } else {
 *         printf("Parse error: %s\n", result.error);
 *     }
 *
 *     jcl_free_result(&result);
 *     return 0;
 * }
 * ```
 *
 * ## Memory Management
 *
 * - Strings returned in JclResult must be freed with jcl_free_result()
 * - The version string from jcl_version() is static and should NOT be freed
 * - Always call jcl_free_result() after using a JclResult
 *
 * @author JCL Contributors
 * @version 0.1.0
 */

#ifndef JCL_H
#define JCL_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdbool.h>
#include <stdint.h>

/**
 * @brief Opaque handle to a JCL parse result
 *
 * This type is not directly accessible. Use the provided functions to work with it.
 */
typedef struct JclModule JclModule;

/**
 * @brief Result of a JCL operation
 *
 * Contains either a success value or an error message.
 * The caller is responsible for freeing the result with jcl_free_result().
 */
typedef struct {
    bool success;      /**< True if operation succeeded, false otherwise */
    char* value;       /**< Success value (null if error). Caller must free. */
    char* error;       /**< Error message (null if success). Caller must free. */
} JclResult;

/**
 * @brief Initialize JCL library
 *
 * This function must be called before using any other JCL functions.
 * Currently a no-op, but reserved for future initialization logic.
 *
 * @return 0 on success, non-zero on error
 *
 * @code
 * if (jcl_init() != 0) {
 *     fprintf(stderr, "Failed to initialize JCL\n");
 *     return 1;
 * }
 * @endcode
 */
int jcl_init(void);

/**
 * @brief Parse JCL source code
 *
 * Validates the syntax of JCL source code.
 *
 * @param source Null-terminated UTF-8 string containing JCL source code
 * @return JclResult with parse status. Caller must free with jcl_free_result().
 *
 * @code
 * const char* source = "x = 42\ny = x + 1";
 * JclResult result = jcl_parse(source);
 *
 * if (result.success) {
 *     printf("Parse successful\n");
 * } else {
 *     printf("Parse error: %s\n", result.error);
 * }
 *
 * jcl_free_result(&result);
 * @endcode
 *
 * @note source must be valid UTF-8
 * @note Returns error if source is NULL
 */
JclResult jcl_parse(const char* source);

/**
 * @brief Format JCL source code
 *
 * Auto-formats JCL source code according to standard style guidelines.
 *
 * @param source Null-terminated UTF-8 string containing JCL source code
 * @return JclResult with formatted code. Caller must free with jcl_free_result().
 *
 * @code
 * const char* source = "x=42";
 * JclResult result = jcl_format(source);
 *
 * if (result.success) {
 *     printf("Formatted code:\n%s\n", result.value);
 * } else {
 *     printf("Format error: %s\n", result.error);
 * }
 *
 * jcl_free_result(&result);
 * @endcode
 *
 * @note source must be valid UTF-8
 * @note Returns error if source is NULL or has syntax errors
 */
JclResult jcl_format(const char* source);

/**
 * @brief Lint JCL source code
 *
 * Checks JCL source code for style issues and best practice violations.
 * Returns lint issues as a JSON array.
 *
 * @param source Null-terminated UTF-8 string containing JCL source code
 * @return JclResult with lint issues as JSON. Caller must free with jcl_free_result().
 *
 * @code
 * const char* source = "CONSTANT = 42";
 * JclResult result = jcl_lint(source);
 *
 * if (result.success) {
 *     printf("Lint results:\n%s\n", result.value);
 * } else {
 *     printf("Lint error: %s\n", result.error);
 * }
 *
 * jcl_free_result(&result);
 * @endcode
 *
 * @note source must be valid UTF-8
 * @note Returns error if source is NULL or has syntax errors
 * @note Result value is JSON-formatted string
 */
JclResult jcl_lint(const char* source);

/**
 * @brief Generate documentation from JCL source code
 *
 * Extracts function signatures and doc comments to generate Markdown documentation.
 *
 * @param source Null-terminated UTF-8 string containing JCL source code
 * @param module_name Null-terminated UTF-8 string for the module name
 * @return JclResult with Markdown documentation. Caller must free with jcl_free_result().
 *
 * @code
 * const char* source = "/// Adds two numbers\nfn add(x: int, y: int): int = x + y";
 * JclResult result = jcl_generate_docs(source, "math");
 *
 * if (result.success) {
 *     printf("Documentation:\n%s\n", result.value);
 * } else {
 *     printf("Doc generation error: %s\n", result.error);
 * }
 *
 * jcl_free_result(&result);
 * @endcode
 *
 * @note Both source and module_name must be valid UTF-8
 * @note Returns error if either parameter is NULL
 */
JclResult jcl_generate_docs(const char* source, const char* module_name);

/**
 * @brief Get JCL version string
 *
 * Returns the version of the JCL library.
 *
 * @return Pointer to static null-terminated UTF-8 string. Do NOT free this pointer.
 *
 * @code
 * const char* version = jcl_version();
 * printf("JCL version: %s\n", version);
 * @endcode
 *
 * @warning Do NOT free the returned pointer - it points to static memory
 */
const char* jcl_version(void);

/**
 * @brief Free a string returned by JCL functions
 *
 * Frees memory allocated for strings returned by JCL functions.
 *
 * @param ptr Pointer to string returned by JCL function
 *
 * @note ptr must have been allocated by JCL
 * @note ptr must not be used after this call
 * @note Do NOT use for the static string from jcl_version()
 * @note Safe to call with NULL pointer (no-op)
 *
 * @deprecated Use jcl_free_result() instead for JclResult structs
 */
void jcl_free_string(char* ptr);

/**
 * @brief Free a JclResult returned by JCL functions
 *
 * Frees all memory associated with a JclResult, including value and error strings.
 *
 * @param result Pointer to JclResult to free
 *
 * @code
 * JclResult result = jcl_parse("x = 42");
 * // ... use result ...
 * jcl_free_result(&result);
 * @endcode
 *
 * @note result and its contents must not be used after this call
 * @note Safe to call with NULL pointer (no-op)
 * @note Automatically frees both value and error strings
 */
void jcl_free_result(JclResult* result);

#ifdef __cplusplus
}
#endif

#endif /* JCL_H */
