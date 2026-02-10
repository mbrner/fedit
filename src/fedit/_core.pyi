"""Type stubs for the fedit._core Rust extension module."""

from typing import Optional, Tuple

class Encoding:
    """File encoding enum."""

    def __init__(self, name: str = "utf-8") -> None:
        """Create an Encoding from a string name."""
        ...

    @staticmethod
    def utf8() -> "Encoding":
        """UTF-8 encoding."""
        ...

    @staticmethod
    def utf16() -> "Encoding":
        """UTF-16 Little Endian encoding."""
        ...

    @staticmethod
    def iso8859_1() -> "Encoding":
        """ISO-8859-1 (Latin-1) encoding."""
        ...

    @staticmethod
    def windows1252() -> "Encoding":
        """Windows-1252 encoding."""
        ...

class EditResult:
    """Result of an edit operation."""

    @property
    def content(self) -> str:
        """The modified content."""
        ...

    @property
    def replacements(self) -> int:
        """Number of replacements made."""
        ...

    @property
    def line_ending(self) -> Optional[str]:
        """Detected line ending style ('lf', 'crlf', or None)."""
        ...

class EditResultWithDiff:
    """Result of an edit operation with diff information."""

    @property
    def content(self) -> str:
        """The modified content."""
        ...

    @property
    def replacements(self) -> int:
        """Number of replacements made."""
        ...

    @property
    def line_ending(self) -> Optional[str]:
        """Detected line ending style ('lf', 'crlf', or None)."""
        ...

    @property
    def diff(self) -> str:
        """Unified diff of changes with line numbers."""
        ...

    @property
    def first_changed_line(self) -> Optional[int]:
        """The first line number that changed (in the new file)."""
        ...

    @property
    def used_fuzzy_match(self) -> bool:
        """Whether fuzzy matching was used (False = exact match)."""
        ...

class FuzzyMatchResult:
    """Result of a fuzzy find operation."""

    @property
    def found(self) -> bool:
        """Whether a match was found."""
        ...

    @property
    def index(self) -> int:
        """The index where the match starts."""
        ...

    @property
    def match_length(self) -> int:
        """Length of the matched text."""
        ...

    @property
    def used_fuzzy_match(self) -> bool:
        """Whether fuzzy matching was used (False = exact match)."""
        ...

    @property
    def content_for_replacement(self) -> str:
        """The content to use for replacement operations."""
        ...

class DiffResult:
    """Result of diff generation."""

    @property
    def diff(self) -> str:
        """The unified diff string with line numbers."""
        ...

    @property
    def first_changed_line(self) -> Optional[int]:
        """The first line number that changed (in the new file)."""
        ...

def edit(
    path: str,
    search: str,
    replace: str,
    multiple: bool = False,
    ignore_whitespace: bool = False,
    encoding: str = "utf-8",
    dry_run: bool = False,
) -> EditResult:
    """Edit a file in place.

    Args:
        path: Path to the file to edit
        search: The string to search for
        replace: The replacement string
        multiple: If True, replace all occurrences
        ignore_whitespace: If True, treat consecutive whitespace as equivalent
        encoding: File encoding (default: "utf-8")
        dry_run: If True, don't actually modify the file

    Returns:
        EditResult with replacement count and detected line ending

    Raises:
        FileNotFoundError: If the file does not exist
        ValueError: If no matches found, multiple matches without multiple=True
        IOError: If there's an error writing the file
    """
    ...

def edit_fuzzy(
    path: str,
    old_text: str,
    new_text: str,
    multiple: bool = False,
    encoding: str = "utf-8",
    dry_run: bool = False,
) -> EditResultWithDiff:
    """Edit a file using fuzzy matching with diff output.

    This function provides fuzzy matching capabilities. It tries exact match
    first, then falls back to fuzzy matching (normalizing Unicode characters
    like smart quotes, dashes, and special spaces).

    Args:
        path: Path to the file to edit
        old_text: The text to find (will try exact match, then fuzzy)
        new_text: The replacement text
        multiple: If True, replace all occurrences
        encoding: File encoding (default: "utf-8")
        dry_run: If True, don't actually modify the file

    Returns:
        EditResultWithDiff with the modified content, diff, and match info

    Raises:
        FileNotFoundError: If the file does not exist
        ValueError: If no matches found, multiple matches without multiple=True
        IOError: If there's an error writing the file
    """
    ...

def replace_in_string(
    content: str,
    search: str,
    replace: str,
    multiple: bool = False,
    ignore_whitespace: bool = False,
) -> EditResult:
    """Replace text in a string (in-memory operation).

    Args:
        content: The text content to search in
        search: The string to search for
        replace: The replacement string
        multiple: If True, replace all occurrences
        ignore_whitespace: If True, treat consecutive whitespace as equivalent

    Returns:
        EditResult with the modified content and replacement count

    Raises:
        ValueError: If no matches found or multiple matches without multiple=True
    """
    ...

def read(path: str, encoding: str = "utf-8") -> Tuple[str, Optional[str]]:
    """Read a file and return its contents.

    Args:
        path: Path to the file
        encoding: File encoding (default: "utf-8")

    Returns:
        Tuple of (content, line_ending) where line_ending is "lf", "crlf", or None

    Raises:
        FileNotFoundError: If the file does not exist
        ValueError: If there's an encoding error
    """
    ...

