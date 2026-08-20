package dev.agni.core.protocol

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test

class CommandTest {
    private fun parse(input: String): Command = Command.fromBytes(input.toByteArray())

    @Test
    fun `ping is case insensitive`() {
        assertEquals(Command.Ping, parse("ping"))
        assertEquals(Command.Ping, parse("PING"))
    }

    @Test
    fun `healthcheck parses`() {
        assertEquals(Command.Healthcheck, parse("HEALTHCHECK"))
    }

    @Test
    fun `get parses key`() {
        assertEquals(Command.Get("foo"), parse("GET foo"))
    }

    @Test
    fun `get without key is unknown`() {
        val result = parse("GET")
        assertTrue(result is Command.Unknown)
        assertEquals("GET requires a key", (result as Command.Unknown).message)
    }

    @Test
    fun `get ignores trailing words`() {
        assertEquals(Command.Get("foo"), parse("GET foo bar baz"))
    }

    @Test
    fun `set parses key and value`() {
        val result = parse("SET foo bar")
        assertTrue(result is Command.Set)
        result as Command.Set
        assertEquals("foo", result.key)
        assertEquals("bar", result.value.toString(Charsets.UTF_8))
    }

    @Test
    fun `set value preserves internal spaces`() {
        val result = parse("SET foo bar baz")
        assertTrue(result is Command.Set)
        result as Command.Set
        assertEquals("foo", result.key)
        assertEquals("bar baz", result.value.toString(Charsets.UTF_8))
    }

    @Test
    fun `set without value is unknown`() {
        val result = parse("SET foo")
        assertTrue(result is Command.Unknown)
        assertEquals("SET requires a key and value", (result as Command.Unknown).message)
    }

    @Test
    fun `unrecognized command is unknown`() {
        assertEquals(Command.Unknown("FOO"), parse("foo"))
    }
}
