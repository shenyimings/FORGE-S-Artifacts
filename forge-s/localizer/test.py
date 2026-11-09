import os
import sys

# Ensure the current directory is in the path so we can import tool.py
sys.path.append(os.path.dirname(os.path.abspath(__file__)))

from tool import ToolRegistry, ListDirectoryTool, ReadFileTool, SearchFileTool


def test_tools():
    print("Starting Tool Tests...\n")

    registry = ToolRegistry()
    registry.register(ListDirectoryTool())
    registry.register(ReadFileTool())
    registry.register(SearchFileTool())

    # --- Test 1: ListDirectoryTool ---
    print("--- Testing ListDirectoryTool ---")
    path = os.path.dirname(os.path.abspath(__file__))  # Current dir
    params = {"path": path}
    print(f"Parameters: {params}")
    result = registry.call("list_directory", params)
    print(f"Result: {result}\n")

    # --- Test 2: ReadFileTool ---
    print("--- Testing ReadFileTool ---")
    # Create a dummy file for testing
    dummy_file = os.path.join(path, "test_dummy.txt")
    with open(dummy_file, "w") as f:
        f.write("This is a secret_key for testing ReadFileTool.")

    params = {"path": dummy_file}
    print(f"Parameters: {params}")
    result = registry.call("read_file", params)
    print(f"Result: {result}\n")

    # --- Test 3: SearchFileTool ---
    print("--- Testing SearchFileTool ---")
    query = "secret_key"
    params = {"directory": path, "query": query}
    print(f"Parameters: {params}")
    result = registry.call("search_file", params)
    print(f"Result: {result}\n")

    # Clean up
    if os.path.exists(dummy_file):
        os.remove(dummy_file)
        print("Cleaned up test file.")


if __name__ == "__main__":
    test_tools()
