//! Structured editing support for JSON, JSONC, JSON5, TOML, and YAML files.
//!
//! This module provides key-path based editing that preserves formatting
//! and comments where possible.

use crate::api::{
    key_not_found_msg, normalize_to_lf, read_file, restore_line_endings, strip_bom,
    write_file_atomic, EditError, Encoding, LineEnding,
};
use std::path::Path;

/// Supported structured file formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredFormat {
    /// Standard JSON
    Json,
    /// JSON with Comments (VS Code style)
    Jsonc,
    /// JSON5 (relaxed JSON with comments, trailing commas, etc.)
    Json5,
    /// TOML
    Toml,
    /// YAML
    Yaml,
}

impl StructuredFormat {
    /// Detect format from file extension
    pub fn from_extension(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?.to_lowercase();
        match ext.as_str() {
            "json" => Some(StructuredFormat::Json),
            "jsonc" => Some(StructuredFormat::Jsonc),
            "json5" => Some(StructuredFormat::Json5),
            "toml" => Some(StructuredFormat::Toml),
            "yaml" | "yml" => Some(StructuredFormat::Yaml),
            _ => None,
        }
    }

    /// Parse format from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "json" => Some(StructuredFormat::Json),
            "jsonc" => Some(StructuredFormat::Jsonc),
            "json5" => Some(StructuredFormat::Json5),
            "toml" => Some(StructuredFormat::Toml),
            "yaml" | "yml" => Some(StructuredFormat::Yaml),
            _ => None,
        }
    }

    /// Get the format name
    pub fn as_str(&self) -> &'static str {
        match self {
            StructuredFormat::Json => "json",
            StructuredFormat::Jsonc => "jsonc",
            StructuredFormat::Json5 => "json5",
            StructuredFormat::Toml => "toml",
            StructuredFormat::Yaml => "yaml",
        }
    }
}

/// Result of a structured edit operation
#[derive(Debug, Clone)]
pub struct StructuredEditResult {
    /// The modified content
    pub content: String,
    /// The format that was used
    pub format: StructuredFormat,
    /// The key path that was modified
    pub key_path: String,
    /// The old value (as string)
    pub old_value: Option<String>,
    /// The new value (as string)
    pub new_value: String,
    /// Detected line ending
    pub line_ending: Option<LineEnding>,
}

/// Parse a key path into segments
/// Supports: "foo.bar", "foo[0]", "foo[0].bar", "foo.bar[1].baz"
fn parse_key_path(path: &str) -> Result<Vec<KeySegment>, EditError> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut chars = path.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '.' => {
                if !current.is_empty() {
                    segments.push(KeySegment::Key(current.clone()));
                    current.clear();
                }
            }
            '[' => {
                if !current.is_empty() {
                    segments.push(KeySegment::Key(current.clone()));
                    current.clear();
                }
                // Parse index
                let mut index_str = String::new();
                while let Some(&c) = chars.peek() {
                    if c == ']' {
                        chars.next();
                        break;
                    }
                    index_str.push(chars.next().unwrap());
                }
                let index: usize = index_str.parse().map_err(|_| {
                    EditError::InvalidKeyPath(format!("Invalid array index: {}", index_str))
                })?;
                segments.push(KeySegment::Index(index));
            }
            _ => {
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        segments.push(KeySegment::Key(current));
    }

    if segments.is_empty() {
        return Err(EditError::InvalidKeyPath("Empty key path".to_string()));
    }

    Ok(segments)
}

#[derive(Debug, Clone)]
enum KeySegment {
    Key(String),
    Index(usize),
}

// ============================================================================
// JSON / JSONC / JSON5 Support
// ============================================================================

