from __future__ import annotations

import time
from dataclasses import dataclass

import requests


@dataclass(slots=True)
class HttpPolicy:
    timeout_seconds: float = 10.0
    retry_count: int = 3
    backoff_seconds: float = 0.3


def get_with_retry(url: str, policy: HttpPolicy) -> requests.Response:
    last_exception: Exception | None = None
    for attempt in range(policy.retry_count + 1):
        try:
            response = requests.get(url, timeout=policy.timeout_seconds)
            if response.status_code >= 500:
                raise requests.HTTPError(f"server error {response.status_code}")
            return response
        except Exception as exc:  # noqa: BLE001
            last_exception = exc
            if attempt == policy.retry_count:
                break
            sleep_seconds = policy.backoff_seconds * (2**attempt)
            time.sleep(sleep_seconds)
    if last_exception is None:
        raise RuntimeError("Unknown HTTP failure")
    raise RuntimeError(f"HTTP request failed after retries: {last_exception}")
