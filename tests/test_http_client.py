from __future__ import annotations

import requests
from pytest import MonkeyPatch

import market_crawler.http_client as http_client
from market_crawler.http_client import HttpPolicy, get_with_retry


def test_get_with_retry_success_first_try(monkeypatch: MonkeyPatch) -> None:
    class DummyResponse:
        status_code = 200

    def fake_get(_url: str, timeout: float) -> DummyResponse:
        assert timeout == 0.5
        return DummyResponse()

    monkeypatch.setattr(requests, "get", fake_get)
    response = get_with_retry("https://example.com", HttpPolicy(timeout_seconds=0.5, retry_count=1))
    assert response.status_code == 200


def test_get_with_retry_retries_then_succeeds(monkeypatch: MonkeyPatch) -> None:
    class DummyResponse:
        status_code = 200

    calls = {"n": 0}

    def fake_get(_url: str, timeout: float) -> DummyResponse:
        _ = timeout
        calls["n"] += 1
        if calls["n"] < 3:
            raise requests.Timeout("timeout")
        return DummyResponse()

    sleep_calls: list[float] = []
    monkeypatch.setattr(requests, "get", fake_get)
    def fake_sleep(seconds: float) -> None:
        sleep_calls.append(seconds)

    monkeypatch.setattr(http_client.time, "sleep", fake_sleep)

    response = get_with_retry(
        "https://example.com",
        HttpPolicy(timeout_seconds=1.0, retry_count=3, backoff_seconds=0.1),
    )
    assert response.status_code == 200
    assert calls["n"] == 3
    assert sleep_calls == [0.1, 0.2]


def test_get_with_retry_raises_after_all_retries(monkeypatch: MonkeyPatch) -> None:
    def fake_get(_url: str, timeout: float) -> requests.Response:
        _ = timeout
        raise requests.Timeout("always fail")

    monkeypatch.setattr(requests, "get", fake_get)
    def fake_sleep(_seconds: float) -> None:
        return None

    monkeypatch.setattr(http_client.time, "sleep", fake_sleep)

    try:
        get_with_retry(
            "https://example.com",
            HttpPolicy(timeout_seconds=1.0, retry_count=2, backoff_seconds=0.01),
        )
    except RuntimeError as exc:
        assert "HTTP request failed after retries" in str(exc)
    else:
        raise AssertionError("Expected RuntimeError when retries exhausted")


def test_get_with_retry_treats_5xx_as_retryable(monkeypatch: MonkeyPatch) -> None:
    class BadResponse:
        status_code = 503

    def fake_get(_url: str, timeout: float) -> BadResponse:
        _ = timeout
        return BadResponse()

    def fake_sleep(_seconds: float) -> None:
        return None

    monkeypatch.setattr(requests, "get", fake_get)
    monkeypatch.setattr(http_client.time, "sleep", fake_sleep)

    try:
        get_with_retry("https://example.com", HttpPolicy(retry_count=1))
    except RuntimeError as exc:
        assert "server error 503" in str(exc)
    else:
        raise AssertionError("Expected RuntimeError for 5xx response")
