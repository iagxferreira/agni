package dev.agni.core.protocol

sealed class Response {
    object Pong : Response()
    object Ok : Response()
    data class Value(val value: ByteArray) : Response()
    object Null : Response()
    data class Error(val message: String) : Response()

    fun toBytes(): ByteArray = when (this) {
        is Pong -> "PONG".toByteArray(Charsets.UTF_8)
        is Ok -> "OK".toByteArray(Charsets.UTF_8)
        is Value -> value
        is Null -> "NULL".toByteArray(Charsets.UTF_8)
        is Error -> "ERR $message".toByteArray(Charsets.UTF_8)
    }
}
