//! Core FEdit replacement engine.
//!
//! Provides the complete algorithm for search-and-replace operations with:
//! - Exact-match and whitespace-insensitive matching
//! - Line ending detection and preservation (LF/CRLF)
//! - Multiple encoding support
//! - Atomic file writes

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

use regex::Regex;

/// Supported file encodings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Encoding {
    #[default]
    Utf8,
    Utf16Le,
    Utf16Be,
    Iso8859_1,
    Windows1252,
}

impl Encoding {
    /// Parse encoding from string (case-insensitive)
    pub fn from_str(s: &str) -> Result<Self, EditError> {
        match s.to_lowercase().as_str() {
            "utf-8" | "utf8" => Ok(Encoding::Utf8),
            "utf-16" | "utf16" | "utf-16le" | "utf16le" => Ok(Encoding::Utf16Le),
            "utf-16be" | "utf16be" => Ok(Encoding::Utf16Be),
            "iso-8859-1" | "iso88591" | "latin1" => Ok(Encoding::Iso8859_1),
            "windows-1252" | "windows1252" | "cp1252" => Ok(Encoding::Windows1252),
            _ => Err(EditError::EncodingError(format!("Unknown encoding: {}", s))),
        }
    }

    /// Get encoding name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Encoding::Utf8 => "utf-8",
            Encoding::Utf16Le => "utf-16le",
            Encoding::Utf16Be => "utf-16be",
            Encoding::Iso8859_1 => "iso-8859-1",
            Encoding::Windows1252 => "windows-1252",
        }
    }
}

/// Detected line ending style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    Lf,   // Unix: \n
    CrLf, // Windows: \r\n
}

impl LineEnding {
    pub fn as_str(&self) -> &'static str {
        match self {
            LineEnding::Lf => "\n",
            LineEnding::CrLf => "\r\n",
        }
    }
}

/// Options for replacement operations
#[derive(Debug, Clone, Default)]
pub struct ReplaceOptions {
    /// Replace all occurrences (default: single match only)
    pub multiple: bool,
    /// Whitespace-insensitive matching
    pub ignore_whitespace: bool,
    /// File encoding to use
    pub encoding: Encoding,
    /// Dry run - don't actually modify the file
    pub dry_run: bool,
}

/// Result of a replacement operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditResult {
    /// The modified content
    pub content: String,
    /// Number of replacements made
    pub replacements: usize,
    /// Detected line ending style (if any)
    pub line_ending: Option<LineEnding>,
}

/// Errors that can occur during edit operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditError {
    /// Search string not found
    NotFound(String),
    /// Multiple matches found but multiple mode not enabled
    MultipleFound(usize),
    /// I/O error
    IoError(String),
    /// Encoding/decoding error
    EncodingError(String),
    /// Invalid key path (for structured mode)
    InvalidKeyPath(String),
    /// Key not found in document (for structured mode)
    KeyNotFound(String),
    /// Generic error
    Other(String),
}

impl std::fmt::Display for EditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditError::NotFound(s) => write!(f, "No matches found for: {}", s),
            EditError::MultipleFound(n) => {
                write!(
                    f,
                    "Multiple matches found ({}); use --multiple to replace all",
                    n
                )
            }
            EditError::IoError(msg) => write!(f, "IO error: {}", msg),
            EditError::EncodingError(msg) => write!(f, "Encoding error: {}", msg),
            EditError::InvalidKeyPath(p) => write!(f, "Invalid key path: {}", p),
            EditError::KeyNotFound(p) => write!(f, "Key not found: {}", p),
            EditError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for EditError {}

/// Detect line ending style from raw bytes
pub fn detect_line_endings(content: &[u8]) -> Option<LineEnding> {
    let crlf_count = content.windows(2).filter(|w| w == b"\r\n").count();
    let lf_only_count = content
        .iter()
        .filter(|&&b| b == b'\n')
        .count()
        .saturating_sub(crlf_count);

    if crlf_count == 0 && lf_only_count == 0 {
        None
    } else if crlf_count >= lf_only_count {
        Some(LineEnding::CrLf)
    } else {
        Some(LineEnding::Lf)
    }
}