/// Edit a JSON value at the given key path
pub fn edit_json(
    content: &str,
    key_path: &str,
    new_value: &str,
) -> Result<(String, Option<String>), EditError> {
    let segments = parse_key_path(key_path)?;

    let mut doc: serde_json::Value = serde_json::from_str(content)
        .map_err(|e| EditError::Other(format!("JSON parse error: {}", e)))?;

    let old_value = get_json_value(&doc, &segments)?;
    let old_str = serde_json::to_string(&old_value).ok();

    // Parse the new value
    let parsed_value: serde_json::Value = serde_json::from_str(new_value)
        .map_err(|_| {
            // If it's not valid JSON, try as a raw string
            serde_json::Value::String(new_value.to_string())
        })
        .unwrap_or_else(|_| serde_json::Value::String(new_value.to_string()));

    set_json_value(&mut doc, &segments, parsed_value)?;

    let output = serde_json::to_string_pretty(&doc)
        .map_err(|e| EditError::Other(format!("JSON serialize error: {}", e)))?;

    Ok((output, old_str))
}

/// Edit a JSONC file (JSON with comments)
/// We strip comments, edit, then try to preserve structure
pub fn edit_jsonc(
    content: &str,
    key_path: &str,
    new_value: &str,
) -> Result<(String, Option<String>), EditError> {
    // Strip comments for parsing
    let stripped = strip_json_comments(content);
    edit_json(&stripped, key_path, new_value)
}

/// Edit a JSON5 file
pub fn edit_json5(
    content: &str,
    key_path: &str,
    new_value: &str,
) -> Result<(String, Option<String>), EditError> {
    let segments = parse_key_path(key_path)?;

    let mut doc: serde_json::Value = json5::from_str(content)
        .map_err(|e| EditError::Other(format!("JSON5 parse error: {}", e)))?;

    let old_value = get_json_value(&doc, &segments)?;
    let old_str = serde_json::to_string(&old_value).ok();

    // Parse the new value (try JSON5 first, then JSON, then string)
    let parsed_value: serde_json::Value = json5::from_str(new_value)
        .or_else(|_| serde_json::from_str(new_value))
        .unwrap_or_else(|_| serde_json::Value::String(new_value.to_string()));

    set_json_value(&mut doc, &segments, parsed_value)?;

    // Output as standard JSON (JSON5 doesn't have a writer)
    let output = serde_json::to_string_pretty(&doc)
        .map_err(|e| EditError::Other(format!("JSON serialize error: {}", e)))?;

    Ok((output, old_str))
}

fn get_json_value(
    doc: &serde_json::Value,
    segments: &[KeySegment],
) -> Result<serde_json::Value, EditError> {
    let mut current = doc;

    for segment in segments {
        current = match segment {
            KeySegment::Key(key) => {
                let available: Vec<&str> = current
                    .as_object()
                    .map(|obj| obj.keys().map(|k| k.as_str()).collect())
                    .unwrap_or_default();
                current
                    .get(key)
                    .ok_or_else(|| EditError::KeyNotFound(key_not_found_msg(key, &available)))?
            }
            KeySegment::Index(idx) => current
                .get(*idx)
                .ok_or_else(|| EditError::KeyNotFound(format!("Index {} out of bounds", idx)))?,
        };
    }

    Ok(current.clone())
}

fn set_json_value(
    doc: &mut serde_json::Value,
    segments: &[KeySegment],
    value: serde_json::Value,
) -> Result<(), EditError> {
    if segments.is_empty() {
        return Err(EditError::InvalidKeyPath("Empty key path".to_string()));
    }

    let mut current = doc;

    // Navigate to parent
    for segment in &segments[..segments.len() - 1] {
        current = match segment {
            KeySegment::Key(key) => {
                let available: Vec<String> = current
                    .as_object()
                    .map(|obj| obj.keys().cloned().collect())
                    .unwrap_or_default();
                let available_refs: Vec<&str> =
                    available.iter().map(|s| s.as_str()).collect();
                current
                    .get_mut(key)
                    .ok_or_else(|| EditError::KeyNotFound(key_not_found_msg(key, &available_refs)))?
            }
            KeySegment::Index(idx) => current
                .get_mut(*idx)
                .ok_or_else(|| EditError::KeyNotFound(format!("Index {} out of bounds", idx)))?,
        };
    }

    // Set the value
    match &segments[segments.len() - 1] {
        KeySegment::Key(key) => {
            if let Some(obj) = current.as_object_mut() {
                obj.insert(key.clone(), value);
            } else {
                return Err(EditError::InvalidKeyPath(format!(
                    "Cannot set key '{}' on non-object",
                    key
                )));
            }
        }
        KeySegment::Index(idx) => {
            if let Some(arr) = current.as_array_mut() {
                if *idx < arr.len() {
                    arr[*idx] = value;
                } else {
                    return Err(EditError::KeyNotFound(format!(
                        "Index {} out of bounds (array length: {})",
                        idx,
                        arr.len()
                    )));
                }
            } else {
                return Err(EditError::InvalidKeyPath(format!(
                    "Cannot set index {} on non-array",
                    idx
                )));
            }
        }
    }

    Ok(())
}

