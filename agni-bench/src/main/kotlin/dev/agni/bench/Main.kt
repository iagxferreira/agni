package dev.agni.bench

import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.core.main
import com.github.ajalt.clikt.parameters.options.default
import com.github.ajalt.clikt.parameters.options.option
import com.github.ajalt.clikt.parameters.types.int
import io.ktor.network.selector.SelectorManager
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.runBlocking
import kotlin.time.DurationUnit

class BenchCommand : CliktCommand(name = "agni-bench") {
    private val host: String by option("--host", help = "Server host").default("127.0.0.1")
    private val port: Int by option("--port", help = "Server port").int().default(6379)
    private val concurrency: Int by option("-c", "--concurrency", help = "Number of concurrent connections")
        .int().default(50)
    private val ops: Int by option("-n", "--ops", help = "Total number of operations per scenario")
        .int().default(10000)

    override fun run() =
        runBlocking {
            println("=== Agni Bench ===")
            println("  Target:       $host:$port")
            println("  Concurrency:  $concurrency connections")
            println("  Ops/scenario: $ops")

            val selectorManager = SelectorManager(Dispatchers.IO)

            // Warm up
            runScenario(selectorManager, host, port, concurrency, concurrency) { "PING" }

            val pingResult = runScenario(selectorManager, host, port, concurrency, ops) { "PING" }
            printResult("PING", pingResult)

            val setResult =
                runScenario(selectorManager, host, port, concurrency, ops) { i ->
                    "SET key:${i % 1000} value:$i"
                }
            printResult("SET (1000 unique keys)", setResult)

            val getHitResult =
                runScenario(selectorManager, host, port, concurrency, ops) { i -> "GET key:${i % 1000}" }
            printResult("GET (hit)", getHitResult)

            val getMissResult =
                runScenario(selectorManager, host, port, concurrency, ops) { i -> "GET missing:$i" }
            printResult("GET (miss)", getMissResult)

            println()
            println("=== Mixed SET+GET ===")
            val mixedResult =
                runScenario(selectorManager, host, port, concurrency, ops) { i ->
                    if (i % 2 == 0) "SET key:${i % 500} value:$i" else "GET key:${i % 500}"
                }
            println("  Ops:          ${mixedResult.totalOps}")
            println("  Total time:   ${"%.2f".format(mixedResult.elapsed.toDouble(DurationUnit.SECONDS))}s")
            println("  Throughput:   ${"%.0f".format(mixedResult.opsPerSec())} ops/sec")

            println()
            println("Done.")

            selectorManager.close()
        }
}

fun main(args: Array<String>) = BenchCommand().main(args)
