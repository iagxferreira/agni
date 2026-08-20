// TCP server binary.

plugins {
    alias(libs.plugins.kotlin.jvm)
    application
}

kotlin {
    jvmToolchain(21)
}

application {
    mainClass = "dev.agni.server.MainKt"
}

dependencies {
    implementation(project(":agni-core"))
    implementation(libs.ktor.network)
    implementation(libs.kotlinx.coroutines.core)
    implementation(libs.clikt)
    implementation(libs.kotlin.logging)
    runtimeOnly(libs.logback.classic)

    testImplementation(platform(libs.junit.bom))
    testImplementation(libs.junit.jupiter)
    testRuntimeOnly(libs.junit.platform.launcher)
}

tasks.test {
    useJUnitPlatform()
}
