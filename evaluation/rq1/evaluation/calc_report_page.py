#!/usr/bin/env python3
"""
Token Counter Script
Calculates token counts for PDF and Markdown files in a folder using tiktoken.
"""

import sys
from pathlib import Path
import tiktoken
from typing import List, Tuple

# For PDF text extraction
try:
    import fitz  # PyMuPDF
except ImportError:
    print("PyMuPDF not installed. Installing...")
    import subprocess

    subprocess.check_call(
        [sys.executable, "-m", "pip", "install", "PyMuPDF", "--break-system-packages"]
    )
    import fitz


def extract_text_from_pdf(pdf_path: Path) -> str:
    """Extract text content from a PDF file using PyMuPDF."""
    text = ""
    try:
        doc = fitz.open(pdf_path)
        for page in doc:
            text += page.get_text()
        doc.close()
    except Exception as e:
        print(f"Error reading {pdf_path}: {e}")
    return text


def extract_text_from_md(md_path: Path) -> str:
    """Extract text content from a Markdown file."""
    try:
        with open(md_path, "r", encoding="utf-8") as file:
            return file.read()
    except Exception as e:
        print(f"Error reading {md_path}: {e}")
        return ""


def count_tokens(text: str, encoding) -> int:
    """Count tokens in text using tiktoken encoding."""
    return len(encoding.encode(text))


def process_folder(folder_path: str) -> None:
    """Process all PDF and MD files in the folder and calculate token statistics."""

    # Initialize tiktoken encoding for gpt-4o
    enc = tiktoken.encoding_for_model("gpt-4o")

    folder = Path(folder_path)

    if not folder.exists():
        print(f"Error: Folder '{folder_path}' does not exist.")
        return

    if not folder.is_dir():
        print(f"Error: '{folder_path}' is not a directory.")
        return

    # Find all PDF and MD files
    pdf_files = list(folder.glob("*.pdf"))
    md_files = list(folder.glob("*.md"))
    all_files = pdf_files + md_files

    if not all_files:
        print(f"No PDF or Markdown files found in '{folder_path}'")
        return

    print(f"Found {len(pdf_files)} PDF file(s) and {len(md_files)} Markdown file(s)")
    print("=" * 80)

    token_counts: List[Tuple[Path, int]] = []

    # Process each file
    for file_path in all_files:
        print(f"Processing: {file_path.name}...", end=" ")

        # Extract text based on file type
        if file_path.suffix.lower() == ".pdf":
            text = extract_text_from_pdf(file_path)
        else:  # .md
            text = extract_text_from_md(file_path)

        # Count tokens
        token_count = count_tokens(text, enc)
        token_counts.append((file_path, token_count))

        print(f"{token_count:,} tokens")

    print("=" * 80)

    # Calculate statistics
    total_tokens = sum(count for _, count in token_counts)
    average_tokens = total_tokens / len(token_counts) if token_counts else 0

    print(f"\nSummary:")
    print(f"Total files processed: {len(token_counts)}")
    print(f"Total tokens: {total_tokens:,}")
    print(f"Average tokens per file: {average_tokens:,.2f}")

    # Show top 5 files by token count
    if len(token_counts) > 1:
        print(f"\nTop 5 files by token count:")
        sorted_counts = sorted(token_counts, key=lambda x: x[1], reverse=True)[:5]
        for i, (path, count) in enumerate(sorted_counts, 1):
            print(f"  {i}. {path.name}: {count:,} tokens")


def main():
    if len(sys.argv) != 2:
        print("Usage: python token_counter.py <folder_path>")
        print("Example: python token_counter.py ./documents")
        sys.exit(1)

    folder_path = sys.argv[1]
    process_folder(folder_path)


if __name__ == "__main__":
    main()


'''
Solana:
Total files processed: 199
Total tokens: 1,627,268
Average tokens per file: 8,177.23
'''

'''
EVM:
Total files processed: 6454
Total tokens: 31,020,610
Average tokens per file: 4,806.42
'''