plugins {
    id("com.android.application")
    id("kotlin-android")
    id("dev.flutter.flutter-gradle-plugin")
}

android {
    namespace = "my.sahi.vaultpass"
    compileSdk = 34

    defaultConfig {
        applicationId = "my.sahi.vaultpass"
        minSdk = 26 // Required for Keystore with biometrics
        targetSdk = 34
        versionCode = 1
        versionName = "1.0.0"
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = "17"
    }

    buildTypes {
        release {
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
            signingConfig = signingConfigs.getByName("debug") // TODO: Use release signing
        }
    }
}

flutter {
    source = "../.."
}

dependencies {
    // Biometric authentication
    implementation("androidx.biometric:biometric:1.2.0-alpha05")

    // Fragment activity for biometric prompt
    implementation("androidx.fragment:fragment-ktx:1.6.2")
}
