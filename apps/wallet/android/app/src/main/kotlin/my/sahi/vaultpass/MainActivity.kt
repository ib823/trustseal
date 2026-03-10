package my.sahi.vaultpass

import io.flutter.embedding.android.FlutterFragmentActivity
import io.flutter.embedding.engine.FlutterEngine

/**
 * Main activity for VaultPass wallet.
 *
 * Uses FlutterFragmentActivity to support BiometricPrompt.
 */
class MainActivity : FlutterFragmentActivity() {
    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)

        // Register keystore plugin
        flutterEngine.plugins.add(KeystorePlugin())
    }
}
