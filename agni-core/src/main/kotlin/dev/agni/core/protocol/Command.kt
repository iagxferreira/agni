package dev.agni.core.protocol

sealed class Command {
    object Ping : Command()
    object Healthcheck : Command()
    data class Get(val key: String) : Command()
    data class Set(val key: String, val value: ByteArray) : Command()
    data class Unknown(val message: String) : Command()

    companion object {
        // Mirrors the Rust parser: split on the first two spaces only, so a
        // SET value can contain spaces of its own.
        fun fromBytes(bytes: ByteArray): Command {
            val input = String(bytes, Charsets.UTF_8)
            val parts = input.split(' ', limit = 3)
            val command = parts.getOrElse(0) { "" }.uppercase()

            return when (command) {
                "PING" -> Ping
                "HEALTHCHECK" -> Healthcheck
                "GET" -> {
                    val key = parts.getOrNull(1)
                    if (key != null) Get(key.trim()) else Unknown("GET requires a key")
                }
                "SET" -> {
                    val key = parts.getOrNull(1)
                    val value = parts.getOrNull(2)
                    if (key != null && value != null) {
                        Set(key.trim(), value.trim().toByteArray(Charsets.UTF_8))
                    } else {
                        Unknown("SET requires a key and value")
                    }
                }
                else -> Unknown(command)
            }
        }
    }
}
