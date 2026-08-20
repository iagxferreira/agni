package dev.agni.core.store

import kotlinx.serialization.json.Json
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test

class StoreTest {
    @Test
    fun `set and get`() {
        val store = Store()
        store.set("name", "agni".toByteArray())
        assertEquals("agni", store.get("name")?.toString(Charsets.UTF_8))
    }

    @Test
    fun `get missing key returns null`() {
        val store = Store()
        assertNull(store.get("missing"))
    }

    @Test
    fun `overwrite value`() {
        val store = Store()
        store.set("key", "first".toByteArray())
        store.set("key", "second".toByteArray())
        assertEquals("second", store.get("key")?.toString(Charsets.UTF_8))
    }

    @Test
    fun `delete existing key`() {
        val store = Store()
        store.set("key", "value".toByteArray())
        assertTrue(store.delete("key"))
        assertNull(store.get("key"))
    }

    @Test
    fun `delete missing key`() {
        val store = Store()
        assertFalse(store.delete("missing"))
    }

    @Test
    fun `shared across references`() {
        val store = Store()
        val storeRef = store
        store.set("key", "value".toByteArray())
        assertEquals("value", storeRef.get("key")?.toString(Charsets.UTF_8))
    }

    @Test
    fun `get as json contains key and base64 value`() {
        val store = Store()
        store.set("hello", "world".toByteArray())
        val json = store.getAsJson("hello")
        checkNotNull(json)
        val parsed = Json.parseToJsonElement(json).jsonObject
        assertEquals("hello", parsed["key"]?.jsonPrimitive?.content)
        assertEquals("d29ybGQ=", parsed["value"]?.jsonPrimitive?.content)
        assertTrue(parsed["id"]?.jsonPrimitive?.isString == true)
    }

    @Test
    fun `get as json missing key`() {
        val store = Store()
        assertNull(store.getAsJson("missing"))
    }
}
