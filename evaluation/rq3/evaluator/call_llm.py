"""LLM client for the RQ3 detection evaluation.

All models are called through OpenRouter's OpenAI-compatible API.
Set `API_KEY` (your OpenRouter key) in a `.env` file or the environment;
`BASE_URL` can optionally point to another OpenAI-compatible endpoint.
"""

import os
from typing import Dict, List, Tuple

import openai
from dotenv import load_dotenv

load_dotenv()

BASE_URL = os.getenv("BASE_URL", "https://openrouter.ai/api/v1")

# Short names used in the paper -> OpenRouter model IDs
MODELS = {
    "gpt-5.2": "openai/gpt-5.2",
    "claude-sonnet-4.5": "anthropic/claude-sonnet-4.5",
    "deepseek-r1": "deepseek/deepseek-r1-0528",
    "qwen3": "qwen/qwen3-235b-a22b-2507",
}


def create_messages(*args: Tuple[str, str]) -> List[Dict[str, str]]:
    return [{"role": role, "content": content} for role, content in args]


def call_llm(
    messages: List[Dict[str, str]], model: str = "qwen3"
) -> Tuple[str, int, int, int]:
    """
    Calls the LLM with the specified messages and model.

    Args:
        messages: List of message dictionaries.
        model: Key from MODELS or a raw OpenRouter model ID. Defaults to "qwen3".

    Returns:
        (content, prompt_tokens, completion_tokens, total_tokens)
    """
    client = openai.OpenAI(base_url=BASE_URL, api_key=os.getenv("API_KEY"))
    response = client.chat.completions.create(
        model=MODELS.get(model, model),
        messages=messages,
        temperature=0,
    )
    return (
        response.choices[0].message.content,
        response.usage.prompt_tokens,
        response.usage.completion_tokens,
        response.usage.total_tokens,
    )


if __name__ == "__main__":
    msgs = create_messages(("system", "You are a kind assistant."), ("user", "hi"))
    for name in MODELS:
        print(f"Model: {name}")
        print(call_llm(msgs, model=name))
