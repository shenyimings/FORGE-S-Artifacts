import json
import re
import sys
import os
import yaml
from typing import List, Dict, Any, Optional
from loguru import logger


# Ensure we can import modules from the same directory
sys.path.append(os.path.dirname(os.path.abspath(__file__)))


from call_llm import call_llm, create_messages
from tool import (
    ToolRegistry,
    ListDirectoryTool,
    ReadFileTool,
    SearchFileTool,
    ReportTool,
)

# Modular Prompt Template
REACT_PROMPT_TEMPLATE = """
You are a smart autonomous research agent.
Your task is:
{task}

You have access to the following tools:
{tool_descriptions}

You must output your response in a single YAML block wrapped in markdown code fences (```yaml).
The YAML must have the following structure:

```yaml
thought: |
  Your reasoning process here.
action: tool_name
parameters:
  param_name: param_value
```

Set `action` to one of the available tool names and provide the appropriate `parameters`.
Valid tools: [{tool_names}]

When you have located all the file paths for the findings, use the `report_finding` tool to submit your results.

Example of using a tool:
```yaml
thought: |
  I need to list files to see the structure.
action: list_directory
parameters:
  path: .
```

Example of reporting findings:
```yaml
thought: |
  I have located all the files. Now I will report the findings.
action: report_finding
parameters:
  findings:
    0:
      - path/to/file1.rs
      - path/to/file2.rs
    1:
      - path/to/file3.rs
```

Begin!

History:
{history}
"""


class ReActAgent:
    def __init__(
        self,
        registry: ToolRegistry,
        model: str = "qwen-next",
        max_steps: int = 15,
        max_retries: int = 3,
    ):
        self.registry = registry
        self.model = model
        self.max_steps = max_steps
        self.max_retries = max_retries
        self.history = []

    def _build_tool_descriptions(self) -> str:
        """Dynamically build tool descriptions from the registry."""
        schemas = self.registry.get_schemas()
        descriptions = []
        for schema in schemas:
            func = schema["function"]
            # Format nicely for the LLM to understand
            desc = f"- {func['name']}: {func['description']}\n  Parameters: {json.dumps(func['parameters'])}"
            descriptions.append(desc)
        return "\n".join(descriptions)

    def _build_tool_names(self) -> str:
        return ", ".join(self.registry._tools.keys())

    def run(self, task: str) -> str:
        """
        Execute the ReAct loop for the given task.
        """
        self.history = []
        tool_desc = self._build_tool_descriptions()
        tool_names = self._build_tool_names()

        # logger.debug(f"Agent started with task: {task[:100]}...")

        for step in range(self.max_steps):
            # Construct prompt with current history
            if not self.history:
                self.history.append("")
            history_text = "\n".join(self.history)
            prompt_content = REACT_PROMPT_TEMPLATE.format(
                task=task,
                tool_descriptions=tool_desc,
                tool_names=tool_names,
                history=history_text,
            )

            messages = create_messages(("user", prompt_content))

            # print(f"\n--- Iteration {step + 1} ---")
            # Call LLM with retry logic
            response = None
            for retry in range(self.max_retries):
                try:
                    response = call_llm(messages, model=self.model)
                    # print(f"LLM Response:\n{response}\n")
                    break  # Success, exit retry loop
                except Exception as e:
                    error_msg = f"Error calling LLM (attempt {retry + 1}/{self.max_retries}): {e}"
                    logger.warning(error_msg)

                    if retry < self.max_retries - 1:
                        # Add failure reason to history for next retry
                        failure_note = f"[System Notice] Previous LLM call failed: {str(e)}. Please try again with a different approach."
                        self.history.append(failure_note)

                        # Reconstruct messages with updated history
                        history_text = "\n".join(self.history)
                        prompt_content = REACT_PROMPT_TEMPLATE.format(
                            task=task,
                            tool_descriptions=tool_desc,
                            tool_names=tool_names,
                            history=history_text,
                        )
                        messages = create_messages(("user", prompt_content))
                    else:
                        # Max retries reached
                        return (
                            f"Error calling LLM after {self.max_retries} attempts: {e}"
                        )

            if response is None:
                return "Error: No response received from LLM"

            # Parse YAML Response
            try:
                # Extract YAML block
                match = re.search(r"```yaml\s*(.*?)\s*```", response, re.DOTALL)
                if not match:
                    # Try without language specifier
                    match = re.search(r"```\s*(.*?)\s*```", response, re.DOTALL)

                if not match:
                    raise ValueError("No YAML block found in response.")

                yaml_content = match.group(1)
                data = yaml.safe_load(yaml_content)

                if not isinstance(data, dict):
                    raise ValueError("Parsed YAML is not a dictionary.")

                thought = data.get("thought", "")
                action = data.get("action", "")
                params = data.get("parameters", {})

                logger.debug(f"Thought: {thought}")

                # Execute tool
                logger.debug(f"Executing Tool: {action} with input: {params}")

                # The registry handles JSON parsing if string, but here we have a dict.
                result = self.registry.call(action, params)

                # Check if this is a report completion
                try:
                    result_data = json.loads(result)
                    if isinstance(result_data, dict) and result_data.get(
                        "__REPORT_COMPLETE__"
                    ):
                        # Extract findings and return in YAML format
                        findings = result_data.get("findings", {})
                        return yaml.dump(findings, default_flow_style=False)
                except (json.JSONDecodeError, TypeError):
                    pass

                observation = f"Observation: {result}"

            except Exception as e:
                observation = f"Observation: Error parsing response or executing tool: {str(e)}. Please ensure you output valid YAML within code fences."
                logger.warning(f"⚠️ {observation}")

            logger.debug(
                f" {observation[:400]}..."
                if len(observation) > 400
                else f" {observation}"
            )

            # Append the full LLM response and the observation to history
            if response:
                self.history.append(response)
            if observation:
                self.history.append(observation)

        return "Max steps reached without Final Answer."