/// Decode bytes to string using the specified encoding
pub fn decode_content(bytes: &[u8], encoding: Encoding) -> Result<String, EditError> {
    match encoding {
        Encoding::Utf8 => String::from_utf8(bytes.to_vec())
            .map_err(|e| EditError::EncodingError(format!("UTF-8 decode error: {}", e))),
        Encoding::Utf16Le => {
            if bytes.len() % 2 != 0 {
                return Err(EditError::EncodingError(
                    "Invalid UTF-16LE: odd byte count".into(),
                ));
            }
            let u16_iter = bytes
                .chunks_exact(2)
                .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]));
            String::from_utf16(&u16_iter.collect::<Vec<_>>())
                .map_err(|e| EditError::EncodingError(format!("UTF-16LE decode error: {}", e)))
        }
        Encoding::Utf16Be => {
            if bytes.len() % 2 != 0 {
                return Err(EditError::EncodingError(
                    "Invalid UTF-16BE: odd byte count".into(),
                ));
            }
            let u16_iter = bytes
                .chunks_exact(2)
                .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]));
            String::from_utf16(&u16_iter.collect::<Vec<_>>())
                .map_err(|e| EditError::EncodingError(format!("UTF-16BE decode error: {}", e)))
        }
        Encoding::Iso8859_1 => {
            // ISO-8859-1 maps directly to Unicode code points 0-255
            Ok(bytes.iter().map(|&b| b as char).collect())
        }
        Encoding::Windows1252 => {
            // Windows-1252 is similar to ISO-8859-1 but has different mappings for 0x80-0x9F
            const CP1252_MAP: [char; 32] = [
                '\u{20AC}', '\u{0081}', '\u{201A}', '\u{0192}', '\u{201E}', '\u{2026}', '\u{2020}',
                '\u{2021}', '\u{02C6}', '\u{2030}', '\u{0160}', '\u{2039}', '\u{0152}', '\u{008D}',
                '\u{017D}', '\u{008F}', '\u{0090}', '\u{2018}', '\u{2019}', '\u{201C}', '\u{201D}',
                '\u{2022}', '\u{2013}', '\u{2014}', '\u{02DC}', '\u{2122}', '\u{0161}', '\u{203A}',
                '\u{0153}', '\u{009D}', '\u{017E}', '\u{0178}',
            ];
            Ok(bytes
                .iter()
                .map(|&b| {
                    if (0x80..=0x9F).contains(&b) {
                        CP1252_MAP[(b - 0x80) as usize]
                    } else {
                        b as char
                    }
                })
                .collect())
        }
    }
}

/// Encode string to bytes using the specified encoding
pub fn encode_content(content: &str, encoding: Encoding) -> Result<Vec<u8>, EditError> {
    match encoding {
        Encoding::Utf8 => Ok(content.as_bytes().to_vec()),
        Encoding::Utf16Le => {
            let mut bytes = Vec::with_capacity(content.len() * 2);
            for c in content.encode_utf16() {
                bytes.extend_from_slice(&c.to_le_bytes());
            }
            Ok(bytes)
        }
        Encoding::Utf16Be => {
            let mut bytes = Vec::with_capacity(content.len() * 2);
            for c in content.encode_utf16() {
                bytes.extend_from_slice(&c.to_be_bytes());
            }
            Ok(bytes)
        }
        Encoding::Iso8859_1 => content
            .chars()
            .map(|c| {
                let cp = c as u32;
                if cp <= 255 {
                    Ok(cp as u8)
                } else {
                    Err(EditError::EncodingError(format!(
                        "Character '{}' cannot be encoded in ISO-8859-1",
                        c
                    )))
                }
            })
            .collect(),
        Encoding::Windows1252 => {
            // Reverse mapping for Windows-1252
            content
                .chars()
                .map(|c| {
                    let cp = c as u32;
                    if cp <= 127 || (160..=255).contains(&cp) {
                        Ok(cp as u8)
                    } else {
                        // Check special CP1252 characters
                        match c {
                            '\u{20AC}' => Ok(0x80),
                            '\u{201A}' => Ok(0x82),
                            '\u{0192}' => Ok(0x83),
                            '\u{201E}' => Ok(0x84),
                            '\u{2026}' => Ok(0x85),
                            '\u{2020}' => Ok(0x86),
                            '\u{2021}' => Ok(0x87),
                            '\u{02C6}' => Ok(0x88),
                            '\u{2030}' => Ok(0x89),
                            '\u{0160}' => Ok(0x8A),
                            '\u{2039}' => Ok(0x8B),
                            '\u{0152}' => Ok(0x8C),
                            '\u{017D}' => Ok(0x8E),
                            '\u{2018}' => Ok(0x91),
                            '\u{2019}' => Ok(0x92),
                            '\u{201C}' => Ok(0x93),
                            '\u{201D}' => Ok(0x94),
                            '\u{2022}' => Ok(0x95),
                            '\u{2013}' => Ok(0x96),
                            '\u{2014}' => Ok(0x97),
                            '\u{02DC}' => Ok(0x98),
                            '\u{2122}' => Ok(0x99),
                            '\u{0161}' => Ok(0x9A),
                            '\u{203A}' => Ok(0x9B),
                            '\u{0153}' => Ok(0x9C),
                            '\u{017E}' => Ok(0x9E),
                            '\u{0178}' => Ok(0x9F),
                            _ => Err(EditError::EncodingError(format!(
                                "Character '{}' cannot be encoded in Windows-1252",
                                c
                            ))),
                        }
                    }
                })
                .collect()
        }
    }
}

