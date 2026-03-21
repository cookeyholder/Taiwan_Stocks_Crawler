from __future__ import annotations


def decode_with_fallback(content: bytes, encodings: list[str] | None = None) -> tuple[str, str]:
    encodings = encodings or ["cp950", "big5hkscs", "utf-8-sig", "utf-8"]
    for encoding in encodings:
        try:
            return content.decode(encoding), encoding
        except UnicodeDecodeError:
            continue
    return content.decode("utf-8", errors="replace"), "utf-8-replace"
