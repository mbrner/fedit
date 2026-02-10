//! FEdit - Exact File Edit Toolkit
//!
//! A Rust library for structured search-and-replace operations with Python bindings.

use pyo3::exceptions::{PyFileNotFoundError, PyIOError, PyValueError};
use pyo3::prelude::*;
use std::path::Path;

// Public Rust API
pub mod api;
pub mod structured;

// Re-export core items for Rust usage
pub use api::{
    count_fuzzy_occurrences, decode_content, detect_line_endings, detect_line_endings_str,
    edit_file, edit_file_fuzzy, encode_content, fuzzy_find_text, generate_diff,
    normalize_for_fuzzy_match, normalize_to_lf, read_file, replace_in_content,
    restore_line_endings, strip_bom, strip_bom_bytes, write_file_atomic, BomStripped, DiffResult,
    EditError, EditResult, EditResultWithDiff, Encoding, FuzzyMatchResult, LineEnding,
    ReplaceOptions, UTF8_BOM, UTF8_BOM_BYTES,
};

pub use structured::{
    edit_json, edit_json5, edit_jsonc, edit_structured, edit_structured_content, edit_toml,
    edit_yaml, StructuredEditResult, StructuredFormat,
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

/// Python-exposed edit result with diff information
#[pyclass(name = "EditResultWithDiff")]
#[derive(Clone)]
pub struct PyEditResultWithDiff {
    #[pyo3(get)]
    content: String,
    #[pyo3(get)]
    replacements: usize,
    #[pyo3(get)]
    line_ending: Option<String>,
    #[pyo3(get)]
    diff: String,
    #[pyo3(get)]
    first_changed_line: Option<usize>,
    #[pyo3(get)]
    used_fuzzy_match: bool,
}

#[pymethods]
impl PyEditResultWithDiff {
    fn __repr__(&self) -> String {
        format!(
            "EditResultWithDiff(replacements={}, line_ending={:?}, first_changed_line={:?}, used_fuzzy_match={})",
            self.replacements, self.line_ending, self.first_changed_line, self.used_fuzzy_match
        )
    }
}

impl From<api::EditResultWithDiff> for PyEditResultWithDiff {
    fn from(r: api::EditResultWithDiff) -> Self {
        PyEditResultWithDiff {
            content: r.content,
            replacements: r.replacements,
            line_ending: r.line_ending.map(|le| le.as_str().to_string()),
            diff: r.diff,
            first_changed_line: r.first_changed_line,
            used_fuzzy_match: r.used_fuzzy_match,
        }
    }
}

/// Python-exposed fuzzy match result
#[pyclass(name = "FuzzyMatchResult")]
#[derive(Clone)]
pub struct PyFuzzyMatchResult {
    #[pyo3(get)]
    found: bool,
    #[pyo3(get)]
    index: usize,
    #[pyo3(get)]
    match_length: usize,
    #[pyo3(get)]
    used_fuzzy_match: bool,
    #[pyo3(get)]
    content_for_replacement: String,
}

#[pymethods]
impl PyFuzzyMatchResult {
    fn __repr__(&self) -> String {
        format!(
            "FuzzyMatchResult(found={}, index={}, match_length={}, used_fuzzy_match={})",
            self.found, self.index, self.match_length, self.used_fuzzy_match
        )
    }
}

impl From<FuzzyMatchResult> for PyFuzzyMatchResult {
    fn from(r: FuzzyMatchResult) -> Self {
        PyFuzzyMatchResult {
            found: r.found,
            index: r.index,
            match_length: r.match_length,
            used_fuzzy_match: r.used_fuzzy_match,
            content_for_replacement: r.content_for_replacement,
        }
    }
}

/// Python-exposed diff result
#[pyclass(name = "DiffResult")]
#[derive(Clone)]
pub struct PyDiffResult {
    #[pyo3(get)]
    diff: String,
    #[pyo3(get)]
    first_changed_line: Option<usize>,
}

#[pymethods]
impl PyDiffResult {
    fn __repr__(&self) -> String {
        format!(
            "DiffResult(first_changed_line={:?}, diff_lines={})",
            self.first_changed_line,
            self.diff.lines().count()
        )
    }
}

impl From<DiffResult> for PyDiffResult {
    fn from(r: DiffResult) -> Self {
        PyDiffResult {
            diff: r.diff,
            first_changed_line: r.first_changed_line,
        }
    }
}

/// Python-exposed structured format enum
#[pyclass(name = "StructuredFormat")]
#[derive(Clone)]
pub struct PyStructuredFormat(structured::StructuredFormat);

#[pymethods]
impl PyStructuredFormat {
    #[new]
    #[pyo3(signature = (name))]
    fn new(name: &str) -> PyResult<Self> {
        structured::StructuredFormat::from_str(name)
            .map(PyStructuredFormat)
            .ok_or_else(|| {
                PyValueError::new_err(format!(
                    "Unknown format '{}'. Valid formats: json, jsonc, json5, toml, yaml",
                    name
                ))
            })
    }

    #[staticmethod]
    fn json() -> Self {
        PyStructuredFormat(structured::StructuredFormat::Json)
    }

    #[staticmethod]
    fn jsonc() -> Self {
        PyStructuredFormat(structured::StructuredFormat::Jsonc)
    }

    #[staticmethod]
    fn json5() -> Self {
        PyStructuredFormat(structured::StructuredFormat::Json5)
    }

    #[staticmethod]
    fn toml() -> Self {
        PyStructuredFormat(structured::StructuredFormat::Toml)
    }

    #[staticmethod]
    fn yaml() -> Self {
        PyStructuredFormat(structured::StructuredFormat::Yaml)
    }

    fn __str__(&self) -> &'static str {
        self.0.as_str()
    }

    fn __repr__(&self) -> String {
        format!("StructuredFormat('{}')", self.0.as_str())
    }
}