/// Strip C-style comments from JSON (for JSONC support)
fn strip_json_comments(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();
    let mut in_string = false;
    let mut escape_next = false;

    while let Some(c) = chars.next() {
        if escape_next {
            result.push(c);
            escape_next = false;
            continue;
        }

        if c == '\\' && in_string {
            result.push(c);
            escape_next = true;
            continue;
        }

        if c == '"' && !escape_next {
            in_string = !in_string;
            result.push(c);
            continue;
        }

        if !in_string && c == '/' {
            if let Some(&next) = chars.peek() {
                if next == '/' {
                    // Line comment - skip to end of line
                    chars.next();
                    while let Some(&c) = chars.peek() {
                        if c == '\n' {
                            break;
                        }
                        chars.next();
                    }
                    continue;
                } else if next == '*' {
                    // Block comment - skip to */
                    chars.next();
                    while let Some(c) = chars.next() {
                        if c == '*' {
                            if let Some(&'/') = chars.peek() {
                                chars.next();
                                break;
                            }
                        }
                    }
                    continue;
                }
            }
        }

        result.push(c);
    }

    result
}

// ============================================================================
// TOML Support
// ============================================================================

/// Edit a TOML file, preserving formatting and comments
pub fn edit_toml(
    content: &str,
    key_path: &str,
    new_value: &str,
) -> Result<(String, Option<String>), EditError> {
    use toml_edit::DocumentMut;

    let segments = parse_key_path(key_path)?;

    let mut doc: DocumentMut = content
        .parse()
        .map_err(|e| EditError::Other(format!("TOML parse error: {}", e)))?;

    // Get old value
    let old_value = get_toml_value(&doc, &segments)?;

    // Parse new value
    let parsed_value = parse_toml_value(new_value)?;

    // Set new value
    set_toml_value(&mut doc, &segments, parsed_value)?;

    Ok((doc.to_string(), old_value))
}

