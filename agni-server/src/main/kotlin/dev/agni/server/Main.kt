package dev.agni.server

import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.core.Context
import com.github.ajalt.clikt.core.main
import com.github.ajalt.clikt.parameters.options.option
import dev.agni.core.config.Config
import dev.agni.core.config.ConfigException
import io.github.oshai.kotlinlogging.KotlinLogging
import kotlinx.coroutines.runBlocking
import java.nio.file.Path
import kotlin.system.exitProcess

private val logger = KotlinLogging.logger {}

class ServerCommand : CliktCommand(name = "agni-server") {
    override fun help(context: Context): String = "A Redis-like in-memory cache server"

    private val configPath: String? by option("--config", "-c", help = "Path to the YAML configuration file")

    override fun run() {
        val config =
            configPath?.let { path ->
                try {
                    Config.fromFile(Path.of(path))
                } catch (e: ConfigException) {
                    logger.error { e.message }
                    exitProcess(1)
                }
            } ?: Config()

        runBlocking {
            val server = Server.create(config)
            server.run()
        }
    }
}

fun main(args: Array<String>) = ServerCommand().main(args)
