use pyo3::prelude::*;

// Public library API for string replacement functionality.
// Expose a pure Rust API in addition to the Python bindings.
pub mod api;

// Re-export common items for convenient access when using the library from Rust.
pub use api::{replace_in_content, EditError, EditResult, ReplaceOptions};

/// A Python module implemented in Rust. The name of this module must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
mod _core {
    use pyo3::prelude::*;

    #[pyfunction]
    fn hello_from_bin() -> String {
        "Hello from fedit!".to_string()
    }
}