/// Build a regex pattern for whitespace-insensitive matching
fn build_whitespace_pattern(search: &str) -> String {
    let mut pattern = String::new();
    let mut chars = search.chars().peekable();

    while let Some(c) = chars.next() {
        if c.is_whitespace() {
            // Consume all consecutive whitespace
            while chars.peek().is_some_and(|c| c.is_whitespace()) {
                chars.next();
            }
            pattern.push_str(r"\s+");
        } else {
            // Escape regex special characters
            match c {
                '\\' | '.' | '+' | '*' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '|' | '^'
                | '$' => {
                    pattern.push('\\');
                    pattern.push(c);
                }
                _ => pattern.push(c),
            }
        }
    }

    pattern
}

/// Find all matches in content
fn find_matches(content: &str, search: &str, ignore_whitespace: bool) -> Vec<(usize, usize)> {
    let mut matches = Vec::new();

    if ignore_whitespace {
        let pattern = build_whitespace_pattern(search);
        if let Ok(re) = Regex::new(&pattern) {
            for m in re.find_iter(content) {
                matches.push((m.start(), m.end()));
            }
        }
    } else {
        let mut start = 0;
        while let Some(pos) = content[start..].find(search) {
            let abs_pos = start + pos;
            matches.push((abs_pos, abs_pos + search.len()));
            start = abs_pos + search.len();
        }
    }

    matches
}

/// Perform replacement in content string
pub fn replace_in_content(
    content: &str,
    search: &str,
    replace: &str,
    options: &ReplaceOptions,
) -> Result<EditResult, EditError> {
    // Find all matches
    let matches = find_matches(content, search, options.ignore_whitespace);
    let count = matches.len();

    if count == 0 {
        return Err(EditError::NotFound(search.to_string()));
    }

    if count > 1 && !options.multiple {
        return Err(EditError::MultipleFound(count));
    }

    // Build the new content by replacing matches
    let mut new_content = String::with_capacity(content.len());
    let mut last_end = 0;

    let matches_to_use = if options.multiple {
        &matches[..]
    } else {
        &matches[..1]
    };

    for (start, end) in matches_to_use {
        new_content.push_str(&content[last_end..*start]);
        new_content.push_str(replace);
        last_end = *end;
    }
    new_content.push_str(&content[last_end..]);

    Ok(EditResult {
        content: new_content,
        replacements: matches_to_use.len(),
        line_ending: None,
    })
}

/// Read a file and decode its contents
pub fn read_file(
    path: &Path,
    encoding: Encoding,
) -> Result<(String, Option<LineEnding>), EditError> {
    let mut file = File::open(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            EditError::IoError(format!("No such file: {}", path.display()))
        } else {
            EditError::IoError(format!("Failed to open file: {}", e))
        }
    })?;

    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|e| EditError::IoError(format!("Failed to read file: {}", e)))?;

    let line_ending = detect_line_endings(&bytes);
    let content = decode_content(&bytes, encoding)?;

    Ok((content, line_ending))
}

