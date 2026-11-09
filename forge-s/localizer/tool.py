import os
import json
import subprocess
from abc import ABC, abstractmethod
from typing import Dict, Any, List, Optional

# Import VFS
import sys

sys.path.append(os.path.dirname(os.path.abspath(__file__)))
from vfs import VirtualFileSystem


class BaseAgentTool(ABC):
    def __init__(self):
        self.vfs: Optional[VirtualFileSystem] = None

    def set_vfs(self, vfs: VirtualFileSystem):
        """Set the virtual file system for this tool."""
        self.vfs = vfs

    @property
    @abstractmethod
    def name(self) -> str:
        """Tool name, used by LLM to call it"""
        pass

    @property
    @abstractmethod
    def description(self) -> str:
        """Tool description, telling LLM what this tool does"""
        pass

    @property
    @abstractmethod
    def parameters(self) -> Dict[str, Any]:
        """Parameter definition in JSON Schema format"""
        pass

    @abstractmethod
    def execute(self, **kwargs) -> str:
        """
        Unified execution entry point.
        Returns: String (Agent can only read text)
        """
        pass

    def to_openai_schema(self) -> Dict[str, Any]:
        """Helper method: Generate OpenAI/General LLM compatible schema directly"""
        return {
            "type": "function",
            "function": {
                "name": self.name,
                "description": self.description,
                "parameters": self.parameters,
            },
        }


class ListDirectoryTool(BaseAgentTool):
    name = "list_directory"
    description = "List all file names and subfolder names under the specified directory path. All paths are relative to the project root. Use '.' for the root directory."

    parameters = {
        "type": "object",
        "properties": {
            "path": {
                "type": "string",
                "description": "Relative path of the directory to list (e.g., 'src' or '.' for root)",
            }
        },
        "required": ["path"],
    }

    def execute(self, path: str) -> str:
        try:
            if not self.vfs:
                return "Error: Virtual file system not initialized."

            # Check if path exists and is a directory
            if not self.vfs.exists(path):
                return f"Error: Path '{path}' does not exist."
            if not self.vfs.is_dir(path):
                return f"Error: Path '{path}' is not a directory."

            # Get file list using VFS
            items = self.vfs.listdir(path)

            # Filter out hidden files and add directory markers
            visible_items = []
            for item in items:
                if item.startswith("."):
                    continue

                # Check if it's a directory
                item_path = self.vfs.joinpath(path, item)
                if self.vfs.is_dir(item_path):
                    visible_items.append(item + "/")
                else:
                    visible_items.append(item)

            if not visible_items:
                return "The directory is empty."

            return json.dumps(visible_items, ensure_ascii=False)
        except Exception as e:
            return f"Error listing directory: {str(e)}"


class ReadFileTool(BaseAgentTool):
    name = "read_file"
    description = "Read the content of the file at the specified path. All paths are relative to the project root."

    parameters = {
        "type": "object",
        "properties": {
            "path": {
                "type": "string",
                "description": "Relative path of the file (e.g., 'src/lib.rs')",
            }
        },
        "required": ["path"],
    }

    def execute(self, path: str) -> str:
        try:
            if not self.vfs:
                return "Error: Virtual file system not initialized."

            # Check if path exists and is a file
            if not self.vfs.exists(path):
                return f"Error: File '{path}' does not exist."
            if not self.vfs.is_file(path):
                return f"Error: Path '{path}' is not a file."

            # Read file using VFS (limit to 100KB)
            content = self.vfs.read_file(path, max_bytes=100000)
            return content

        except Exception as e:
            return f"Error reading file: {str(e)}"


