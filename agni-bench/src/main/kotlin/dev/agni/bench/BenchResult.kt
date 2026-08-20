package dev.agni.bench

import kotlin.time.Duration
import kotlin.time.DurationUnit

class BenchResult(
    val totalOps: Int,
    val elapsed: Duration,
    val latencies: List<Duration>,
) {
    fun opsPerSec(): Double = totalOps / elapsed.toDouble(DurationUnit.SECONDS)

    fun percentile(p: Double): Duration {
        val sorted = latencies.sorted()
        val idx = (sorted.size * p / 100.0).toInt().coerceAtMost(sorted.size - 1)
        return sorted[idx]
    }
}
