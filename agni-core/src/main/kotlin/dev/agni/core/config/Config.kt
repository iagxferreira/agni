package dev.agni.core.config

import com.fasterxml.jackson.core.JacksonException
import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.dataformat.yaml.YAMLFactory
import com.fasterxml.jackson.module.kotlin.readValue
import com.fasterxml.jackson.module.kotlin.registerKotlinModule
import java.io.IOException
import java.nio.file.Files
import java.nio.file.Path

data class Config(
    val host: String = "127.0.0.1",
    val port: Int = 6379,
) {
    fun addr(): String = "$host:$port"

    companion object {
        private val mapper = ObjectMapper(YAMLFactory()).registerKotlinModule()

        // Unlike the Rust version, which returns a Result, this throws
        // ConfigException: Kotlin has no checked exceptions, so a thrown
        // unchecked exception is the idiomatic equivalent here.
        fun fromFile(path: Path): Config {
            val contents =
                try {
                    Files.readString(path)
                } catch (e: IOException) {
                    throw ConfigException.Io(e)
                }
            return try {
                mapper.readValue<Config>(contents)
            } catch (e: JacksonException) {
                throw ConfigException.Parse(e)
            }
        }
    }
}

sealed class ConfigException(message: String, cause: Throwable) : Exception(message, cause) {
    class Io(cause: IOException) : ConfigException("could not read config file: ${cause.message}", cause)

    class Parse(cause: JacksonException) : ConfigException("invalid config: ${cause.message}", cause)
}
