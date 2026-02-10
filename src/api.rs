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

/// UTF-8 BOM constant
pub const UTF8_BOM: &str = "\u{FEFF}";
pub const UTF8_BOM_BYTES: &[u8] = &[0xEF, 0xBB, 0xBF];

/// Result of stripping BOM from content
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BomStripped {
    /// The BOM that was found (empty string if none)
    pub bom: String,
    /// The content without the BOM
    pub text: String,
}

/// Strip UTF-8 BOM from string content if present
pub fn strip_bom(content: &str) -> BomStripped {
    if content.starts_with(UTF8_BOM) {
        BomStripped {
            bom: UTF8_BOM.to_string(),
            text: content[UTF8_BOM.len()..].to_string(),
        }
    } else {
        BomStripped {
            bom: String::new(),
            text: content.to_string(),
        }
    }
}

/// Strip UTF-8 BOM from raw bytes if present
pub fn strip_bom_bytes(bytes: &[u8]) -> (bool, &[u8]) {
    if bytes.starts_with(UTF8_BOM_BYTES) {
        (true, &bytes[UTF8_BOM_BYTES.len()..])
    } else {
        (false, bytes)
    }
}

impl LineEnding {
    pub fn as_str(&self) -> &'static str {
        match self {
            LineEnding::Lf => "\n",
            LineEnding::CrLf => "\r\n",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            LineEnding::Lf => "lf",
            LineEnding::CrLf => "crlf",
        }
    }
}

/// Result of fuzzy matching operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuzzyMatchResult {
    /// Whether a match was found
    pub found: bool,
    /// The index where the match starts (in the content used for replacement)
    pub index: usize,
    /// Length of the matched text
    pub match_length: usize,
    /// Whether fuzzy matching was used (false = exact match)
    pub used_fuzzy_match: bool,
    /// The content to use for replacement operations
    /// When exact match: original content. When fuzzy match: normalized content.
    pub content_for_replacement: String,
}

/// Normalize text for fuzzy matching.
/// Applies progressive transformations:
/// - Strip trailing whitespace from each line
/// - Normalize smart quotes to ASCII equivalents
/// - Normalize Unicode dashes/hyphens to ASCII hyphen
/// - Normalize special Unicode spaces to regular space
pub fn normalize_for_fuzzy_match(text: &str) -> String {
    text.lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
        .chars()
        .map(|c| match c {
            // Smart single quotes → '
            '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{201B}' => '\'',
            // Smart double quotes → "
            '\u{201C}' | '\u{201D}' | '\u{201E}' | '\u{201F}' => '"',
            // Various dashes/hyphens → -
            // U+2010 hyphen, U+2011 non-breaking hyphen, U+2012 figure dash,
            // U+2013 en-dash, U+2014 em-dash, U+2015 horizontal bar, U+2212 minus
            '\u{2010}' | '\u{2011}' | '\u{2012}' | '\u{2013}' | '\u{2014}' | '\u{2015}'
            | '\u{2212}' => '-',
            // Special spaces → regular space
            // U+00A0 NBSP, U+2002-U+200A various spaces, U+202F narrow NBSP,
            // U+205F medium math space, U+3000 ideographic space
            '\u{00A0}' | '\u{2002}' | '\u{2003}' | '\u{2004}' | '\u{2005}' | '\u{2006}'
            | '\u{2007}' | '\u{2008}' | '\u{2009}' | '\u{200A}' | '\u{202F}' | '\u{205F}'
            | '\u{3000}' => ' ',
            _ => c,
        })
        .collect()
}

/// Find oldText in content, trying exact match first, then fuzzy match.
/// When fuzzy matching is used, the returned content_for_replacement is the
/// fuzzy-normalized version of the content.
pub fn fuzzy_find_text(content: &str, old_text: &str) -> FuzzyMatchResult {
    // Try exact match first
    if let Some(index) = content.find(old_text) {
        return FuzzyMatchResult {
            found: true,
            index,
            match_length: old_text.len(),
            used_fuzzy_match: false,
            content_for_replacement: content.to_string(),
        };
    }

    // Try fuzzy match - work entirely in normalized space
    let fuzzy_content = normalize_for_fuzzy_match(content);
    let fuzzy_old_text = normalize_for_fuzzy_match(old_text);

    if let Some(index) = fuzzy_content.find(&fuzzy_old_text) {
        return FuzzyMatchResult {
            found: true,
            index,
            match_length: fuzzy_old_text.len(),
            used_fuzzy_match: true,
            content_for_replacement: fuzzy_content,
        };
    }

    FuzzyMatchResult {
        found: false,
        index: 0,
        match_length: 0,
        used_fuzzy_match: false,
        content_for_replacement: content.to_string(),
    }
}

