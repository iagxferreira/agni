plugins {
    id("org.gradle.toolchains.foojay-resolver-convention") version "1.0.0"
}

rootProject.name = "agni"

dependencyResolutionManagement {
    repositories {
        mavenCentral()
    }
}

include("agni-core", "agni-server", "agni-client", "agni-bench")
