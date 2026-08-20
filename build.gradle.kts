plugins {
    alias(libs.plugins.kotlin.jvm) apply false
}

subprojects {
    group = "dev.agni"
    version = "0.1.0"

    repositories {
        mavenCentral()
    }
}
