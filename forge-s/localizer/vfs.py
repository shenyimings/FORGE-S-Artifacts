"""
Virtual File System (VFS) for Localizer

Provides a sandboxed file system abstraction that:
1. Restricts all file operations to a specific root directory
2. Uses relative paths consistently (LLM sees relative paths, not absolute)
3. Automatically converts between relative and absolute paths
4. Prevents directory traversal attacks
"""

import os
from pathlib import Path
from typing import Union, Optional


class VirtualFileSystem:
    """
    A virtual file system that maps a real directory to a virtual root.
    All paths exposed to the LLM are relative to this virtual root.
    """

    def __init__(self, root_path: Union[str, Path]):
        """
        Initialize the virtual file system.

        Args:
            root_path: The real absolute path that will be the virtual root
        """
        self.root_path = Path(root_path).resolve()

        if not self.root_path.exists():
            raise ValueError(f"Root path does not exist: {self.root_path}")

        if not self.root_path.is_dir():
            raise ValueError(f"Root path is not a directory: {self.root_path}")

        # Virtual root is always "."
        self.virtual_root = "."

    def to_real_path(self, virtual_path: Union[str, Path]) -> Path:
        """
        Convert a virtual (relative) path to a real absolute path.

        Args:
            virtual_path: Relative path from virtual root (e.g., "src/lib.rs")

        Returns:
            Absolute path in the real file system

        Raises:
            ValueError: If the path tries to escape the root directory
        """
        # Handle "." as current directory (virtual root)
        if virtual_path == "." or virtual_path == "":
            return self.root_path

        # Construct the real path
        virtual_path = Path(virtual_path)

        # Join with root and resolve to handle any ".." or "."
        real_path = (self.root_path / virtual_path).resolve()

        # Security check: ensure the resolved path is still within root
        try:
            real_path.relative_to(self.root_path)
        except ValueError:
            raise ValueError(
                f"Path '{virtual_path}' attempts to escape root directory. "
                f"Resolved to: {real_path}, Root: {self.root_path}"
            )

        return real_path

    def to_virtual_path(self, real_path: Union[str, Path]) -> str:
        """
        Convert a real absolute path to a virtual (relative) path.

        Args:
            real_path: Absolute path in the real file system

        Returns:
            Relative path from virtual root

        Raises:
            ValueError: If the path is not within the root directory
        """
        real_path = Path(real_path).resolve()

        try:
            relative = real_path.relative_to(self.root_path)
            return str(relative) if str(relative) != "." else "."
        except ValueError:
            raise ValueError(
                f"Path '{real_path}' is not within root directory '{self.root_path}'"
            )

    def exists(self, virtual_path: Union[str, Path]) -> bool:
        """Check if a virtual path exists."""
        try:
            real_path = self.to_real_path(virtual_path)
            return real_path.exists()
        except (ValueError, OSError):
            return False

    def is_file(self, virtual_path: Union[str, Path]) -> bool:
        """Check if a virtual path is a file."""
        try:
            real_path = self.to_real_path(virtual_path)
            return real_path.is_file()
        except (ValueError, OSError):
            return False

    def is_dir(self, virtual_path: Union[str, Path]) -> bool:
        """Check if a virtual path is a directory."""
        try:
            real_path = self.to_real_path(virtual_path)
            return real_path.is_dir()
        except (ValueError, OSError):
            return False

    def listdir(self, virtual_path: Union[str, Path] = ".") -> list:
        """
        List contents of a virtual directory.

        Returns:
            List of filenames (not full paths)
        """
        real_path = self.to_real_path(virtual_path)

        if not real_path.is_dir():
            raise NotADirectoryError(f"'{virtual_path}' is not a directory")

        return os.listdir(real_path)

    def read_file(self, virtual_path: Union[str, Path], max_bytes: int = 100000) -> str:
        """
        Read contents of a virtual file.

        Args:
            virtual_path: Path to the file
            max_bytes: Maximum number of bytes to read

        Returns:
            File contents as string
        """
        real_path = self.to_real_path(virtual_path)

        if not real_path.is_file():
            raise FileNotFoundError(f"File not found: '{virtual_path}'")

        with open(real_path, "r", encoding="utf-8", errors="ignore") as f:
            return f.read(max_bytes)

    def joinpath(self, *parts: Union[str, Path]) -> str:
        """
        Join path components and return a virtual path.

        Args:
            *parts: Path components to join

        Returns:
            Joined virtual path as string
        """
        # Join the parts
        joined = Path(parts[0])
        for part in parts[1:]:
            joined = joined / part

        # Normalize the path
        return str(joined)

    def get_root_description(self) -> str:
        """
        Get a description of the virtual file system for the LLM.

        Returns:
            Description string
        """
        return (
            f"You are working in a virtual file system rooted at '{self.virtual_root}'. "
            f"All paths should be relative to this root. "
            f"For example, use 'src/lib.rs' instead of '/absolute/path/to/src/lib.rs'. "
            f"The virtual root '.' represents the project directory."
        )

    def __str__(self) -> str:
        return f"VFS(root={self.root_path})"

    def __repr__(self) -> str:
        return f"VirtualFileSystem(root_path='{self.root_path}')"


# Example usage
if __name__ == "__main__":
    # Create a VFS for a contracts directory
    vfs = VirtualFileSystem("/path/to/contracts")

    # Convert virtual to real
    real = vfs.to_real_path("src/lib.rs")
    print(f"Virtual 'src/lib.rs' -> Real: {real}")

    # Convert real to virtual
    virtual = vfs.to_virtual_path("/path/to/contracts/src/lib.rs")
    print(f"Real '/path/to/contracts/src/lib.rs' -> Virtual: {virtual}")

    # Check existence
    exists = vfs.exists("src/lib.rs")
    print(f"'src/lib.rs' exists: {exists}")

    # List directory
    files = vfs.listdir(".")
    print(f"Root directory contents: {files}")
