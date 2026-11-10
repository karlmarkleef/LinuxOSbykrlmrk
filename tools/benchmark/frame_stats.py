"""Frame time and FPS statistics for the Mark Linux game benchmark suite.

Pure Python helpers used by the Mark Compatibility Platform gaming test
suite to summarize frame-time samples collected while a Windows game is
running through the Wine/Proton + DXVK/VKD3R stack.

The helpers intentionally avoid any hardware access or process launching so
they can be unit tested with synthetic frame-time samples. Real game runs are
started through `mark-win-launch` and their samples aggregated here.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Sequence

STUTTER_THRESHOLD_MS = 50.0


@dataclass
class FrameStats:
    """Aggregated frame-time statistics for one benchmark session."""

    samples_ms: list[float]
    avg_frame_time_ms: float
    avg_fps: float
    min_frame_time_ms: float
    max_frame_time_ms: float
    stutter_ratio: float
    stuttering_frames: int


def percentile(values: Sequence[float], pct: float) -> float:
    """Linearly interpolated percentile of a sample sequence."""
    if not values:
        return 0.0
    ordered = sorted(float(v) for v in values)
    if len(ordered) == 1:
        return ordered[0]
    position = (len(ordered) - 1) * (pct / 100.0)
    lower = int(position)
    upper = min(lower + 1, len(ordered) - 1)
    fraction = position - lower
    return ordered[lower] + (ordered[upper] - ordered[lower]) * fraction


def compute_frame_stats(
    samples_ms: Sequence[float],
    stutter_threshold_ms: float = STUTTER_THRESHOLD_MS,
) -> FrameStats:
    """Compute FPS, frame time and stutter statistics from frame samples."""
    frames = [float(s) for s in samples_ms]
    count = len(frames)
    if count == 0:
        return FrameStats(
            samples_ms=[],
            avg_frame_time_ms=0.0,
            avg_fps=0.0,
            min_frame_time_ms=0.0,
            max_frame_time_ms=0.0,
            stutter_ratio=0.0,
            stuttering_frames=0,
        )
    avg_frame_time_ms = sum(frames) / count
    avg_fps = 1000.0 / avg_frame_time_ms if avg_frame_time_ms > 0 else 0.0
    stuttering_frames = sum(1 for frame in frames if frame > stutter_threshold_ms)
    return FrameStats(
        samples_ms=frames,
        avg_frame_time_ms=avg_frame_time_ms,
        avg_fps=avg_fps,
        min_frame_time_ms=min(frames),
        max_frame_time_ms=max(frames),
        stutter_ratio=stuttering_frames / count,
        stuttering_frames=stuttering_frames,
    )


def format_frame_stats(stats: FrameStats) -> str:
    """Render frame statistics as a human-readable benchmark line."""
    return (
        f"frames={len(stats.samples_ms)} "
        f"avg_frame_ms={stats.avg_frame_time_ms:.3f} "
        f"avg_fps={stats.avg_fps:.2f} "
        f"min_ms={stats.min_frame_time_ms:.3f} "
        f"max_ms={stats.max_frame_time_ms:.3f} "
        f"stutter_ratio={stats.stutter_ratio:.3f}"
    )
