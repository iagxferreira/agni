package dev.agni.core.store

import kotlinx.serialization.Serializable
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import java.util.UUID

@Serializable
data class Entry(
    @Serializable(with = UuidSerializer::class) val id: UUID,
    val key: String,
    @Serializable(with = ByteArrayBase64Serializer::class) val value: ByteArray,
) {
    constructor(key: String, value: ByteArray) : this(UUID.randomUUID(), key, value)

    fun toJson(): String = Json.encodeToString(this)
}
