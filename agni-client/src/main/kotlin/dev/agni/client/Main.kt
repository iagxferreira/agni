package dev.agni.client

import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.core.Context
import com.github.ajalt.clikt.core.main
import com.github.ajalt.clikt.parameters.arguments.argument
import com.github.ajalt.clikt.parameters.arguments.multiple
import com.github.ajalt.clikt.parameters.options.default
import com.github.ajalt.clikt.parameters.options.option
import com.github.ajalt.clikt.parameters.types.int
import io.github.oshai.kotlinlogging.KotlinLogging
import io.ktor.network.selector.SelectorManager
import io.ktor.network.sockets.aSocket
import io.ktor.network.sockets.openReadChannel
import io.ktor.network.sockets.openWriteChannel
import io.ktor.utils.io.readFully
import io.ktor.utils.io.readInt
import io.ktor.utils.io.writeFully
import io.ktor.utils.io.writeInt
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.runBlocking
import java.io.EOFException
import kotlin.system.exitProcess

private val logger = KotlinLogging.logger {}

class ClientCommand : CliktCommand(name = "agni-client") {
    override fun help(context: Context): String = "CLI client for agni server"

    private val host: String by option("--host", help = "Server host").default("127.0.0.1")
    private val port: Int by option("--port", help = "Server port").int().default(6379)
    private val command: List<String> by argument(
        help = "Command and arguments (e.g. PING, GET key, SET key value)",
    ).multiple(required = true)

    override fun run() {
        val frame = command.joinToString(" ")

        runBlocking {
            val selectorManager = SelectorManager(Dispatchers.IO)
            val socket =
                try {
                    aSocket(selectorManager).tcp().connect(host, port)
                } catch (e: Exception) {
                    logger.error { "could not connect to $host:$port: ${e.message}" }
                    exitProcess(1)
                }

            val readChannel = socket.openReadChannel()
            val writeChannel = socket.openWriteChannel(autoFlush = true)
            val bytes = frame.toByteArray(Charsets.UTF_8)

            try {
                writeChannel.writeInt(bytes.size)
                writeChannel.writeFully(bytes)
            } catch (e: Exception) {
                logger.error { "failed to send command: ${e.message}" }
                exitProcess(1)
            }

            try {
                val length = readChannel.readInt()
                val response = ByteArray(length)
                readChannel.readFully(response)
                println(response.toString(Charsets.UTF_8))
            } catch (e: EOFException) {
                logger.error { "connection closed with no response" }
            } catch (e: Exception) {
                logger.error { e.message }
            }

            socket.close()
            selectorManager.close()
        }
    }
}

fun main(args: Array<String>) = ClientCommand().main(args)
