package dev.agni.core.protocol

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test

class ResponseTest {
    private fun bytesOf(response: Response): String = response.toBytes().toString(Charsets.UTF_8)

    @Test
    fun `pong encodes to PONG`() {
        assertEquals("PONG", bytesOf(Response.Pong))
    }

    @Test
    fun `ok encodes to OK`() {
        assertEquals("OK", bytesOf(Response.Ok))
    }

    @Test
    fun `value encodes to raw bytes`() {
        assertEquals("hello", bytesOf(Response.Value("hello".toByteArray())))
    }

    @Test
    fun `null encodes to NULL`() {
        assertEquals("NULL", bytesOf(Response.Null))
    }

    @Test
    fun `error encodes with ERR prefix`() {
        assertEquals("ERR bad command", bytesOf(Response.Error("bad command")))
    }
}