/// Python-exposed structured edit result
#[pyclass(name = "StructuredEditResult")]
#[derive(Clone)]
pub struct PyStructuredEditResult {
    #[pyo3(get)]
    content: String,
    #[pyo3(get)]
    format: String,
    #[pyo3(get)]
    key_path: String,
    #[pyo3(get)]
    old_value: Option<String>,
    #[pyo3(get)]
    new_value: String,
    #[pyo3(get)]
    line_ending: Option<String>,
}

#[pymethods]
impl PyStructuredEditResult {
    fn __repr__(&self) -> String {
        format!(
            "StructuredEditResult(format='{}', key_path='{}', old_value={:?})",
            self.format, self.key_path, self.old_value
        )
    }
}

impl From<structured::StructuredEditResult> for PyStructuredEditResult {
    fn from(r: structured::StructuredEditResult) -> Self {
        PyStructuredEditResult {
            content: r.content,
            format: r.format.as_str().to_string(),
            key_path: r.key_path,
            old_value: r.old_value,
            new_value: r.new_value,
            line_ending: r.line_ending.map(|le| le.as_str().to_string()),
        }
    }
}

/// Convert EditError to Python exception
fn edit_error_to_py(e: EditError) -> PyErr {
    match e {
        EditError::NotFound(s) => PyValueError::new_err(format!(
            "Could not find the text to replace. The old text must match exactly including all whitespace and newlines.\nSearched for: {}",
            if s.len() > 100 { format!("{}...", &s[..100]) } else { s }
        )),
        EditError::MultipleFound(n) => PyValueError::new_err(format!(
            "Found {} occurrences of the text. The text must be unique. Please provide more context to make it unique, or use multiple=True to replace all.",
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
    detect_line_endings(content).map(|le| le.name().to_string())
}

/// Edit a file using fuzzy matching with diff output
///
/// This function provides fuzzy matching capabilities similar to the TypeScript implementation.
/// It tries exact match first, then falls back to fuzzy matching (normalizing Unicode
/// characters like smart quotes, dashes, and special spaces).
///
/// Args:
///     path: Path to the file to edit
///     old_text: The text to find (will try exact match, then fuzzy)
///     new_text: The replacement text
///     multiple: If True, replace all occurrences
///     encoding: File encoding (default: "utf-8")
///     dry_run: If True, don't actually modify the file
///
/// Returns:
///     EditResultWithDiff with the modified content, diff, and match info
#[pyfunction]
#[pyo3(signature = (path, old_text, new_text, multiple=false, encoding="utf-8", dry_run=false))]
fn edit_fuzzy(
    path: &str,
    old_text: &str,
    new_text: &str,
    multiple: bool,
    encoding: &str,
    dry_run: bool,
) -> PyResult<PyEditResultWithDiff> {
    let enc = Encoding::from_str(encoding).map_err(edit_error_to_py)?;

    let options = ReplaceOptions {
        multiple,
        ignore_whitespace: false,
        encoding: enc,
        dry_run,
    };

    edit_file_fuzzy(Path::new(path), old_text, new_text, &options)
        .map(PyEditResultWithDiff::from)
        .map_err(edit_error_to_py)
}

/// Find text using fuzzy matching
///
/// Tries exact match first, then falls back to fuzzy matching
/// (normalizing trailing whitespace, Unicode quotes, dashes, and special spaces).
///
/// Args:
///     content: The text content to search in
///     old_text: The text to find
///
/// Returns:
///     FuzzyMatchResult with match information
#[pyfunction]
fn fuzzy_find(content: &str, old_text: &str) -> PyFuzzyMatchResult {
    fuzzy_find_text(content, old_text).into()
}

/// Normalize text for fuzzy matching
///
/// Applies the following transformations:
/// - Strip trailing whitespace from each line
/// - Normalize smart quotes to ASCII equivalents
/// - Normalize Unicode dashes/hyphens to ASCII hyphen
/// - Normalize special Unicode spaces to regular space
///
/// Args:
///     text: The text to normalize
///
/// Returns:
///     The normalized text
#[pyfunction]
fn normalize_fuzzy(text: &str) -> String {
    normalize_for_fuzzy_match(text)
}

/// Generate a unified diff between two strings
///
/// Args:
///     old_content: The original content
///     new_content: The new content
///     context_lines: Number of context lines to include (default: 4)
///
/// Returns:
///     DiffResult with the diff string and first changed line
#[pyfunction]
#[pyo3(signature = (old_content, new_content, context_lines=4))]
fn diff(old_content: &str, new_content: &str, context_lines: usize) -> PyDiffResult {
    generate_diff(old_content, new_content, context_lines).into()
}

/// Strip UTF-8 BOM from content if present
///
/// Args:
///     content: The text content
///
/// Returns:
///     Tuple of (bom, text) where bom is the BOM string (or empty) and text is the content without BOM
#[pyfunction]
fn strip_bom_py(content: &str) -> (String, String) {
    let result = strip_bom(content);
    (result.bom, result.text)
}

/// Normalize line endings to LF
///
/// Args:
///     text: The text to normalize
///
/// Returns:
///     Text with all line endings converted to LF
#[pyfunction]
fn normalize_line_endings(text: &str) -> String {
    normalize_to_lf(text)
}

/// Edit a structured file (JSON, JSONC, JSON5, TOML, YAML) at a key path
///
/// Args:
///     path: Path to the file to edit
///     key_path: Dot-separated key path (e.g., "settings.port", "items[0].name")
///     new_value: The new value (will be parsed as appropriate for the format)
///     format: Optional format override ("json", "jsonc", "json5", "toml", "yaml")
///     encoding: File encoding (default: "utf-8")
///     dry_run: If True, don't actually modify the file
///
/// Returns:
///     StructuredEditResult with the modified content and metadata
#[pyfunction]
#[pyo3(signature = (path, key_path, new_value, format=None, encoding="utf-8", dry_run=false))]
fn edit_structured_file(
    path: &str,
    key_path: &str,
    new_value: &str,
    format: Option<&str>,
    encoding: &str,
    dry_run: bool,
) -> PyResult<PyStructuredEditResult> {
    let enc = Encoding::from_str(encoding).map_err(edit_error_to_py)?;

    let fmt = match format {
        Some(f) => Some(structured::StructuredFormat::from_str(f).ok_or_else(|| {
            PyValueError::new_err(format!(
                "Unknown format '{}'. Valid formats: json, jsonc, json5, toml, yaml",
                f
            ))
        })?),
        None => None,
    };

    structured::edit_structured(Path::new(path), key_path, new_value, fmt, enc, dry_run)
        .map(PyStructuredEditResult::from)
        .map_err(edit_error_to_py)
}

/// Edit structured content in-memory (without file I/O)
///
/// Args:
///     content: The content to edit
///     key_path: Dot-separated key path (e.g., "settings.port", "items[0].name")
///     new_value: The new value (will be parsed as appropriate for the format)
///     format: Format of the content ("json", "jsonc", "json5", "toml", "yaml")
///
/// Returns:
///     Tuple of (new_content, old_value) where old_value may be None
#[pyfunction]
fn edit_structured_string(
    content: &str,
    key_path: &str,
    new_value: &str,
    format: &str,
) -> PyResult<(String, Option<String>)> {
    let fmt = structured::StructuredFormat::from_str(format).ok_or_else(|| {
        PyValueError::new_err(format!(
            "Unknown format '{}'. Valid formats: json, jsonc, json5, toml, yaml",
            format
        ))
    })?;

    structured::edit_structured_content(content, key_path, new_value, fmt).map_err(edit_error_to_py)
}

/// Python module definition
#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyEncoding>()?;
    m.add_class::<PyEditResult>()?;
    m.add_class::<PyEditResultWithDiff>()?;
    m.add_class::<PyFuzzyMatchResult>()?;
    m.add_class::<PyDiffResult>()?;
    m.add_class::<PyStructuredFormat>()?;
    m.add_class::<PyStructuredEditResult>()?;
    m.add_function(wrap_pyfunction!(edit, m)?)?;
    m.add_function(wrap_pyfunction!(edit_fuzzy, m)?)?;
    m.add_function(wrap_pyfunction!(replace_in_string, m)?)?;
    m.add_function(wrap_pyfunction!(read, m)?)?;
    m.add_function(wrap_pyfunction!(detect_line_ending, m)?)?;
    m.add_function(wrap_pyfunction!(fuzzy_find, m)?)?;
    m.add_function(wrap_pyfunction!(normalize_fuzzy, m)?)?;
    m.add_function(wrap_pyfunction!(diff, m)?)?;
    m.add_function(wrap_pyfunction!(strip_bom_py, m)?)?;
    m.add_function(wrap_pyfunction!(normalize_line_endings, m)?)?;
    m.add_function(wrap_pyfunction!(edit_structured_file, m)?)?;
    m.add_function(wrap_pyfunction!(edit_structured_string, m)?)?;
    Ok(())
}
