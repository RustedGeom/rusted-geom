#!/usr/bin/env python3
"""Fail CI if key benchmark estimates regress beyond fixed thresholds."""

from __future__ import annotations

import json
from pathlib import Path
import sys


# Criterion estimates are stored in nanoseconds.
THRESHOLDS_NS = {
    "curve_bounds_fast": 120.0,
    "curve_point_and_d1_at_polyline_1k": 700.0,
    "curve_fit_nurbs_from_200_points": 500_000.0,
}


def estimate_path(repo_root: Path, bench_name: str) -> Path:
    return repo_root / "target" / "criterion" / bench_name / "new" / "estimates.json"


def read_mean_ns(path: Path) -> float:
    data = json.loads(path.read_text())
    return float(data["mean"]["point_estimate"])


def main() -> int:
    repo_root = Path(__file__).resolve().parents[1]
    failures: list[str] = []

    for bench_name, threshold in THRESHOLDS_NS.items():
        path = estimate_path(repo_root, bench_name)
        if not path.exists():
            failures.append(f"{bench_name}: missing estimates file at {path}")
            continue
        value = read_mean_ns(path)
        if value > threshold:
            failures.append(
                f"{bench_name}: {value:.3f}ns exceeds threshold {threshold:.3f}ns"
            )

    if failures:
        print("Benchmark threshold check failed:")
        for msg in failures:
            print(f" - {msg}")
        return 1

    print("Benchmark threshold check passed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
