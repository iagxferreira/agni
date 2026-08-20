package dev.agni.server

import dev.agni.core.protocol.Command
import dev.agni.core.protocol.Response
import dev.agni.core.store.Store
import io.github.oshai.kotlinlogging.KotlinLogging
import io.ktor.network.sockets.Socket
import io.ktor.network.sockets.openReadChannel
import io.ktor.network.sockets.openWriteChannel
import io.ktor.utils.io.readFully
import io.ktor.utils.io.readInt
import io.ktor.utils.io.writeFully
import io.ktor.utils.io.writeInt
import java.io.EOFException
import java.time.Duration
import java.time.Instant

private val logger = KotlinLogging.logger {}

// Matches tokio_util's LengthDelimitedCodec default max_frame_length, so a
// corrupt/hostile length prefix can't force an unbounded allocation.
private const val MAX_FRAME_LENGTH = 8 * 1024 * 1024

suspend fun handleConnection(
    socket: Socket,
    store: Store,
    host: String,
    port: Int,
    startedAt: Instant,
) {
    socket.use {
        val readChannel = socket.openReadChannel()
        val writeChannel = socket.openWriteChannel(autoFlush = true)

        try {
            while (!readChannel.isClosedForRead) {
                // A clean disconnect between frames surfaces as an EOFException
                // right here, same as a truncated one mid-frame would. Only the
                // former should be silent, so it's handled inline rather than by
                // the catch below.
                val length =
                    try {
                        readChannel.readInt()
                    } catch (e: EOFException) {
                        break
                    }
                if (length < 0 || length > MAX_FRAME_LENGTH) {
                    error("frame too large: $length bytes")
                }

                val frame = ByteArray(length)
                readChannel.readFully(frame)

                val response = dispatch(Command.fromBytes(frame), store, host, port, startedAt)
                val bytes = response.toBytes()
                writeChannel.writeInt(bytes.size)
                writeChannel.writeFully(bytes)
            }
        } catch (e: Exception) {
            logger.error(e) { "connection error" }
        }
    }
}

private fun dispatch(
    command: Command,
    store: Store,
    host: String,
    port: Int,
    startedAt: Instant,
): Response =
    when (command) {
        is Command.Ping -> Response.Pong
        is Command.Healthcheck -> {
            val uptimeSecs = Duration.between(startedAt, Instant.now()).seconds
            logger.warn { "healthcheck ok service=agni host=$host port=$port uptime_secs=$uptimeSecs" }
            Response.Ok
        }
        is Command.Get -> store.get(command.key)?.let { Response.Value(it) } ?: Response.Null
        is Command.Set -> {
            store.set(command.key, command.value)
            Response.Ok
        }
        is Command.Unknown -> Response.Error("unknown command '${command.message}'")
    }
