/**
 * JCL (Jack-of-All Configuration Language) Node.js bindings
 *
 * @packageDocumentation
 */

/**
 * Linter issue severity level
 */
export type Severity = 'error' | 'warning' | 'info';

/**
 * Linter issue
 */
export interface LintIssue {
  /** Rule that was violated */
  rule: string;
  /** Description of the issue */
  message: string;
  /** Severity level */
  severity: Severity;
  /** Optional suggestion for fixing */
  suggestion?: string;
}

/**
 * JCL value types that can be returned from evaluation
 */
export type JclValue =
  | string
  | number
  | boolean
  | null
  | JclValue[]
  | { [key: string]: JclValue };

/**
 * Result of evaluating JCL code
 */
export type EvalResult = { [key: string]: JclValue };

/**
 * Parse JCL source code
 *
 * @param source - JCL source code to parse
 * @returns Summary message about parsed statements
 * @throws {Error} If the source code has syntax errors
 *
 * @example
 * ```typescript
 * import * as jcl from '@hemmerio/jcl';
 *
 * const result = jcl.parse('name = "my-app"');
 * console.log(result); // "Parsed 1 statements"
 * ```
 */
export function parse(source: string): string;

/**
 * Evaluate JCL source code and return all defined variables
 *
 * @param source - JCL source code to evaluate
 * @returns Object containing all defined variables
 * @throws {Error} If the source code has syntax or runtime errors
 *
 * @example
 * ```typescript
 * import * as jcl from '@hemmerio/jcl';
 *
 * const result = jcl.eval(`
 *   name = "my-app"
 *   version = "1.0.0"
 *   ports = [80, 443]
 * `);
 *
 * console.log(result.name);    // "my-app"
 * console.log(result.version); // "1.0.0"
 * console.log(result.ports);   // [80, 443]
 * ```
 */
export function eval(source: string): EvalResult;

/**
 * Load and evaluate a JCL file
 *
 * @param path - Path to the JCL file
 * @returns Object containing all defined variables
 * @throws {Error} If the file cannot be read or has syntax/runtime errors
 *
 * @example
 * ```typescript
 * import * as jcl from '@hemmerio/jcl';
 *
 * const config = jcl.evalFile('./config.jcl');
 * console.log(config);
 * ```
 */
export function evalFile(path: string): EvalResult;

/**
 * Format JCL source code
 *
 * @param source - JCL source code to format
 * @returns Formatted JCL source code
 * @throws {Error} If the source code has syntax errors
 *
 * @example
 * ```typescript
 * import * as jcl from '@hemmerio/jcl';
 *
 * const formatted = jcl.format('x=1+2\ny   ="hello"');
 * console.log(formatted);
 * // Output:
 * // x = 1 + 2
 * // y = "hello"
 * ```
 */
export function format(source: string): string;

/**
 * Lint JCL source code and return issues
 *
 * @param source - JCL source code to lint
 * @returns Array of linter issues
 * @throws {Error} If the source code has syntax errors
 *
 * @example
 * ```typescript
 * import * as jcl from '@hemmerio/jcl';
 *
 * const issues = jcl.lint(`
 *   x = 1
 *   unused_var = 2
 *   y = x + 1
 * `);
 *
 * for (const issue of issues) {
 *   console.log(`${issue.severity}: ${issue.message}`);
 *   if (issue.suggestion) {
 *     console.log(`  Suggestion: ${issue.suggestion}`);
 *   }
 * }
 * ```
 */
export function lint(source: string): LintIssue[];