/// Write content to a file atomically (temp file + rename)
pub fn write_file_atomic(
    path: &Path,
    content: &str,
    encoding: Encoding,
    line_ending: Option<LineEnding>,
) -> Result<(), EditError> {
    // Normalize line endings in replacement if needed
    let final_content = if let Some(le) = line_ending {
        match le {
            LineEnding::CrLf => {
                // Normalize \n to \r\n (but not already \r\n)
                let mut result = String::with_capacity(content.len());
                let mut chars = content.chars().peekable();
                while let Some(c) = chars.next() {
                    if c == '\r' && chars.peek() == Some(&'\n') {
                        result.push('\r');
                        result.push('\n');
                        chars.next();
                    } else if c == '\n' {
                        result.push('\r');
                        result.push('\n');
                    } else {
                        result.push(c);
                    }
                }
                result
            }
            LineEnding::Lf => {
                // Normalize \r\n to \n
                content.replace("\r\n", "\n")
            }
        }
    } else {
        content.to_string()
    };

    let bytes = encode_content(&final_content, encoding)?;

    // Get the directory for the temp file
    let dir = path.parent().unwrap_or(Path::new("."));

    // Create temp file in the same directory
    let temp_path = dir.join(format!(
        ".fedit.tmp.{}.{}",
        std::process::id(),
        path.file_name().and_then(|n| n.to_str()).unwrap_or("file")
    ));

    // Write to temp file
    let mut temp_file = File::create(&temp_path)
        .map_err(|e| EditError::IoError(format!("Failed to create temp file: {}", e)))?;

    temp_file
        .write_all(&bytes)
        .map_err(|e| EditError::IoError(format!("Failed to write temp file: {}", e)))?;

    temp_file
        .sync_all()
        .map_err(|e| EditError::IoError(format!("Failed to sync temp file: {}", e)))?;

    drop(temp_file);

    // Atomic rename
    fs::rename(&temp_path, path).map_err(|e| {
        // Clean up temp file on failure
        let _ = fs::remove_file(&temp_path);
        EditError::IoError(format!("Failed to rename temp file: {}", e))
    })?;

    Ok(())
}

/// High-level function to perform a file edit operation
pub fn edit_file(
    path: &Path,
    search: &str,
    replace: &str,
    options: &ReplaceOptions,
) -> Result<EditResult, EditError> {
    // Read file
    let (content, line_ending) = read_file(path, options.encoding)?;

    // Process replacement string (handle \n escape sequences)
    let processed_replace = replace.replace("\\n", "\n");

    // Perform replacement
    let mut result = replace_in_content(&content, search, &processed_replace, options)?;
    result.line_ending = line_ending;

    // Write back (unless dry run)
    if !options.dry_run {
        write_file_atomic(path, &result.content, options.encoding, line_ending)?;
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_line_endings_lf() {
        assert_eq!(detect_line_endings(b"hello\nworld\n"), Some(LineEnding::Lf));
    }

    #[test]
    fn test_detect_line_endings_crlf() {
        assert_eq!(
            detect_line_endings(b"hello\r\nworld\r\n"),
            Some(LineEnding::CrLf)
        );
    }

    #[test]
    fn test_detect_line_endings_none() {
        assert_eq!(detect_line_endings(b"hello world"), None);
    }

    #[test]
    fn test_replace_single() {
        let opts = ReplaceOptions::default();
        let result = replace_in_content("hello world", "world", "rust", &opts).unwrap();
        assert_eq!(result.content, "hello rust");
        assert_eq!(result.replacements, 1);
    }

    #[test]
    fn test_replace_multiple_error() {
        let opts = ReplaceOptions::default();
        let result = replace_in_content("foo bar foo", "foo", "baz", &opts);
        assert!(matches!(result, Err(EditError::MultipleFound(2))));
    }

    #[test]
    fn test_replace_multiple_allowed() {
        let opts = ReplaceOptions {
            multiple: true,
            ..Default::default()
        };
        let result = replace_in_content("foo bar foo", "foo", "baz", &opts).unwrap();
        assert_eq!(result.content, "baz bar baz");
        assert_eq!(result.replacements, 2);
    }

    #[test]
    fn test_whitespace_insensitive() {
        let opts = ReplaceOptions {
            ignore_whitespace: true,
            ..Default::default()
        };
        let result = replace_in_content("hello    world", "hello world", "goodbye", &opts).unwrap();
        assert_eq!(result.content, "goodbye");
    }

    #[test]
    fn test_encoding_parse() {
        assert_eq!(Encoding::from_str("utf-8").unwrap(), Encoding::Utf8);
        assert_eq!(Encoding::from_str("UTF-8").unwrap(), Encoding::Utf8);
        assert_eq!(Encoding::from_str("utf-16").unwrap(), Encoding::Utf16Le);
        assert!(Encoding::from_str("invalid").is_err());
    }
}
