use pyo3::prelude::*;

// Public Rust API: provide a lightweight, library‑bound API for replacements.
// This is a separate module to keep the Python bindings thin while exposing a
// clean Rust interface for embedding.
pub mod api;

// Re-export core items for convenient usage from Rust code.
pub use api::{replace_in_content, EditError, EditResult, ReplaceOptions};

/// A Python module implemented in Rust. The name of this module must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
fn _core() -> PyResult<()> {
    // Minimal Python exposure to avoid altering existing bindings.
    Ok(())
}
