package dev.agni.server

import dev.agni.core.config.Config
import dev.agni.core.store.Store
import io.github.oshai.kotlinlogging.KotlinLogging
import io.ktor.network.selector.SelectorManager
import io.ktor.network.sockets.InetSocketAddress
import io.ktor.network.sockets.ServerSocket
import io.ktor.network.sockets.aSocket
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.launch
import java.time.Instant

private val logger = KotlinLogging.logger {}

class Server private constructor(
    private val serverSocket: ServerSocket,
    private val store: Store,
    private val host: String,
    private val port: Int,
    private val startedAt: Instant,
) {
    val boundPort: Int
        get() = (serverSocket.localAddress as InetSocketAddress).port

    companion object {
        // Mirrors Cargo.toml's package version; env!("CARGO_PKG_VERSION") has no
        // direct Gradle equivalent without extra build wiring.
        private const val VERSION = "0.1.0"

        suspend fun create(config: Config): Server {
            val selectorManager = SelectorManager(Dispatchers.IO)
            val serverSocket = aSocket(selectorManager).tcp().bind(config.host, config.port)
            val startedAt = Instant.now()
            logger.warn {
                "server started service=agni host=${config.host} port=${config.port} version=$VERSION"
            }
            return Server(serverSocket, Store(), config.host, config.port, startedAt)
        }
    }

    suspend fun run(): Unit =
        coroutineScope {
            while (true) {
                val socket = serverSocket.accept()
                logger.info { "new connection peer=${socket.remoteAddress}" }
                launch { handleConnection(socket, store, host, port, startedAt) }
            }
        }
}
