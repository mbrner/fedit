// Lightweight Rust API for FEdit core replacement engine.
// Exposes a small, embeddable surface for usage from other Rust projects.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditResult {
    pub content: String,
    pub replacements: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplaceOptions {
    pub multiple: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditError {
    NotFound(String),       // search string not found
    MultipleFound(usize),   // multiple matches found
    IoError(String),        // I/O related error context
    EncodingError(String),  // encoding related issue
    InvalidKeyPath(String), // invalid structured mode path
    KeyNotFound(String),    // path not found in document
    Other(String),          // generic error
}

impl fmt::Display for EditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EditError::NotFound(p) => write!(f, "No matches found for: {}", p),
            EditError::MultipleFound(n) => write!(f, "Multiple matches found ({})", n),
            EditError::IoError(msg) => write!(f, "IO error: {}", msg),
            EditError::EncodingError(msg) => write!(f, "Encoding error: {}", msg),
            EditError::InvalidKeyPath(p) => write!(f, "Invalid key path: {}", p),
            EditError::KeyNotFound(p) => write!(f, "Key not found: {}", p),
            EditError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for EditError {}

/// Core in-memory replacement function.
/// - If zero matches: Err(EditError::NotFound(search.to_string()))
/// - If multiple matches and options.multiple is false: Err(EditError::MultipleFound(count))
/// - If options.multiple is true: replace all occurrences and return count as replacements
pub fn replace_in_content(
    content: &str,
    search: &str,
    replace: &str,
    options: &ReplaceOptions,
) -> Result<EditResult, EditError> {
    // Count occurrences of `search` in `content` without allocating excessively
    let mut count: usize = 0;
    let mut idx: usize = 0;
    while let Some(pos) = content[idx..].find(search) {
        count += 1;
        idx += pos + search.len();
    }

    if count == 0 {
        return Err(EditError::NotFound(search.to_string()));
    }
    if count > 1 && !options.multiple {
        return Err(EditError::MultipleFound(count));
    }

    let new_content = if options.multiple {
        content.replace(search, replace)
    } else {
        // replace only the first occurrence
        let mut s = content.to_string();
        if let Some(pos) = s.find(search) {
            s.replace_range(pos..pos + search.len(), replace);
        }
        s
    };

    Ok(EditResult {
        content: new_content,
        replacements: if options.multiple { count } else { 1 },
    })
}