fn get_toml_value(
    doc: &toml_edit::DocumentMut,
    segments: &[KeySegment],
) -> Result<Option<String>, EditError> {
    use toml_edit::Item;

    // Helper enum to track what we're pointing at
    enum TomlRef<'a> {
        Item(&'a Item),
        Value(&'a toml_edit::Value),
    }

    let mut current = TomlRef::Item(doc.as_item());

    for segment in segments {
        current = match (&current, segment) {
            (TomlRef::Item(item), KeySegment::Key(key)) => {
                if let Some(table) = item.as_table_like() {
                    let available: Vec<&str> = table.iter().map(|(k, _)| k).collect();
                    TomlRef::Item(table.get(key).ok_or_else(|| {
                        EditError::KeyNotFound(key_not_found_msg(key, &available))
                    })?)
                } else {
                    return Err(EditError::InvalidKeyPath(format!(
                        "Cannot access key '{}' on non-table",
                        key
                    )));
                }
            }
            (TomlRef::Item(item), KeySegment::Index(idx)) => {
                if let Some(arr) = item.as_array() {
                    TomlRef::Value(arr.get(*idx).ok_or_else(|| {
                        EditError::KeyNotFound(format!("Index {} out of bounds", idx))
                    })?)
                } else {
                    return Err(EditError::InvalidKeyPath(format!(
                        "Cannot access index {} on non-array",
                        idx
                    )));
                }
            }
            (TomlRef::Value(val), KeySegment::Key(key)) => {
                if let Some(tbl) = val.as_inline_table() {
                    let available: Vec<&str> = tbl.iter().map(|(k, _)| k).collect();
                    TomlRef::Value(tbl.get(key).ok_or_else(|| {
                        EditError::KeyNotFound(key_not_found_msg(key, &available))
                    })?)
                } else {
                    return Err(EditError::InvalidKeyPath(format!(
                        "Cannot access key '{}' on non-table value",
                        key
                    )));
                }
            }
            (TomlRef::Value(val), KeySegment::Index(idx)) => {
                if let Some(arr) = val.as_array() {
                    TomlRef::Value(arr.get(*idx).ok_or_else(|| {
                        EditError::KeyNotFound(format!("Index {} out of bounds", idx))
                    })?)
                } else {
                    return Err(EditError::InvalidKeyPath(format!(
                        "Cannot access index {} on non-array value",
                        idx
                    )));
                }
            }
        };
    }

    // Convert to string representation
    match current {
        TomlRef::Item(item) => {
            if let Some(value) = item.as_value() {
                Ok(Some(value.to_string()))
            } else {
                Ok(Some(item.to_string()))
            }
        }
        TomlRef::Value(val) => Ok(Some(val.to_string())),
    }
}

fn set_toml_value(
    doc: &mut toml_edit::DocumentMut,
    segments: &[KeySegment],
    value: toml_edit::Value,
) -> Result<(), EditError> {
    use toml_edit::Item;

    if segments.is_empty() {
        return Err(EditError::InvalidKeyPath("Empty key path".to_string()));
    }

    // Helper enum to track what we're pointing at
    enum TomlRefMut<'a> {
        Item(&'a mut Item),
        Value(&'a mut toml_edit::Value),
    }

    let mut current = TomlRefMut::Item(doc.as_item_mut());

    // Navigate to parent
    for segment in &segments[..segments.len() - 1] {
        current = match (current, segment) {
            (TomlRefMut::Item(item), KeySegment::Key(key)) => {
                // Collect keys before taking mutable ref
                let available: Vec<String> = item
                    .as_table_like()
                    .map(|t| t.iter().map(|(k, _)| k.to_string()).collect())
                    .unwrap_or_default();
                if let Some(table) = item.as_table_like_mut() {
                    let available_refs: Vec<&str> =
                        available.iter().map(|s| s.as_str()).collect();
                    TomlRefMut::Item(table.get_mut(key).ok_or_else(|| {
                        EditError::KeyNotFound(key_not_found_msg(key, &available_refs))
                    })?)
                } else {
                    return Err(EditError::InvalidKeyPath(format!(
                        "Cannot access key '{}' on non-table",
                        key
                    )));
                }
            }
            (TomlRefMut::Item(item), KeySegment::Index(idx)) => {
                if let Some(arr) = item.as_array_mut() {
                    TomlRefMut::Value(arr.get_mut(*idx).ok_or_else(|| {
                        EditError::KeyNotFound(format!("Index {} out of bounds", idx))
                    })?)
                } else {
                    return Err(EditError::InvalidKeyPath(format!(
                        "Cannot access index {} on non-array",
                        idx
                    )));
                }
            }
            (TomlRefMut::Value(val), KeySegment::Key(key)) => {
                // Collect keys before taking mutable ref
                let available: Vec<String> = val
                    .as_inline_table()
                    .map(|t| t.iter().map(|(k, _)| k.to_string()).collect())
                    .unwrap_or_default();
                if let Some(tbl) = val.as_inline_table_mut() {
                    let available_refs: Vec<&str> =
                        available.iter().map(|s| s.as_str()).collect();
                    TomlRefMut::Value(tbl.get_mut(key).ok_or_else(|| {
                        EditError::KeyNotFound(key_not_found_msg(key, &available_refs))
                    })?)
                } else {
                    return Err(EditError::InvalidKeyPath(format!(
                        "Cannot access key '{}' on non-table value",
                        key
                    )));
                }
            }
            (TomlRefMut::Value(val), KeySegment::Index(idx)) => {
                if let Some(arr) = val.as_array_mut() {
                    TomlRefMut::Value(arr.get_mut(*idx).ok_or_else(|| {
                        EditError::KeyNotFound(format!("Index {} out of bounds", idx))
                    })?)
                } else {
                    return Err(EditError::InvalidKeyPath(format!(
                        "Cannot access index {} on non-array value",
                        idx
                    )));
                }
            }
        };
    }

    // Set the value
    match (&segments[segments.len() - 1], current) {
        (KeySegment::Key(key), TomlRefMut::Item(item)) => {
            if let Some(table) = item.as_table_like_mut() {
                table.insert(key, Item::Value(value));
            } else {
                return Err(EditError::InvalidKeyPath(format!(
                    "Cannot set key '{}' on non-table",
                    key
                )));
            }
        }
        (KeySegment::Key(key), TomlRefMut::Value(val)) => {
            if let Some(tbl) = val.as_inline_table_mut() {
                tbl.insert(key, value);
            } else {
                return Err(EditError::InvalidKeyPath(format!(
                    "Cannot set key '{}' on non-table value",
                    key
                )));
            }
        }
        (KeySegment::Index(idx), TomlRefMut::Item(item)) => {
            if let Some(arr) = item.as_array_mut() {
                if *idx < arr.len() {
                    arr.replace(*idx, value);
                } else {
                    return Err(EditError::KeyNotFound(format!(
                        "Index {} out of bounds",
                        idx
                    )));
                }
            } else {
                return Err(EditError::InvalidKeyPath(format!(
                    "Cannot set index {} on non-array",
                    idx
                )));
            }
        }
        (KeySegment::Index(idx), TomlRefMut::Value(val)) => {
            if let Some(arr) = val.as_array_mut() {
                if *idx < arr.len() {
                    arr.replace(*idx, value);
                } else {
                    return Err(EditError::KeyNotFound(format!(
                        "Index {} out of bounds",
                        idx
                    )));
                }
            } else {
                return Err(EditError::InvalidKeyPath(format!(
                    "Cannot set index {} on non-array value",
                    idx
                )));
            }
        }
    }

    Ok(())
}

fn parse_toml_value(s: &str) -> Result<toml_edit::Value, EditError> {
    // Try to parse as various TOML types
    // First try as a TOML value directly
    if let Ok(v) = s.parse::<toml_edit::Value>() {
        return Ok(v);
    }

    // Try to infer the type
    if s == "true" {
        return Ok(toml_edit::Value::from(true));
    }
    if s == "false" {
        return Ok(toml_edit::Value::from(false));
    }
    if let Ok(i) = s.parse::<i64>() {
        return Ok(toml_edit::Value::from(i));
    }
    if let Ok(f) = s.parse::<f64>() {
        return Ok(toml_edit::Value::from(f));
    }

    // Default to string (unquoted input becomes quoted string)
    Ok(toml_edit::Value::from(s))
}

// ============================================================================
// YAML Support
// ============================================================================

/// Edit a YAML file
pub fn edit_yaml(
    content: &str,
    key_path: &str,
    new_value: &str,
) -> Result<(String, Option<String>), EditError> {
    let segments = parse_key_path(key_path)?;

    let mut doc: serde_yaml::Value = serde_yaml::from_str(content)
        .map_err(|e| EditError::Other(format!("YAML parse error: {}", e)))?;

    let old_value = get_yaml_value(&doc, &segments)?;
    let old_str = serde_yaml::to_string(&old_value).ok();

    // Parse the new value
    let parsed_value: serde_yaml::Value = serde_yaml::from_str(new_value)
        .unwrap_or_else(|_| serde_yaml::Value::String(new_value.to_string()));

    set_yaml_value(&mut doc, &segments, parsed_value)?;

    let output = serde_yaml::to_string(&doc)
        .map_err(|e| EditError::Other(format!("YAML serialize error: {}", e)))?;

    Ok((output, old_str))
}

fn get_yaml_value(
    doc: &serde_yaml::Value,
    segments: &[KeySegment],
) -> Result<serde_yaml::Value, EditError> {
    let mut current = doc;

    for segment in segments {
        current = match segment {
            KeySegment::Key(key) => {
                let available: Vec<&str> = current
                    .as_mapping()
                    .map(|m| {
                        m.keys()
                            .filter_map(|k| k.as_str())
                            .collect()
                    })
                    .unwrap_or_default();
                current
                    .get(key)
                    .ok_or_else(|| EditError::KeyNotFound(key_not_found_msg(key, &available)))?
            }
            KeySegment::Index(idx) => current
                .get(*idx)
                .ok_or_else(|| EditError::KeyNotFound(format!("Index {} out of bounds", idx)))?,
        };
    }

    Ok(current.clone())
}

fn set_yaml_value(
    doc: &mut serde_yaml::Value,
    segments: &[KeySegment],
    value: serde_yaml::Value,
) -> Result<(), EditError> {
    if segments.is_empty() {
        return Err(EditError::InvalidKeyPath("Empty key path".to_string()));
    }

    let mut current = doc;

    // Navigate to parent
    for segment in &segments[..segments.len() - 1] {
        current = match segment {
            KeySegment::Key(key) => {
                let available: Vec<String> = current
                    .as_mapping()
                    .map(|m| {
                        m.keys()
                            .filter_map(|k| k.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();
                let available_refs: Vec<&str> =
                    available.iter().map(|s| s.as_str()).collect();
                current
                    .get_mut(key)
                    .ok_or_else(|| EditError::KeyNotFound(key_not_found_msg(key, &available_refs)))?
            }
            KeySegment::Index(idx) => current
                .get_mut(*idx)
                .ok_or_else(|| EditError::KeyNotFound(format!("Index {} out of bounds", idx)))?,
        };
    }

    // Set the value
    match &segments[segments.len() - 1] {
        KeySegment::Key(key) => {
            if let Some(mapping) = current.as_mapping_mut() {
                mapping.insert(serde_yaml::Value::String(key.clone()), value);
            } else {
                return Err(EditError::InvalidKeyPath(format!(
                    "Cannot set key '{}' on non-mapping",
                    key
                )));
            }
        }
        KeySegment::Index(idx) => {
            if let Some(seq) = current.as_sequence_mut() {
                if *idx < seq.len() {
                    seq[*idx] = value;
                } else {
                    return Err(EditError::KeyNotFound(format!(
                        "Index {} out of bounds (sequence length: {})",
                        idx,
                        seq.len()
                    )));
                }
            } else {
                return Err(EditError::InvalidKeyPath(format!(
                    "Cannot set index {} on non-sequence",
                    idx
                )));
            }
        }
    }

    Ok(())
}

// ============================================================================
// High-level API
// ============================================================================

/// Edit a structured file at the given key path
pub fn edit_structured(
    path: &Path,
    key_path: &str,
    new_value: &str,
    format: Option<StructuredFormat>,
    encoding: Encoding,
    dry_run: bool,
) -> Result<StructuredEditResult, EditError> {
    // Determine format
    let format = format
        .or_else(|| StructuredFormat::from_extension(path))
        .ok_or_else(|| {
            EditError::Other(format!(
                "Cannot determine format for '{}'. Use --format to specify.",
                path.display()
            ))
        })?;

    // Read file
    let (raw_content, line_ending) = read_file(path, encoding)?;

    // Strip BOM
    let stripped = strip_bom(&raw_content);
    let content = normalize_to_lf(&stripped.text);

    // Edit based on format
    let (new_content, old_value) = match format {
        StructuredFormat::Json => edit_json(&content, key_path, new_value)?,
        StructuredFormat::Jsonc => edit_jsonc(&content, key_path, new_value)?,
        StructuredFormat::Json5 => edit_json5(&content, key_path, new_value)?,
        StructuredFormat::Toml => edit_toml(&content, key_path, new_value)?,
        StructuredFormat::Yaml => edit_yaml(&content, key_path, new_value)?,
    };

    // Restore line endings and BOM
    let final_content = if let Some(le) = line_ending {
        restore_line_endings(&new_content, le)
    } else {
        new_content.clone()
    };
    let final_with_bom = format!("{}{}", stripped.bom, final_content);

    // Write back (unless dry run)
    if !dry_run {
        write_file_atomic(path, &final_with_bom, encoding, None)?;
    }

    Ok(StructuredEditResult {
        content: final_with_bom,
        format,
        key_path: key_path.to_string(),
        old_value,
        new_value: new_value.to_string(),
        line_ending,
    })
}

/// Edit structured content in-memory (without file I/O)
pub fn edit_structured_content(
    content: &str,
    key_path: &str,
    new_value: &str,
    format: StructuredFormat,
) -> Result<(String, Option<String>), EditError> {
    match format {
        StructuredFormat::Json => edit_json(content, key_path, new_value),
        StructuredFormat::Jsonc => edit_jsonc(content, key_path, new_value),
        StructuredFormat::Json5 => edit_json5(content, key_path, new_value),
        StructuredFormat::Toml => edit_toml(content, key_path, new_value),
        StructuredFormat::Yaml => edit_yaml(content, key_path, new_value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_key_path_simple() {
        let segments = parse_key_path("foo.bar.baz").unwrap();
        assert_eq!(segments.len(), 3);
    }

    #[test]
    fn test_parse_key_path_with_array() {
        let segments = parse_key_path("foo[0].bar").unwrap();
        assert_eq!(segments.len(), 3);
    }

    #[test]
    fn test_edit_json_simple() {
        let json = r#"{"name": "old", "value": 42}"#;
        let (result, old) = edit_json(json, "name", "\"new\"").unwrap();
        assert!(result.contains("\"new\""));
        assert_eq!(old, Some("\"old\"".to_string()));
    }

    #[test]
    fn test_edit_json_nested() {
        let json = r#"{"outer": {"inner": "old"}}"#;
        let (result, _) = edit_json(json, "outer.inner", "\"new\"").unwrap();
        assert!(result.contains("\"new\""));
    }

    #[test]
    fn test_edit_json_array() {
        let json = r#"{"items": ["a", "b", "c"]}"#;
        let (result, _) = edit_json(json, "items[1]", "\"x\"").unwrap();
        assert!(result.contains("\"x\""));
    }

    #[test]
    fn test_strip_json_comments() {
        let jsonc = r#"{
            // This is a comment
            "name": "value", /* inline comment */
            "other": 123
        }"#;
        let stripped = strip_json_comments(jsonc);
        assert!(!stripped.contains("//"));
        assert!(!stripped.contains("/*"));
        assert!(stripped.contains("\"name\""));
    }

    #[test]
    fn test_edit_toml_simple() {
        let toml = r#"name = "old"
value = 42"#;
        let (result, old) = edit_toml(toml, "name", "\"new\"").unwrap();
        assert!(result.contains("\"new\""));
        assert!(old.is_some());
    }

    #[test]
    fn test_edit_toml_nested() {
        let toml = r#"[section]
name = "old""#;
        let (result, _) = edit_toml(toml, "section.name", "\"new\"").unwrap();
        assert!(result.contains("\"new\""));
    }

    #[test]
    fn test_edit_yaml_simple() {
        let yaml = "name: old\nvalue: 42";
        let (result, old) = edit_yaml(yaml, "name", "new").unwrap();
        assert!(result.contains("new"));
        assert!(old.is_some());
    }

    #[test]
    fn test_edit_yaml_nested() {
        let yaml = "outer:\n  inner: old";
        let (result, _) = edit_yaml(yaml, "outer.inner", "new").unwrap();
        assert!(result.contains("new"));
    }

    #[test]
    fn test_json_key_not_found_suggestion() {
        let json = r#"{"name": "old", "value": 42, "description": "test"}"#;
        let result = edit_json(json, "nme", "\"new\"");
        match result {
            Err(EditError::KeyNotFound(msg)) => {
                assert!(
                    msg.contains("Did you mean 'name'?"),
                    "Expected suggestion, got: {}",
                    msg
                );
            }
            other => panic!("Expected KeyNotFound error, got: {:?}", other),
        }
    }

    #[test]
    fn test_json_key_not_found_no_suggestion() {
        let json = r#"{"name": "old", "value": 42}"#;
        let result = edit_json(json, "zzzzzzzzz", "\"new\"");
        match result {
            Err(EditError::KeyNotFound(msg)) => {
                assert!(
                    !msg.contains("Did you mean"),
                    "Expected no suggestion, got: {}",
                    msg
                );
            }
            other => panic!("Expected KeyNotFound error, got: {:?}", other),
        }
    }

    #[test]
    fn test_json_nested_key_not_found_suggestion() {
        let json = r#"{"outer": {"inner": "old", "value": 42}}"#;
        let result = edit_json(json, "outer.innr", "\"new\"");
        match result {
            Err(EditError::KeyNotFound(msg)) => {
                assert!(
                    msg.contains("Did you mean 'inner'?"),
                    "Expected suggestion for nested key, got: {}",
                    msg
                );
            }
            other => panic!("Expected KeyNotFound error, got: {:?}", other),
        }
    }

    #[test]
    fn test_toml_key_not_found_suggestion() {
        let toml = r#"[section]
name = "old"
value = 42"#;
        let result = edit_toml(toml, "section.nme", "\"new\"");
        match result {
            Err(EditError::KeyNotFound(msg)) => {
                assert!(
                    msg.contains("Did you mean 'name'?"),
                    "Expected suggestion, got: {}",
                    msg
                );
            }
            other => panic!("Expected KeyNotFound error, got: {:?}", other),
        }
    }

    #[test]
    fn test_yaml_key_not_found_suggestion() {
        let yaml = "name: old\nvalue: 42\ndescription: test";
        let result = edit_yaml(yaml, "nme", "new");
        match result {
            Err(EditError::KeyNotFound(msg)) => {
                assert!(
                    msg.contains("Did you mean 'name'?"),
                    "Expected suggestion, got: {}",
                    msg
                );
            }
            other => panic!("Expected KeyNotFound error, got: {:?}", other),
        }
    }

    #[test]
    fn test_yaml_nested_key_not_found_suggestion() {
        let yaml = "outer:\n  inner: old\n  value: 42";
        let result = edit_yaml(yaml, "outer.innr", "new");
        match result {
            Err(EditError::KeyNotFound(msg)) => {
                assert!(
                    msg.contains("Did you mean 'inner'?"),
                    "Expected suggestion for nested key, got: {}",
                    msg
                );
            }
            other => panic!("Expected KeyNotFound error, got: {:?}", other),
        }
    }

    #[test]
    fn test_format_detection() {
        assert_eq!(
            StructuredFormat::from_extension(Path::new("config.json")),
            Some(StructuredFormat::Json)
        );
        assert_eq!(
            StructuredFormat::from_extension(Path::new("config.toml")),
            Some(StructuredFormat::Toml)
        );
        assert_eq!(
            StructuredFormat::from_extension(Path::new("config.yaml")),
            Some(StructuredFormat::Yaml)
        );
        assert_eq!(
            StructuredFormat::from_extension(Path::new("config.yml")),
            Some(StructuredFormat::Yaml)
        );
    }
}