def generate_audit_task(project_path: str, findings: List[Dict[str, str]]) -> str:
    """
    Generates a specific task description for the audit finding location task.
    """
    task_desc = f"""
You are a security researcher helper working in a sandboxed file system.

Your goal is to locate all related file paths for the following security findings.
The project root directory is '{project_path}' - all paths you use and report must be relative to this root.

**Important Path Guidelines:**
- Use relative paths like 'src/lib.rs', 'contracts/token.sol'
- Use '.' to refer to the project root directory
- DO NOT use absolute paths like '/home/user/...' or '/Users/...'
- When listing directories, start with '{project_path}' to see the project structure

The provided 'Hint Location' in the findings might be inaccurate, relative, or just a filename without a path.
You need to use the search tools to find the actual file path relative to the project root.
If a location is vague (e.g., just a filename), search for that filename.
If a location seems wrong, use keywords from the Title or Description to search for the relevant code.

Findings to locate:
"""
    for f in findings:
        task_desc += f"- ID: {f.get('id')}\n  Title: {f.get('title')}\n  Description: {f.get('description')}\n  Hint Location: {f.get('location')}\n\n"

    task_desc += """
When you finish locating the files, use the `report_finding` tool to submit your results.
The `findings` parameter should be a dictionary where:
- Keys are the finding IDs (as strings or integers).
- Values are lists of file paths **relative to the project root**.

For example:
```yaml
action: report_finding
parameters:
  findings:
    0:
      - src/lib.rs
      - src/utils.rs
    1:
      - contracts/token.sol
```

**Remember:** All paths in your report must be relative paths, not absolute paths.
If you cannot find a file for a specific finding, you may omit that finding ID from the report or provide an empty list.
"""
    return task_desc


if __name__ == "__main__":
    # --- Test Setup ---
    from vfs import VirtualFileSystem

    # 1. Setup Tools
    registry = ToolRegistry()
    registry.register(ListDirectoryTool())
    registry.register(ReadFileTool())
    registry.register(SearchFileTool())
    registry.register(ReportTool())

    # 2. Setup Agent
    agent = ReActAgent(registry=registry, model=None, max_steps=15, max_retries=3)

    # 3. Setup VFS - Use current directory as test project
    current_dir = os.path.dirname(os.path.abspath(__file__))
    vfs = VirtualFileSystem(current_dir)
    registry.set_vfs(vfs)

    # 4. Define a Mock Task
    mock_findings = [
        {
            "id": "0",
            "title": "Hardcoded API Key",
            "description": "There is a hardcoded string that looks like a secret key.",
            "location": "test.py",  # Hint is correct
        },
        {
            "id": "1",
            "title": "LLM Call Function",
            "description": "The function that calls the LLM API.",
            "location": "llm_utils.py",  # Hint is wrong, actual file is call_llm.py
        },
    ]

    # Use "." as the project path since VFS handles the real path
    task = generate_audit_task(".", mock_findings)

    # 5. Run Agent
    result = agent.run(task)

    # print("\n\n====== FINAL RESULT ======")
    logger.debug(result)
