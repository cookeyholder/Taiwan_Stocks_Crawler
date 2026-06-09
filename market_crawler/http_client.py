from __future__ import annotations

import time
from dataclasses import dataclass, field
from typing import Any

import requests


@dataclass(slots=True)
class HttpPolicy:
    timeout_seconds: float = 10.0
    retry_count: int = 3
    backoff_seconds: float = 0.3
    # 選用：自訂 HTTP 請求標頭（例如 User-Agent）
    headers: dict[str, Any] | None = field(default=None)
    # 選用：是否驗證 SSL 憑證。政府機關網站（TWSE/TPEX）的舊憑證有時不符合規範，
    # 建議在信任的來源設為 False 並搭配 urllib3 警告遮蔽。
    ssl_verify: bool = True


def get_with_retry(
    url: str,
    policy: HttpPolicy,
    session: requests.Session | None = None,
) -> requests.Response:
    last_exception: Exception | None = None
    for attempt in range(policy.retry_count + 1):
        try:
            getter = session.get if session is not None else requests.get
            response = getter(
                url,
                timeout=policy.timeout_seconds,
                headers=policy.headers,
                verify=policy.ssl_verify,
            )
            response.raise_for_status()
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
