from __future__ import annotations

import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "tools" / "benchmark"))

from frame_stats import (  # noqa: E402
    FrameStats,
    compute_frame_stats,
    format_frame_stats,
    percentile,
)


class FrameStatsTests(unittest.TestCase):
    def test_empty_samples_return_zeroed_stats(self) -> None:
        stats = compute_frame_stats([])
        self.assertEqual(stats.avg_fps, 0.0)
        self.assertEqual(stats.avg_frame_time_ms, 0.0)
        self.assertEqual(stats.stuttering_frames, 0)
        self.assertEqual(stats.stutter_ratio, 0.0)

    def test_constant_frame_time_yields_expected_fps(self) -> None:
        stats = compute_frame_stats([16.6] * 60)
        self.assertEqual(len(stats.samples_ms), 60)
        self.assertAlmostEqual(stats.avg_frame_time_ms, 16.6, places=3)
        self.assertAlmostEqual(stats.avg_fps, 1000.0 / 16.6, places=2)

    def test_stutter_ratio_counts_over_threshold_frames(self) -> None:
        samples = [16.0, 16.0, 90.0, 16.0]
        stats = compute_frame_stats(samples, stutter_threshold_ms=50.0)
        self.assertEqual(stats.stuttering_frames, 1)
        self.assertAlmostEqual(stats.stutter_ratio, 0.25)

    def test_lowest_and_highest_frame_times(self) -> None:
        stats = compute_frame_stats([10.0, 20.0, 30.0])
        self.assertEqual(stats.min_frame_time_ms, 10.0)
        self.assertEqual(stats.max_frame_time_ms, 30.0)

    def test_percentile_interpolates(self) -> None:
        self.assertEqual(percentile([10.0, 20.0, 30.0], 50.0), 20.0)
        self.assertEqual(percentile([10.0, 20.0, 30.0], 90.0), 28.0)
        self.assertEqual(percentile([], 90.0), 0.0)

    def test_format_frame_stats_renders_line(self) -> None:
        stats = compute_frame_stats([16.0, 16.0])
        text = format_frame_stats(stats)
        self.assertIn("avg_fps=", text)
        self.assertIn("stutter_ratio=", text)


if __name__ == "__main__":
    unittest.main()
