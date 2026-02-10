//! Core Rust library API for string replacements.
//!
//! This module provides a small, dependency-free API that can be used by
//! applications embedding FEdit functionality without relying on the Python
//! bindings.

/// Result type returned by library replacement function.
#[derive(Debug, Clone)]
pub struct EditResult {
    /// The modified content after replacements.
    pub content: String,
    /// The number of replacements performed.
    pub replacements: usize,
}

/// Options to customize replacement behavior.
#[derive(Debug, Clone)]
pub struct ReplaceOptions {
    /// When true, replace all occurrences. When false, replace only the first match.
    pub multiple: bool,
}

/// Errors that can occur during replacement.
#[derive(Debug, Clone)]
pub enum EditError {
    /// No matches were found for the search string.
    NoMatches(String),
    /// Multiple matches found. The count is provided.
    MultipleMatches(usize),
    /// Other error with a message.
    Other(String),
}

impl std::fmt::Display for EditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditError::NoMatches(s) => write!(f, "No matches found for: {}", s),
            EditError::MultipleMatches(n) => write!(f, "Multiple matches found ({})", n),
            EditError::Other(s) => write!(f, "{}", s),
        }
    }
}

impl std::error::Error for EditError {}

/// Perform replacements in the provided content.
///
/// This function operates purely on the input content string and does not perform any I/O.
/// - If there are zero matches, returns Err(EditError::NoMatches(...)).
/// - If there are multiple matches and `options.multiple` is false, returns
///   Err(EditError::MultipleMatches(count)).
/// - If `options.multiple` is true, all matches are replaced and Ok is returned with
///   the total replacements performed.
///
/// Basic example:
///
/// ```rust
/// use fedit::EditResult;
/// use fedit::ReplaceOptions;
/// use fedit::replace_in_content;
///
/// let content = "hello world, hello rust";
/// let opts = ReplaceOptions { multiple: true };
/// let res = replace_in_content(content, "hello", "hi", &opts).unwrap();
/// assert_eq!(res.replacements, 2);
/// assert_eq!(res.content, "hi world, hi rust");
/// ```
pub fn replace_in_content(
    content: &str,
    search: &str,
    replace: &str,
    options: &ReplaceOptions,
) -> Result<EditResult, EditError> {
    if search.is_empty() {
        return Err(EditError::Other(
            "search string must not be empty".to_string(),
        ));
    }
    // Collect all non-overlapping indices of matches
    let mut indices = Vec::new();
    let mut start = 0usize;
    while let Some(pos) = content[start..].find(search) {
        let absolute = start + pos;
        indices.push(absolute);
        start = absolute + search.len();
        // Prevent infinite loop on zero-length matches
        if search.is_empty() {
            break;
        }
    }
    let count = indices.len();
    if count == 0 {
        return Err(EditError::NoMatches(search.to_string()));
    }
    // If not replacing all, and there are multiple matches, error
    if count > 1 && !options.multiple {
        return Err(EditError::MultipleMatches(count));
    }
    let new_content = if options.multiple {
        // Replace all occurrences
        let mut out = String::with_capacity(content.len());
        let mut last = 0usize;
        for idx in indices {
            out.push_str(&content[last..idx]);
            out.push_str(replace);
            last = idx + search.len();
        }
        out.push_str(&content[last..]);
        out
    } else {
        // Replace only the first occurrence
        if let Some(first) = content.find(search) {
            let mut out = String::with_capacity(content.len());
            out.push_str(&content[..first]);
            out.push_str(replace);
            out.push_str(&content[first + search.len()..]);
            out
        } else {
            // Fallback; should not happen if count > 0
            content.to_string()
        }
    };
    let replacements_made = if options.multiple { count } else { 1 };
    Ok(EditResult {
        content: new_content,
        replacements: replacements_made,
    })
}
