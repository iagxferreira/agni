package dev.agni.bench

import io.ktor.network.selector.SelectorManager
import io.ktor.network.sockets.aSocket
import io.ktor.network.sockets.openReadChannel
import io.ktor.network.sockets.openWriteChannel
import io.ktor.utils.io.ByteReadChannel
import io.ktor.utils.io.ByteWriteChannel
import io.ktor.utils.io.readFully
import io.ktor.utils.io.readInt
import io.ktor.utils.io.writeFully
import io.ktor.utils.io.writeInt
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope
import kotlin.time.Duration
import kotlin.time.DurationUnit
import kotlin.time.TimeSource

private suspend fun sendRecv(
    readChannel: ByteReadChannel,
    writeChannel: ByteWriteChannel,
    cmd: String,
) {
    val bytes = cmd.toByteArray(Charsets.UTF_8)
    writeChannel.writeInt(bytes.size)
    writeChannel.writeFully(bytes)

    val length = readChannel.readInt()
    val response = ByteArray(length)
    readChannel.readFully(response)
}

suspend fun runScenario(
    selectorManager: SelectorManager,
    host: String,
    port: Int,
    concurrency: Int,
    ops: Int,
    buildCmd: (Int) -> String,
): BenchResult =
    coroutineScope {
        val opsPerTask = ops / concurrency
        val start = TimeSource.Monotonic.markNow()

        val tasks =
            (0 until concurrency).map {
                async(Dispatchers.IO) {
                    val socket = aSocket(selectorManager).tcp().connect(host, port)
                    val readChannel = socket.openReadChannel()
                    val writeChannel = socket.openWriteChannel(autoFlush = true)

                    val latencies = ArrayList<Duration>(opsPerTask)
                    for (i in 0 until opsPerTask) {
                        val opStart = TimeSource.Monotonic.markNow()
                        sendRecv(readChannel, writeChannel, buildCmd(i))
                        latencies.add(opStart.elapsedNow())
                    }

                    socket.close()
                    latencies
                }
            }

        val allLatencies = tasks.awaitAll().flatten()
        val elapsed = start.elapsedNow()

        BenchResult(allLatencies.size, elapsed, allLatencies)
    }

fun printResult(
    label: String,
    result: BenchResult,
) {
    println()
    println("=== $label ===")
    println("  Ops:          ${result.totalOps}")
    println("  Total time:   ${"%.2f".format(result.elapsed.toDouble(DurationUnit.SECONDS))}s")
    println("  Throughput:   ${"%.0f".format(result.opsPerSec())} ops/sec")
    println("  Latency p50:  ${result.percentile(50.0)}")
    println("  Latency p95:  ${result.percentile(95.0)}")
    println("  Latency p99:  ${result.percentile(99.0)}")
}