/// Count occurrences of a pattern in content using fuzzy matching
pub fn count_fuzzy_occurrences(content: &str, pattern: &str) -> usize {
    let fuzzy_content = normalize_for_fuzzy_match(content);
    let fuzzy_pattern = normalize_for_fuzzy_match(pattern);

    if fuzzy_pattern.is_empty() {
        return 0;
    }

    fuzzy_content.matches(&fuzzy_pattern).count()
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
            EditError::NotFound(s) => write!(
                f,
                "Could not find the text to replace. The old text must match exactly including all whitespace and newlines.\nSearched for: {}",
                truncate_for_display(s, 100)
            ),
            EditError::MultipleFound(n) => {
                write!(
                    f,
                    "Found {} occurrences of the text. The text must be unique. Please provide more context to make it unique, or use --multiple to replace all.",
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

/// Truncate a string for display purposes
fn truncate_for_display(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

impl std::error::Error for EditError {}

/// Detect line ending style from raw bytes using first-occurrence approach.
/// This is simpler and more predictable than counting all occurrences.
pub fn detect_line_endings(content: &[u8]) -> Option<LineEnding> {
    // Find positions of first \r\n and first \n
    let crlf_pos = content.windows(2).position(|w| w == b"\r\n");
    let lf_pos = content.iter().position(|&b| b == b'\n');

    match (crlf_pos, lf_pos) {
        (None, None) => None,
        (None, Some(_)) => Some(LineEnding::Lf),
        (Some(_), None) => Some(LineEnding::CrLf), // Shouldn't happen but handle it
        (Some(crlf), Some(lf)) => {
            // Check which comes first
            // Note: if CRLF is at position X, the \n of it is at X+1
            // If LF position equals CRLF position + 1, then the first line ending is CRLF
            if crlf + 1 == lf {
                Some(LineEnding::CrLf)
            } else if lf < crlf {
                Some(LineEnding::Lf)
            } else {
                Some(LineEnding::CrLf)
            }
        }
    }
}

/// Detect line ending style from string content
pub fn detect_line_endings_str(content: &str) -> Option<LineEnding> {
    let crlf_idx = content.find("\r\n");
    let lf_idx = content.find('\n');

    match (crlf_idx, lf_idx) {
        (None, None) => None,
        (None, Some(_)) => Some(LineEnding::Lf),
        (Some(_), None) => Some(LineEnding::CrLf),
        (Some(crlf), Some(lf)) => {
            // If LF is right after CR, it's part of CRLF
            if crlf + 1 == lf {
                Some(LineEnding::CrLf)
            } else if lf < crlf {
                Some(LineEnding::Lf)
            } else {
                Some(LineEnding::CrLf)
            }
        }
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

/// Result of diff generation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffResult {
    /// The unified diff string with line numbers
    pub diff: String,
    /// The first line number that changed (in the new file)
    pub first_changed_line: Option<usize>,
}

/// A single change in a diff
#[derive(Debug, Clone, PartialEq, Eq)]
enum DiffChange {
    /// Unchanged context line
    Context(String),
    /// Added line
    Added(String),
    /// Removed line
    Removed(String),
}

/// Generate a unified diff string with line numbers and context.
/// Returns both the diff string and the first changed line number (in the new file).
pub fn generate_diff(old_content: &str, new_content: &str, context_lines: usize) -> DiffResult {
    let old_lines: Vec<&str> = old_content.lines().collect();
    let new_lines: Vec<&str> = new_content.lines().collect();

    // Simple diff algorithm: find common prefix, suffix, and changed middle
    let changes = compute_line_diff(&old_lines, &new_lines);

    let max_line_num = old_lines.len().max(new_lines.len());
    let line_num_width = max_line_num.to_string().len().max(1);

    let mut output: Vec<String> = Vec::new();
    let mut old_line_num = 1usize;
    let mut new_line_num = 1usize;
    let mut first_changed_line: Option<usize> = None;

    let mut i = 0;
    while i < changes.len() {
        match &changes[i] {
            DiffChange::Context(_) => {
                // Check if there's a change coming up within context_lines
                let next_change_distance = changes[i..]
                    .iter()
                    .position(|c| !matches!(c, DiffChange::Context(_)));

                if let Some(dist) = next_change_distance {
                    if dist <= context_lines {
                        // Show this context line (leading context)
                        if let DiffChange::Context(line) = &changes[i] {
                            let line_num =
                                format!("{:>width$}", old_line_num, width = line_num_width);
                            output.push(format!(" {} {}", line_num, line));
                        }
                        old_line_num += 1;
                        new_line_num += 1;
                    } else {
                        // Skip with ellipsis if we have previous output
                        if !output.is_empty() {
                            let padding = " ".repeat(line_num_width);
                            output.push(format!(" {} ...", padding));
                        }
                        // Skip ahead, keeping track of line numbers
                        let skip_count = dist.saturating_sub(context_lines);
                        for _ in 0..skip_count {
                            old_line_num += 1;
                            new_line_num += 1;
                            i += 1;
                        }
                        continue;
                    }
                } else {
                    // No more changes, check if we need trailing context
                    let changes_before = i > 0
                        && changes[..i]
                            .iter()
                            .rev()
                            .take(context_lines + 1)
                            .any(|c| !matches!(c, DiffChange::Context(_)));

                    if changes_before {
                        // Count how many context lines since last change
                        let context_since_change = changes[..i]
                            .iter()
                            .rev()
                            .take_while(|c| matches!(c, DiffChange::Context(_)))
                            .count();

                        if context_since_change < context_lines {
                            if let DiffChange::Context(line) = &changes[i] {
                                let line_num =
                                    format!("{:>width$}", old_line_num, width = line_num_width);
                                output.push(format!(" {} {}", line_num, line));
                            }
                        } else if context_since_change == context_lines && i + 1 < changes.len() {
                            let padding = " ".repeat(line_num_width);
                            output.push(format!(" {} ...", padding));
                        }
                    }
                    old_line_num += 1;
                    new_line_num += 1;
                }
            }
            DiffChange::Removed(line) => {
                if first_changed_line.is_none() {
                    first_changed_line = Some(new_line_num);
                }
                let line_num = format!("{:>width$}", old_line_num, width = line_num_width);
                output.push(format!("-{} {}", line_num, line));
                old_line_num += 1;
            }
            DiffChange::Added(line) => {
                if first_changed_line.is_none() {
                    first_changed_line = Some(new_line_num);
                }
                let line_num = format!("{:>width$}", new_line_num, width = line_num_width);
                output.push(format!("+{} {}", line_num, line));
                new_line_num += 1;
            }
        }
        i += 1;
    }

    DiffResult {
        diff: output.join("\n"),
        first_changed_line,
    }
}

/// Compute line-by-line diff using longest common subsequence
fn compute_line_diff<'a>(old_lines: &[&'a str], new_lines: &[&'a str]) -> Vec<DiffChange> {
    // Use a simple LCS-based diff algorithm
    let old_len = old_lines.len();
    let new_len = new_lines.len();

    // Build LCS table
    let mut lcs = vec![vec![0usize; new_len + 1]; old_len + 1];
    for i in 1..=old_len {
        for j in 1..=new_len {
            if old_lines[i - 1] == new_lines[j - 1] {
                lcs[i][j] = lcs[i - 1][j - 1] + 1;
            } else {
                lcs[i][j] = lcs[i - 1][j].max(lcs[i][j - 1]);
            }
        }
    }

    // Backtrack to build diff
    let mut changes = Vec::new();
    let mut i = old_len;
    let mut j = new_len;

    while i > 0 || j > 0 {
        if i > 0 && j > 0 && old_lines[i - 1] == new_lines[j - 1] {
            changes.push(DiffChange::Context(old_lines[i - 1].to_string()));
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || lcs[i][j - 1] >= lcs[i - 1][j]) {
            changes.push(DiffChange::Added(new_lines[j - 1].to_string()));
            j -= 1;
        } else if i > 0 {
            changes.push(DiffChange::Removed(old_lines[i - 1].to_string()));
            i -= 1;
        }
    }

    changes.reverse();
    changes
}

/// Normalize line endings to LF
pub fn normalize_to_lf(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

/// Restore line endings from LF to specified style
pub fn restore_line_endings(text: &str, ending: LineEnding) -> String {
    match ending {
        LineEnding::Lf => text.to_string(),
        LineEnding::CrLf => text.replace('\n', "\r\n"),
    }
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

/// Extended edit result with diff information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditResultWithDiff {
    /// The modified content
    pub content: String,
    /// Number of replacements made
    pub replacements: usize,
    /// Detected line ending style
    pub line_ending: Option<LineEnding>,
    /// Unified diff of changes
    pub diff: String,
    /// First line that changed
    pub first_changed_line: Option<usize>,
    /// Whether fuzzy matching was used
    pub used_fuzzy_match: bool,
}

/// High-level function to perform a fuzzy file edit operation with diff output
/// This matches the TypeScript implementation more closely
pub fn edit_file_fuzzy(
    path: &Path,
    old_text: &str,
    new_text: &str,
    options: &ReplaceOptions,
) -> Result<EditResultWithDiff, EditError> {
    // Read file
    let (raw_content, line_ending) = read_file(path, options.encoding)?;

    // Strip BOM
    let stripped = strip_bom(&raw_content);
    let content = stripped.text;

    // Normalize line endings for matching
    let normalized_content = normalize_to_lf(&content);
    let normalized_old_text = normalize_to_lf(old_text);
    let normalized_new_text = normalize_to_lf(new_text);

    // Find the old text using fuzzy matching
    let match_result = fuzzy_find_text(&normalized_content, &normalized_old_text);

    if !match_result.found {
        return Err(EditError::NotFound(format!(
            "Could not find the exact text. The old text must match exactly including all whitespace and newlines."
        )));
    }

    // Count occurrences for uniqueness check
    let occurrences = count_fuzzy_occurrences(&normalized_content, &normalized_old_text);

    if occurrences > 1 && !options.multiple {
        return Err(EditError::MultipleFound(occurrences));
    }

    // Perform replacement(s)
    let base_content = &match_result.content_for_replacement;
    let (new_content, replacement_count) = if options.multiple && occurrences > 1 {
        // Replace all occurrences
        let mut result = base_content.clone();
        let mut count = 0;
        loop {
            let m = fuzzy_find_text(&result, &normalized_old_text);
            if !m.found {
                break;
            }
            result = format!(
                "{}{}{}",
                &m.content_for_replacement[..m.index],
                &normalized_new_text,
                &m.content_for_replacement[m.index + m.match_length..]
            );
            count += 1;
            // Safety: avoid infinite loop if replacement contains search text
            if count > 10000 {
                break;
            }
        }
        (result, count)
    } else {
        // Single replacement
        let result = format!(
            "{}{}{}",
            &base_content[..match_result.index],
            &normalized_new_text,
            &base_content[match_result.index + match_result.match_length..]
        );
        (result, 1)
    };

    // Check if replacement actually changed anything
    if base_content == &new_content {
        return Err(EditError::Other(
            "No changes made. The replacement produced identical content.".to_string(),
        ));
    }

    // Generate diff
    let diff_result = generate_diff(base_content, &new_content, 4);

    // Restore BOM and line endings for final output
    let final_content = if let Some(le) = line_ending {
        restore_line_endings(&new_content, le)
    } else {
        new_content.clone()
    };
    let final_with_bom = format!("{}{}", stripped.bom, final_content);

    // Write back (unless dry run)
    if !options.dry_run {
        write_file_atomic(path, &final_with_bom, options.encoding, None)?;
    }

    Ok(EditResultWithDiff {
        content: final_with_bom,
        replacements: replacement_count,
        line_ending,
        diff: diff_result.diff,
        first_changed_line: diff_result.first_changed_line,
        used_fuzzy_match: match_result.used_fuzzy_match,
    })
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

    // New tests for BOM handling
    #[test]
    fn test_strip_bom_present() {
        let content = "\u{FEFF}hello world";
        let result = strip_bom(content);
        assert_eq!(result.bom, "\u{FEFF}");
        assert_eq!(result.text, "hello world");
    }

    #[test]
    fn test_strip_bom_absent() {
        let content = "hello world";
        let result = strip_bom(content);
        assert_eq!(result.bom, "");
        assert_eq!(result.text, "hello world");
    }

    #[test]
    fn test_strip_bom_bytes_present() {
        let bytes = b"\xEF\xBB\xBFhello";
        let (has_bom, stripped) = strip_bom_bytes(bytes);
        assert!(has_bom);
        assert_eq!(stripped, b"hello");
    }

    #[test]
    fn test_strip_bom_bytes_absent() {
        let bytes = b"hello";
        let (has_bom, stripped) = strip_bom_bytes(bytes);
        assert!(!has_bom);
        assert_eq!(stripped, b"hello");
    }

    // New tests for fuzzy matching
    #[test]
    fn test_normalize_smart_quotes() {
        let text = "He said \u{201C}hello\u{201D} and \u{2018}goodbye\u{2019}";
        let normalized = normalize_for_fuzzy_match(text);
        assert_eq!(normalized, "He said \"hello\" and 'goodbye'");
    }

    #[test]
    fn test_normalize_dashes() {
        let text = "2020\u{2013}2024"; // en-dash
        let normalized = normalize_for_fuzzy_match(text);
        assert_eq!(normalized, "2020-2024");
    }

    #[test]
    fn test_normalize_special_spaces() {
        let text = "hello\u{00A0}world"; // NBSP
        let normalized = normalize_for_fuzzy_match(text);
        assert_eq!(normalized, "hello world");
    }

    #[test]
    fn test_normalize_trailing_whitespace() {
        let text = "hello   \nworld  ";
        let normalized = normalize_for_fuzzy_match(text);
        assert_eq!(normalized, "hello\nworld");
    }

    #[test]
    fn test_fuzzy_find_exact_match() {
        let result = fuzzy_find_text("hello world", "world");
        assert!(result.found);
        assert_eq!(result.index, 6);
        assert!(!result.used_fuzzy_match);
    }

    #[test]
    fn test_fuzzy_find_smart_quotes() {
        let content = "He said \u{201C}hello\u{201D}";
        let search = "He said \"hello\"";
        let result = fuzzy_find_text(content, search);
        assert!(result.found);
        assert!(result.used_fuzzy_match);
    }

    #[test]
    fn test_fuzzy_find_not_found() {
        let result = fuzzy_find_text("hello world", "goodbye");
        assert!(!result.found);
    }

    #[test]
    fn test_count_fuzzy_occurrences() {
        let content = "foo bar foo baz foo";
        assert_eq!(count_fuzzy_occurrences(content, "foo"), 3);
        assert_eq!(count_fuzzy_occurrences(content, "bar"), 1);
        assert_eq!(count_fuzzy_occurrences(content, "qux"), 0);
    }

    // New tests for line ending detection (first-occurrence)
    #[test]
    fn test_detect_line_endings_mixed_crlf_first() {
        // CRLF comes before LF
        let content = b"line1\r\nline2\nline3";
        assert_eq!(detect_line_endings(content), Some(LineEnding::CrLf));
    }

    #[test]
    fn test_detect_line_endings_mixed_lf_first() {
        // LF comes before CRLF
        let content = b"line1\nline2\r\nline3";
        assert_eq!(detect_line_endings(content), Some(LineEnding::Lf));
    }

    #[test]
    fn test_detect_line_endings_str() {
        assert_eq!(
            detect_line_endings_str("hello\nworld"),
            Some(LineEnding::Lf)
        );
        assert_eq!(
            detect_line_endings_str("hello\r\nworld"),
            Some(LineEnding::CrLf)
        );
        assert_eq!(detect_line_endings_str("hello world"), None);
    }

    // New tests for diff generation
    #[test]
    fn test_generate_diff_simple() {
        let old = "line1\nline2\nline3";
        let new = "line1\nmodified\nline3";
        let result = generate_diff(old, new, 1);
        assert!(result.diff.contains("-"));
        assert!(result.diff.contains("+"));
        assert!(result.diff.contains("line2"));
        assert!(result.diff.contains("modified"));
        assert!(result.first_changed_line.is_some());
    }

    #[test]
    fn test_generate_diff_addition() {
        let old = "line1\nline2";
        let new = "line1\nnew line\nline2";
        let result = generate_diff(old, new, 1);
        assert!(result.diff.contains("+"));
        assert!(result.diff.contains("new line"));
    }

    #[test]
    fn test_generate_diff_deletion() {
        let old = "line1\nto delete\nline2";
        let new = "line1\nline2";
        let result = generate_diff(old, new, 1);
        assert!(result.diff.contains("-"));
        assert!(result.diff.contains("to delete"));
    }

    #[test]
    fn test_generate_diff_no_changes() {
        let content = "line1\nline2\nline3";
        let result = generate_diff(content, content, 1);
        assert!(result.first_changed_line.is_none());
    }

    // Tests for line ending normalization
    #[test]
    fn test_normalize_to_lf() {
        assert_eq!(normalize_to_lf("hello\r\nworld"), "hello\nworld");
        assert_eq!(normalize_to_lf("hello\rworld"), "hello\nworld");
        assert_eq!(normalize_to_lf("hello\nworld"), "hello\nworld");
    }

    #[test]
    fn test_restore_line_endings() {
        assert_eq!(
            restore_line_endings("hello\nworld", LineEnding::Lf),
            "hello\nworld"
        );
        assert_eq!(
            restore_line_endings("hello\nworld", LineEnding::CrLf),
            "hello\r\nworld"
        );
    }
}
