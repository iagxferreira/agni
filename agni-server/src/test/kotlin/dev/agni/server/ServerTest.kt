package dev.agni.server

import dev.agni.core.config.Config
import io.ktor.network.selector.SelectorManager
import io.ktor.network.sockets.Socket
import io.ktor.network.sockets.aSocket
import io.ktor.network.sockets.openReadChannel
import io.ktor.network.sockets.openWriteChannel
import io.ktor.utils.io.ByteReadChannel
import io.ktor.utils.io.ByteWriteChannel
import io.ktor.utils.io.readFully
import io.ktor.utils.io.readInt
import io.ktor.utils.io.writeFully
import io.ktor.utils.io.writeInt
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import kotlin.time.Duration.Companion.seconds

class ServerTest {
    private suspend fun send(
        readChannel: ByteReadChannel,
        writeChannel: ByteWriteChannel,
        command: String,
    ): String {
        val bytes = command.toByteArray(Charsets.UTF_8)
        writeChannel.writeInt(bytes.size)
        writeChannel.writeFully(bytes)

        val length = readChannel.readInt()
        val response = ByteArray(length)
        readChannel.readFully(response)
        return response.toString(Charsets.UTF_8)
    }

    private suspend fun CoroutineScope.startServer(): Server {
        val server = Server.create(Config(host = "127.0.0.1", port = 0))
        launch(Dispatchers.IO) { server.run() }
        return server
    }

    private suspend fun connectTo(
        selectorManager: SelectorManager,
        port: Int,
    ): Socket = aSocket(selectorManager).tcp().connect("127.0.0.1", port)

    @Test
    fun `ping get and set round trip`() =
        runBlocking {
            val job = Job()
            val scope = CoroutineScope(coroutineContext + job)
            val server = scope.startServer()

            val selectorManager = SelectorManager(Dispatchers.IO)
            val client = connectTo(selectorManager, server.boundPort)
            val readChannel = client.openReadChannel()
            val writeChannel = client.openWriteChannel(autoFlush = true)

            withTimeout(5.seconds) {
                assertEquals("PONG", send(readChannel, writeChannel, "PING"))
                assertEquals("NULL", send(readChannel, writeChannel, "GET missing"))
                assertEquals("OK", send(readChannel, writeChannel, "SET foo bar baz"))
                assertEquals("bar baz", send(readChannel, writeChannel, "GET foo"))
                assertEquals(
                    "ERR unknown command 'NOPE'",
                    send(readChannel, writeChannel, "NOPE"),
                )
            }

            client.close()
            selectorManager.close()
            job.cancel()
        }

    @Test
    fun `server keeps accepting after a client disconnects cleanly`() =
        runBlocking {
            val job = Job()
            val scope = CoroutineScope(coroutineContext + job)
            val server = scope.startServer()
            val selectorManager = SelectorManager(Dispatchers.IO)

            withTimeout(5.seconds) {
                val first = connectTo(selectorManager, server.boundPort)
                assertEquals("PONG", send(first.openReadChannel(), first.openWriteChannel(autoFlush = true), "PING"))
                first.close()

                val second = connectTo(selectorManager, server.boundPort)
                assertEquals("PONG", send(second.openReadChannel(), second.openWriteChannel(autoFlush = true), "PING"))
                second.close()
            }

            selectorManager.close()
            job.cancel()
        }
}
