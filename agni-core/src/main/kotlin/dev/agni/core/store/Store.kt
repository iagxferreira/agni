package dev.agni.core.store

import java.util.concurrent.ConcurrentHashMap

class Store {
    private val data = ConcurrentHashMap<String, Entry>()

    fun set(key: String, value: ByteArray) {
        data[key] = Entry(key, value)
    }

    fun get(key: String): ByteArray? = data[key]?.value

    fun delete(key: String): Boolean = data.remove(key) != null

    // Unlike the Rust version, this doesn't return a Result: kotlinx.serialization
    // throws on failure rather than returning one, and Entry's shape can't fail to
    // serialize.
    fun getAsJson(key: String): String? = data[key]?.toJson()
}
