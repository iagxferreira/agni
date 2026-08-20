package dev.agni.core.config

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertThrows
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import java.nio.file.Files
import java.nio.file.Path

class ConfigTest {
    @Test
    fun `default host and port`() {
        val config = Config()
        assertEquals("127.0.0.1", config.host)
        assertEquals(6379, config.port)
    }

    @Test
    fun `addr formats host and port`() {
        val config = Config(host = "0.0.0.0", port = 7000)
        assertEquals("0.0.0.0:7000", config.addr())
    }

    @Test
    fun `from file parses valid yaml`(
        @TempDir tempDir: Path,
    ) {
        val file = tempDir.resolve("config.yml")
        Files.writeString(file, "host: 0.0.0.0\nport: 7000\n")

        val config = Config.fromFile(file)

        assertEquals("0.0.0.0", config.host)
        assertEquals(7000, config.port)
    }

    @Test
    fun `from file throws Io when missing`(
        @TempDir tempDir: Path,
    ) {
        val missing = tempDir.resolve("missing.yml")
        assertThrows(ConfigException.Io::class.java) { Config.fromFile(missing) }
    }

    @Test
    fun `from file throws Parse on invalid yaml`(
        @TempDir tempDir: Path,
    ) {
        val file = tempDir.resolve("config.yml")
        Files.writeString(file, "not: [valid")

        assertThrows(ConfigException.Parse::class.java) { Config.fromFile(file) }
    }
}
