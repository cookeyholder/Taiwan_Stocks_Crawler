from __future__ import annotations

from collections import Counter
from dataclasses import dataclass, field


@dataclass(slots=True)
class BatchQualityReport:
    success_count: int = 0
    failed_count: int = 0
    market_closed_count: int = 0
    failure_reasons: Counter[str] = field(default_factory=Counter)

    def record_success(self) -> None:
        self.success_count += 1

    def record_failure(self, reason: str) -> None:
        self.failed_count += 1
        self.failure_reasons[reason] += 1

    def record_market_closed(self) -> None:
        self.market_closed_count += 1

    @property
    def source_error_count(self) -> int:
        return self.failure_reasons["source_error"]

    def as_dict(self) -> dict[str, object]:
        return {
            "success_count": self.success_count,
            "failed_count": self.failed_count,
            "source_error_count": self.source_error_count,
            "market_closed_count": self.market_closed_count,
            "failure_reasons": dict(self.failure_reasons),
        }
