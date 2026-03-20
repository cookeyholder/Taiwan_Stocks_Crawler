from __future__ import annotations

from collections import Counter
from dataclasses import dataclass, field


@dataclass(slots=True)
class BatchQualityReport:
    success_count: int = 0
    failed_count: int = 0
    failure_reasons: Counter[str] = field(default_factory=Counter)

    def record_success(self) -> None:
        self.success_count += 1

    def record_failure(self, reason: str) -> None:
        self.failed_count += 1
        self.failure_reasons[reason] += 1

    def as_dict(self) -> dict[str, object]:
        return {
            "success_count": self.success_count,
            "failed_count": self.failed_count,
            "failure_reasons": dict(self.failure_reasons),
        }
