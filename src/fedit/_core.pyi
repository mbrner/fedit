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
        """Detected line ending style ('\\n', '\\r\\n', or None)."""
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
        Tuple of (content, line_ending) where line_ending is "\\n", "\\r\\n", or None

    Raises:
        FileNotFoundError: If the file does not exist
        ValueError: If there's an encoding error
    """
    ...

def detect_line_ending(content: bytes) -> Optional[str]:
    """Detect line ending style from bytes.

    Args:
        content: Raw bytes to analyze

    Returns:
        "\\n", "\\r\\n", or None if no line endings found
    """
    ...