def detect_line_ending(content: bytes) -> Optional[str]:
    """Detect line ending style from bytes.

    Uses first-occurrence detection: returns the style of the first
    line ending found in the content.

    Args:
        content: Raw bytes to analyze

    Returns:
        "lf", "crlf", or None if no line endings found
    """
    ...

def fuzzy_find(content: str, old_text: str) -> FuzzyMatchResult:
    """Find text using fuzzy matching.

    Tries exact match first, then falls back to fuzzy matching
    (normalizing trailing whitespace, Unicode quotes, dashes, and special spaces).

    Args:
        content: The text content to search in
        old_text: The text to find

    Returns:
        FuzzyMatchResult with match information
    """
    ...

def normalize_fuzzy(text: str) -> str:
    """Normalize text for fuzzy matching.

    Applies the following transformations:
    - Strip trailing whitespace from each line
    - Normalize smart quotes to ASCII equivalents
    - Normalize Unicode dashes/hyphens to ASCII hyphen
    - Normalize special Unicode spaces to regular space

    Args:
        text: The text to normalize

    Returns:
        The normalized text
    """
    ...

def diff(old_content: str, new_content: str, context_lines: int = 4) -> DiffResult:
    """Generate a unified diff between two strings.

    Args:
        old_content: The original content
        new_content: The new content
        context_lines: Number of context lines to include (default: 4)

    Returns:
        DiffResult with the diff string and first changed line
    """
    ...

def strip_bom_py(content: str) -> Tuple[str, str]:
    """Strip UTF-8 BOM from content if present.

    Args:
        content: The text content

    Returns:
        Tuple of (bom, text) where bom is the BOM string (or empty)
        and text is the content without BOM
    """
    ...

def normalize_line_endings(text: str) -> str:
    """Normalize line endings to LF.

    Converts both CRLF and standalone CR to LF.

    Args:
        text: The text to normalize

    Returns:
        Text with all line endings converted to LF
    """
    ...

class StructuredFormat:
    """Structured file format enum."""

    def __init__(self, name: str) -> None:
        """Create a StructuredFormat from a string name.

        Args:
            name: Format name (json, jsonc, json5, toml, yaml)

        Raises:
            ValueError: If format name is unknown
        """
        ...

    @staticmethod
    def json() -> "StructuredFormat":
        """Standard JSON format."""
        ...

    @staticmethod
    def jsonc() -> "StructuredFormat":
        """JSON with Comments (VS Code style) format."""
        ...

    @staticmethod
    def json5() -> "StructuredFormat":
        """JSON5 (relaxed JSON) format."""
        ...

    @staticmethod
    def toml() -> "StructuredFormat":
        """TOML format."""
        ...

    @staticmethod
    def yaml() -> "StructuredFormat":
        """YAML format."""
        ...

class StructuredEditResult:
    """Result of a structured edit operation."""

    @property
    def content(self) -> str:
        """The modified content."""
        ...

    @property
    def format(self) -> str:
        """The format that was used (e.g., 'json', 'toml')."""
        ...

    @property
    def key_path(self) -> str:
        """The key path that was modified."""
        ...

    @property
    def old_value(self) -> Optional[str]:
        """The old value (as string), if available."""
        ...

    @property
    def new_value(self) -> str:
        """The new value (as string)."""
        ...

    @property
    def line_ending(self) -> Optional[str]:
        """Detected line ending style ('lf', 'crlf', or None)."""
        ...

def edit_structured_file(
    path: str,
    key_path: str,
    new_value: str,
    format: Optional[str] = None,
    encoding: str = "utf-8",
    dry_run: bool = False,
) -> StructuredEditResult:
    """Edit a structured file (JSON, JSONC, JSON5, TOML, YAML) at a key path.

    Args:
        path: Path to the file to edit
        key_path: Dot-separated key path (e.g., "settings.port", "items[0].name")
        new_value: The new value (will be parsed as appropriate for the format)
        format: Optional format override ("json", "jsonc", "json5", "toml", "yaml").
                If not specified, the format is detected from the file extension.
        encoding: File encoding (default: "utf-8")
        dry_run: If True, don't actually modify the file

    Returns:
        StructuredEditResult with the modified content and metadata

    Raises:
        FileNotFoundError: If the file does not exist
        ValueError: If the key path is invalid, key not found, or parse error
        IOError: If there's an error writing the file
    """
    ...

def edit_structured_string(
    content: str,
    key_path: str,
    new_value: str,
    format: str,
) -> Tuple[str, Optional[str]]:
    """Edit structured content in-memory (without file I/O).

    Args:
        content: The content to edit
        key_path: Dot-separated key path (e.g., "settings.port", "items[0].name")
        new_value: The new value (will be parsed as appropriate for the format)
        format: Format of the content ("json", "jsonc", "json5", "toml", "yaml")

    Returns:
        Tuple of (new_content, old_value) where old_value may be None

    Raises:
        ValueError: If the format is unknown, key path is invalid, or parse error
    """
    ...
