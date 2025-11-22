# JCL Library Performance Benchmarks

This directory contains benchmarks demonstrating the performance characteristics of JCL library bindings compared to CLI usage and JSON parsing.

## Overview

These benchmarks provide concrete evidence for choosing JCL library bindings over CLI subprocess invocation for production applications.

## Benchmarks

### 1. Library vs CLI (`library_vs_cli.py`)

Compares JCL Python library (PyO3) against CLI subprocess invocation.

**Expected Results:**
- Library: ~0.02-0.05ms
- CLI: ~5ms
- **Speedup: 50-100x** ⚡

**Key Finding:** The CLI includes ~5ms subprocess spawn overhead, making the library bindings dramatically faster for repeated evaluations.

### 2. Library vs JSON (`library_vs_json.py`)

Fair comparison between JCL library and Python's built-in JSON parser.

**Expected Results:**
- JCL Library: ~0.02-0.05ms
- JSON (stdlib): ~0.01-0.02ms
- **Relative: 1-3x slower**

**Key Finding:** JCL library performance is comparable to JSON parsing, despite providing type safety, functions, and validation.

### 3. Scaling (`scaling_benchmark.py`)

Shows how library performance scales with configuration file size.

**Expected Results:**
- Small (10 lines): ~0.02ms (library) vs ~5ms (CLI) = **250x faster**
- Medium (100 lines): ~0.05ms (library) vs ~5ms (CLI) = **100x faster**
- Large (1000 lines): ~0.5ms (library) vs ~5.5ms (CLI) = **11x faster**

**Key Finding:** Library performance scales linearly with file size, while CLI has constant overhead. Advantage is greatest for small configs.

## Running the Benchmarks

### Prerequisites

1. **Build JCL**:
   ```bash
   cargo build --release
   ```

2. **Install Python bindings** (for library benchmarks):
   ```bash
   cd bindings/python
   pip install maturin
   maturin develop --release
   ```

3. **Install dependencies**:
   ```bash
   # No additional dependencies required - uses Python stdlib
   ```

### Run Benchmarks

```bash
cd benchmarks/library

# Library vs CLI comparison
python3 library_vs_cli.py

# Library vs JSON comparison
python3 library_vs_json.py

# Scaling benchmark
python3 scaling_benchmark.py
```

## Interpreting Results

### When to Use Library Bindings

✅ **Use library bindings when:**
- Loading configs frequently (web servers, APIs)
- Latency matters (< 1ms response times)
- Embedded in application code
- Need for hot-reloading configs

### When CLI is Acceptable

⚠️ **CLI is acceptable when:**
- One-time config loading at startup
- Shell scripts and automation
- Latency doesn't matter (>100ms is fine)
- Can't install language bindings

## Example Output

```
================================================================================
JCL Library vs CLI Performance Benchmark
================================================================================

Library (Python PyO3):      0.0234 ms  (median: 0.0221 ms, σ: 0.0056 ms)
                           42,735 ops/sec

CLI (subprocess):           5.1234 ms  (median: 5.0987 ms, σ: 0.2134 ms)
                              195 ops/sec

Speedup:                     218.9x ⚡

================================================================================

Key Findings:
✅ Library is 219x faster than CLI
✅ CLI includes ~5ms subprocess spawn overhead
✅ Library performance is comparable to native code

Recommendation: Use library bindings for production applications
================================================================================
```

## Benchmark Configuration

- **Warmup runs**: 5 (to warm up CPU caches)
- **Benchmark runs**: 100 (CLI), 1000 (library/JSON)
- **Statistical method**: Mean with median and standard deviation
- **Timing**: `time.perf_counter()` (high-precision)

## Test Fixtures

- `fixtures/small.jcf` - 10 lines, typical microservice config
- `fixtures/medium.jcf` - 100 lines, complex service config
- `fixtures/large.jcf` - 1000 lines, platform-scale config
- `fixtures/small.json` - JSON equivalent for comparison

## Notes

- Benchmarks run on the same machine to ensure fair comparison
- CLI benchmarks include full subprocess spawn + execution + teardown
- Library benchmarks include file I/O, parsing, type checking, and evaluation
- Results will vary based on hardware, but relative performance is consistent

## Related Documentation

- [Python Bindings Documentation](../../docs/bindings/python.md)
- [Performance Comparison Guide](../../docs/guides/performance.md) *(future)*
- [Issue #50](https://github.com/hemmer-io/jcl/issues/50) - Original request

## License

MIT OR Apache-2.0
