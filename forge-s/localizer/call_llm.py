"""LLM client for the localization agent.

Uses the same `llm` section of `config.yaml` (model, base_url, temperature) and
the same `API_KEY` from the `.env` file as the other FORGE-S stages, so every
stage talks to one OpenAI-compatible endpoint (OpenRouter by default).
"""

import os
from typing import Dict, List, Optional, Tuple

import dotenv
import openai
import yaml

_FORGE_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def _load_config() -> dict:
    # Prefer the config in the working directory (as the other stages do),
    # fall back to forge-s/config.yaml when run from elsewhere.
    for path in ("config.yaml", os.path.join(_FORGE_ROOT, "config.yaml")):
        if os.path.exists(path):
            with open(path, "r") as f:
                return yaml.safe_load(f)
    raise FileNotFoundError("config.yaml not found")


CONFIG = _load_config()
_env_path = CONFIG["global"].get("env_path", ".env")
if not os.path.isabs(_env_path) and not os.path.exists(_env_path):
    _env_path = os.path.join(_FORGE_ROOT, _env_path)
dotenv.load_dotenv(_env_path)

MODEL = CONFIG["llm"]["model"]
BASE_URL = CONFIG["llm"].get("base_url")
TEMPERATURE = CONFIG["llm"].get("parameters", {}).get("temperature", 0)

CLIENT = openai.OpenAI(
    api_key=os.getenv("API_KEY") or "ollama",
    base_url=BASE_URL,
    timeout=CONFIG["global"].get("timeout", 60),
    max_retries=CONFIG["global"].get("max_retries", 3),
)


def create_messages(*args: Tuple[str, str]) -> List[Dict[str, str]]:
    return [{"role": role, "content": content} for role, content in args]


def call_llm(messages: List[Dict[str, str]], model: Optional[str] = None) -> str:
    """
    Calls the configured LLM with the given messages.

    Args:
        messages: List of message dictionaries.
        model: Model ID on the configured endpoint. Defaults to `llm.model` in config.yaml.
    """
    response = CLIENT.chat.completions.create(
        model=model or MODEL,
        messages=messages,
        temperature=TEMPERATURE,
    )
    return response.choices[0].message.content


if __name__ == "__main__":
    msgs = create_messages(("system", "You are a kind assistant."), ("user", "hi"))
    print(f"Testing with model: {MODEL}")
    print(call_llm(msgs))
