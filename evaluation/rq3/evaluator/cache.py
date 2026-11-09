import hashlib
import json
import os
from pathlib import Path
from typing import List, Dict, Tuple, Optional

CACHE_DIR = Path("evaluation/rq3/auditor/__llmcache__")
CACHE_DIR.mkdir(parents=True, exist_ok=True)


class LLMCache:
    def __init__(self, cache_dir: Path = CACHE_DIR):
        self.cache_dir = cache_dir

    def _compute_hash(self, messages: List[Dict[str, str]], model: str) -> str:
        """
        Compute a unique hash for the input messages and model.
        """
        # Normalize messages to ensure consistent hashing
        # We use json.dumps with sort_keys=True to ensure deterministic output
        data = {"model": model, "messages": messages}
        serialized = json.dumps(data, sort_keys=True)
        return hashlib.sha256(serialized.encode("utf-8")).hexdigest()

    def get(
        self, messages: List[Dict[str, str]], model: str
    ) -> Optional[Tuple[str, int, int, int]]:
        """
        Retrieve cached response if it exists.
        Returns: (content, prompt_tokens, completion_tokens, total_tokens) or None
        """
        cache_key = self._compute_hash(messages, model)
        cache_file = self.cache_dir / f"{cache_key}.json"

        if cache_file.exists():
            try:
                with open(cache_file, "r") as f:
                    data = json.load(f)
                return (
                    data["content"],
                    data["prompt_tokens"],
                    data["completion_tokens"],
                    data["total_tokens"],
                )
            except Exception as e:
                print(f"Error reading cache file {cache_file}: {e}")
                return None
        return None

    def set(
        self,
        messages: List[Dict[str, str]],
        model: str,
        content: str,
        prompt_tokens: int,
        completion_tokens: int,
        total_tokens: int,
    ):
        """
        Save response to cache.
        """
        cache_key = self._compute_hash(messages, model)
        cache_file = self.cache_dir / f"{cache_key}.json"

        data = {
            "content": content,
            "prompt_tokens": prompt_tokens,
            "completion_tokens": completion_tokens,
            "total_tokens": total_tokens,
        }

        try:
            with open(cache_file, "w") as f:
                json.dump(data, f, indent=2)
        except Exception as e:
            print(f"Error writing cache file {cache_file}: {e}")


# Global cache instance
_cache = LLMCache()


def cached_call_llm(
    original_call_llm_func, messages: List[Dict[str, str]], model: str = "qwen-next"
) -> Tuple[str, int, int, int]:
    """
    Wrapper function to add caching to call_llm.
    """
    # Try to get from cache
    cached_result = _cache.get(messages, model)
    if cached_result:
        # print(f"Cache hit for model {model}")
        return cached_result

    # If not in cache, call the original function
    # print(f"Cache miss for model {model}, calling API...")
    result = original_call_llm_func(messages, model=model)

    # Save to cache
    content, p_tokens, c_tokens, t_tokens = result
    _cache.set(messages, model, content, p_tokens, c_tokens, t_tokens)

    return result
