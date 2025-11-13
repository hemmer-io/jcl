/* tslint:disable */
/* eslint-disable */
/**
 * Convenience function to generate docs without creating an instance
 */
export function generate_jcl_docs(source: string, module_name: string): JclResult;
/**
 * Convenience function to format JCL without creating an instance
 */
export function format_jcl(source: string): JclResult;
/**
 * Convenience function to parse JCL without creating an instance
 */
export function parse_jcl(source: string): JclResult;
/**
 * Convenience function to lint JCL without creating an instance
 */
export function lint_jcl(source: string): JclResult;
/**
 * Main JCL interface for WebAssembly
 */
export class Jcl {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Generate documentation from JCL source code
   */
  generate_docs(source: string, module_name: string): JclResult;
  /**
   * Create a new JCL instance
   */
  constructor();
  /**
   * Run linter on JCL source code
   */
  lint(source: string): JclResult;
  /**
   * Parse JCL source code
   */
  parse(source: string): JclResult;
  /**
   * Format JCL source code
   */
  format(source: string): JclResult;
  /**
   * Get the JCL version
   */
  static version(): string;
}
/**
 * Result type returned to JavaScript
 */
export class JclResult {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Check if the operation was successful
   */
  is_success(): boolean;
  /**
   * Get the error message (empty string if success)
   */
  error(): string;
  /**
   * Get the result value (empty string if error)
   */
  value(): string;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_jcl_free: (a: number, b: number) => void;
  readonly __wbg_jclresult_free: (a: number, b: number) => void;
  readonly format_jcl: (a: number, b: number) => number;
  readonly generate_jcl_docs: (a: number, b: number, c: number, d: number) => number;
  readonly jcl_format: (a: number, b: number, c: number) => number;
  readonly jcl_generate_docs: (a: number, b: number, c: number, d: number, e: number) => number;
  readonly jcl_lint: (a: number, b: number, c: number) => number;
  readonly jcl_new: () => number;
  readonly jcl_parse: (a: number, b: number, c: number) => number;
  readonly jcl_version: () => [number, number];
  readonly jclresult_error: (a: number) => [number, number];
  readonly jclresult_is_success: (a: number) => number;
  readonly jclresult_value: (a: number) => [number, number];
  readonly lint_jcl: (a: number, b: number) => number;
  readonly parse_jcl: (a: number, b: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
