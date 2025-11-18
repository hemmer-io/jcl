//! Language bindings for JCL
//!
//! This module contains bindings for various programming languages.

// WebAssembly support
#[cfg(target_arch = "wasm32")]
pub mod wasm;

// C FFI support
#[cfg(feature = "ffi")]
pub mod ffi;

// Python bindings
#[cfg(feature = "python")]
pub mod python;

// Node.js bindings
#[cfg(feature = "nodejs")]
pub mod nodejs;

// Java bindings
#[cfg(feature = "java")]
pub mod java;

// Ruby bindings
#[cfg(feature = "ruby")]
pub mod ruby;
