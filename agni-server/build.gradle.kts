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

// The application plugin defaults the run task's working directory to this
// module's directory, but `--config config.example.yml` (like Makefile's
// run-server target, and Cargo's equivalent working-directory behavior)
// expects to resolve relative paths from the repo root.
tasks.named<JavaExec>("run") {
    workingDir = rootProject.projectDir
}