class SearchFileTool(BaseAgentTool):
    name = "search_file"
    description = "Search for a specific string in all files under a directory recursively. Returns file paths (relative to project root) and matching lines. Equivalent to `grep -r`."

    parameters = {
        "type": "object",
        "properties": {
            "directory": {
                "type": "string",
                "description": "The directory to search in (relative path, use '.' for project root)",
            },
            "query": {
                "type": "string",
                "description": "The string to search for.",
            },
        },
        "required": ["directory", "query"],
    }

    def execute(self, directory: str, query: str) -> str:
        try:
            if not self.vfs:
                return "Error: Virtual file system not initialized."

            # Validate directory exists
            if not self.vfs.exists(directory) or not self.vfs.is_dir(directory):
                return f"Error: Directory '{directory}' does not exist."

            # Convert to real path for grep
            real_dir = self.vfs.to_real_path(directory)

            # Use system grep for performance
            # -r: recursive
            # -n: show line numbers
            # -I: ignore binary files
            command = ["grep", "-r", "-n", "-I", query, str(real_dir)]

            result = subprocess.run(
                command, capture_output=True, text=True, check=False
            )

            if result.returncode == 0:
                lines = result.stdout.strip().split("\n")

                # Convert absolute paths back to virtual paths and get context
                formatted_results = []
                for line in lines[:50]:  # Limit to 50 results
                    # Line format: /abs/path/to/file.rs:123:content
                    parts = line.split(":", 2)
                    if len(parts) >= 3:
                        abs_path = parts[0]
                        line_no = int(parts[1])

                        try:
                            # Convert to virtual path
                            virt_path = self.vfs.to_virtual_path(abs_path)

                            # Read the file to get context (12 lines before and after)
                            file_content = self.vfs.read_file(virt_path)
                            file_lines = file_content.split("\n")

                            # Calculate range (line_no is 1-indexed)
                            start_idx = max(0, line_no - 13)  # 12 lines before
                            end_idx = min(
                                len(file_lines), line_no + 12
                            )  # 12 lines after

                            # Get context lines
                            context_lines = file_lines[start_idx:end_idx]

                            # Format the result with line numbers
                            header = f"\n=== {virt_path}:{line_no} ==="
                            context = "\n".join(
                                [
                                    f"{start_idx + i + 1:4d}: {context_lines[i]}"
                                    for i in range(len(context_lines))
                                ]
                            )

                            formatted_results.append(header + "\n" + context)

                        except ValueError:
                            # If conversion fails, skip this line
                            continue
                        except Exception:
                            # If file reading fails, skip
                            continue

                if formatted_results:
                    return "\n".join(formatted_results)
                else:
                    return "No matches found."

            elif result.returncode == 1:
                return "No matches found."
            else:
                return f"Error executing grep: {result.stderr}"

        except Exception as e:
            return f"Error searching files: {str(e)}"


class ReportTool(BaseAgentTool):
    name = "report_finding"
    description = "Report the located file paths for the security findings. Use this tool when you have found all the file locations for the given findings and are ready to submit your results."

    parameters = {
        "type": "object",
        "properties": {
            "findings": {
                "type": "object",
                "description": "A dictionary where keys are finding IDs (as strings or integers) and values are lists of file paths relative to the project root.",
                "additionalProperties": {
                    "type": "array",
                    "items": {"type": "string"},
                },
            }
        },
        "required": ["findings"],
    }

    def execute(self, findings: Dict[str, List[str]]) -> str:
        """
        Report findings locations.
        Returns a special marker to indicate completion.
        """
        try:
            # Validate that findings is a dict
            if not isinstance(findings, dict):
                return "Error: 'findings' must be a dictionary."

            # Store the findings data in a special format that the agent can detect
            result = {"__REPORT_COMPLETE__": True, "findings": findings}

            # Return JSON string with special marker
            return json.dumps(result, ensure_ascii=False)
        except Exception as e:
            return f"Error reporting findings: {str(e)}"


class ToolRegistry:
    def __init__(self):
        self._tools: Dict[str, BaseAgentTool] = {}
        self._vfs: Optional[VirtualFileSystem] = None

    def register(self, tool: BaseAgentTool):
        self._tools[tool.name] = tool
        # If VFS is already set, apply it to the new tool
        if self._vfs:
            tool.set_vfs(self._vfs)

    def set_vfs(self, vfs: VirtualFileSystem):
        """Set VFS for all registered tools."""
        self._vfs = vfs
        for tool in self._tools.values():
            tool.set_vfs(vfs)

    def get_schemas(self) -> List[Dict]:
        """Get schemas of all tools to pass to LLM"""
        return [tool.to_openai_schema() for tool in self._tools.values()]

    def call(self, tool_name: str, tool_args: str | Dict) -> str:
        """Unified call interface, handling parameter parsing and exceptions"""
        tool = self._tools.get(tool_name)
        if not tool:
            return f"Error: Tool '{tool_name}' not found."

        try:
            # If LLM returns a JSON string, parse it first
            if isinstance(tool_args, str):
                kwargs = json.loads(tool_args)
            else:
                kwargs = tool_args

            # print(f"🛠️  [Agent Action] Calling {tool_name} with {kwargs}")
            return tool.execute(**kwargs)

        except json.JSONDecodeError:
            return "Error: Parameter format error, unable to parse JSON."
        except Exception as e:
            return f"Error executing tool: {str(e)}"
