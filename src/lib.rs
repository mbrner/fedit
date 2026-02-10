//! FEdit - Exact File Edit Toolkit
//!
//! A Rust library for structured search-and-replace operations with Python bindings.

use pyo3::exceptions::{PyFileNotFoundError, PyIOError, PyValueError};
use pyo3::prelude::*;
use std::path::Path;

// Public Rust API
pub mod api;

// Re-export core items for Rust usage
pub use api::{
    decode_content, detect_line_endings, edit_file, encode_content, read_file, replace_in_content,
    write_file_atomic, EditError, EditResult, Encoding, LineEnding, ReplaceOptions,
};

/// Python-exposed encoding enum
#[pyclass(name = "Encoding")]
#[derive(Clone)]
pub struct PyEncoding(Encoding);

#[pymethods]
impl PyEncoding {
    #[new]
    #[pyo3(signature = (name="utf-8"))]
    fn new(name: &str) -> PyResult<Self> {
        Encoding::from_str(name)
            .map(PyEncoding)
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    #[staticmethod]
    fn utf8() -> Self {
        PyEncoding(Encoding::Utf8)
    }

    #[staticmethod]
    fn utf16() -> Self {
        PyEncoding(Encoding::Utf16Le)
    }

    #[staticmethod]
    fn iso8859_1() -> Self {
        PyEncoding(Encoding::Iso8859_1)
    }

    #[staticmethod]
    fn windows1252() -> Self {
        PyEncoding(Encoding::Windows1252)
    }

    fn __str__(&self) -> &'static str {
        self.0.as_str()
    }

    fn __repr__(&self) -> String {
        format!("Encoding('{}')", self.0.as_str())
    }
}

/// Python-exposed edit result
#[pyclass(name = "EditResult")]
#[derive(Clone)]
pub struct PyEditResult {
    #[pyo3(get)]
    content: String,
    #[pyo3(get)]
    replacements: usize,
    #[pyo3(get)]
    line_ending: Option<String>,
}

#[pymethods]
impl PyEditResult {
    fn __repr__(&self) -> String {
        format!(
            "EditResult(replacements={}, line_ending={:?})",
            self.replacements, self.line_ending
        )
    }
}

impl From<EditResult> for PyEditResult {
    fn from(r: EditResult) -> Self {
        PyEditResult {
            content: r.content,
            replacements: r.replacements,
            line_ending: r.line_ending.map(|le| le.as_str().to_string()),
        }
    }
}

/// Convert EditError to Python exception
fn edit_error_to_py(e: EditError) -> PyErr {
    match e {
        EditError::NotFound(s) => PyValueError::new_err(format!("No matches found for: {}", s)),
        EditError::MultipleFound(n) => PyValueError::new_err(format!(
            "Multiple matches found ({}); use multiple=True to replace all",
            n
        )),
        EditError::IoError(msg) => {
            if msg.starts_with("No such file:") {
                PyFileNotFoundError::new_err(msg)
            } else {
                PyIOError::new_err(msg)
            }
        }
        EditError::EncodingError(msg) => PyValueError::new_err(format!("Encoding error: {}", msg)),
        EditError::InvalidKeyPath(p) => PyValueError::new_err(format!("Invalid key path: {}", p)),
        EditError::KeyNotFound(p) => PyValueError::new_err(format!("Key not found: {}", p)),
        EditError::Other(msg) => PyValueError::new_err(msg),
    }
}

/// Replace text in a string (in-memory operation)
///
/// Args:
///     content: The text content to search in
///     search: The string to search for
///     replace: The replacement string
///     multiple: If True, replace all occurrences; if False, error on multiple matches
///     ignore_whitespace: If True, treat consecutive whitespace as equivalent
///
/// Returns:
///     EditResult with the modified content and replacement count
#[pyfunction]
#[pyo3(signature = (content, search, replace, multiple=false, ignore_whitespace=false))]
fn replace_in_string(
    content: &str,
    search: &str,
    replace: &str,
    multiple: bool,
    ignore_whitespace: bool,
) -> PyResult<PyEditResult> {
    let options = ReplaceOptions {
        multiple,
        ignore_whitespace,
        ..Default::default()
    };

    replace_in_content(content, search, replace, &options)
        .map(PyEditResult::from)
        .map_err(edit_error_to_py)
}

/// Edit a file in place
///
/// Args:
///     path: Path to the file to edit
///     search: The string to search for
///     replace: The replacement string
///     multiple: If True, replace all occurrences
///     ignore_whitespace: If True, treat consecutive whitespace as equivalent
///     encoding: File encoding (default: "utf-8")
///     dry_run: If True, don't actually modify the file
///
/// Returns:
///     EditResult with replacement count and detected line ending
#[pyfunction]
#[pyo3(signature = (path, search, replace, multiple=false, ignore_whitespace=false, encoding="utf-8", dry_run=false))]
fn edit(
    path: &str,
    search: &str,
    replace: &str,
    multiple: bool,
    ignore_whitespace: bool,
    encoding: &str,
    dry_run: bool,
) -> PyResult<PyEditResult> {
    let enc = Encoding::from_str(encoding).map_err(edit_error_to_py)?;

    let options = ReplaceOptions {
        multiple,
        ignore_whitespace,
        encoding: enc,
        dry_run,
    };

    edit_file(Path::new(path), search, replace, &options)
        .map(PyEditResult::from)
        .map_err(edit_error_to_py)
}

/// Read a file and return its contents as a string
///
/// Args:
///     path: Path to the file
///     encoding: File encoding (default: "utf-8")
///
/// Returns:
///     Tuple of (content, line_ending) where line_ending is "lf", "crlf", or None
#[pyfunction]
#[pyo3(signature = (path, encoding="utf-8"))]
fn read(path: &str, encoding: &str) -> PyResult<(String, Option<String>)> {
    let enc = Encoding::from_str(encoding).map_err(edit_error_to_py)?;
    let (content, line_ending) = read_file(Path::new(path), enc).map_err(edit_error_to_py)?;
    Ok((content, line_ending.map(|le| le.as_str().to_string())))
}

/// Detect line ending style from bytes
///
/// Args:
///     content: Raw bytes to analyze
///
/// Returns:
///     "lf", "crlf", or None if no line endings found
#[pyfunction]
fn detect_line_ending(content: &[u8]) -> Option<String> {
    detect_line_endings(content).map(|le| le.as_str().to_string())
}

/// Python module definition
#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyEncoding>()?;
    m.add_class::<PyEditResult>()?;
    m.add_function(wrap_pyfunction!(edit, m)?)?;
    m.add_function(wrap_pyfunction!(replace_in_string, m)?)?;
    m.add_function(wrap_pyfunction!(read, m)?)?;
    m.add_function(wrap_pyfunction!(detect_line_ending, m)?)?;
    Ok(())
}
