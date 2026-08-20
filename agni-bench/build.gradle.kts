// Benchmark binary.

plugins {
    alias(libs.plugins.kotlin.jvm)
    application
}

kotlin {
    jvmToolchain(21)
}

application {
    mainClass = "dev.agni.bench.MainKt"
}

dependencies {
    implementation(libs.ktor.network)
    implementation(libs.kotlinx.coroutines.core)
    implementation(libs.clikt)

    testImplementation(platform(libs.junit.bom))
    testImplementation(libs.junit.jupiter)
    testRuntimeOnly(libs.junit.platform.launcher)
}

tasks.test {
    useJUnitPlatform()
}
