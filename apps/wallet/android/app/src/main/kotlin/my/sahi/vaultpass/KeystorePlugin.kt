package my.sahi.vaultpass

import android.content.Context
import android.os.Build
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.security.keystore.StrongBoxUnavailableException
import androidx.biometric.BiometricPrompt
import androidx.core.content.ContextCompat
import androidx.fragment.app.FragmentActivity
import io.flutter.embedding.engine.plugins.FlutterPlugin
import io.flutter.embedding.engine.plugins.activity.ActivityAware
import io.flutter.embedding.engine.plugins.activity.ActivityPluginBinding
import io.flutter.plugin.common.MethodCall
import io.flutter.plugin.common.MethodChannel
import io.flutter.plugin.common.MethodChannel.MethodCallHandler
import io.flutter.plugin.common.MethodChannel.Result
import java.security.KeyPairGenerator
import java.security.KeyStore
import java.security.PrivateKey
import java.security.Signature
import java.security.spec.ECGenParameterSpec

/**
 * Flutter plugin for Android Keystore operations.
 *
 * Provides hardware-bound key storage using Android Keystore.
 * Prefers StrongBox when available for maximum security.
 *
 * Security features:
 * - Hardware-backed keys (TEE or StrongBox)
 * - Biometric authentication required for signing
 * - Keys are non-exportable
 * - Keys are bound to this device
 */
class KeystorePlugin : FlutterPlugin, MethodCallHandler, ActivityAware {
    private lateinit var channel: MethodChannel
    private lateinit var context: Context
    private var activity: FragmentActivity? = null

    private val keyStore: KeyStore by lazy {
        KeyStore.getInstance("AndroidKeyStore").apply { load(null) }
    }

    override fun onAttachedToEngine(binding: FlutterPlugin.FlutterPluginBinding) {
        channel = MethodChannel(binding.binaryMessenger, "my.sahi.vaultpass/keystore")
        channel.setMethodCallHandler(this)
        context = binding.applicationContext
    }

    override fun onDetachedFromEngine(binding: FlutterPlugin.FlutterPluginBinding) {
        channel.setMethodCallHandler(null)
    }

    override fun onAttachedToActivity(binding: ActivityPluginBinding) {
        activity = binding.activity as? FragmentActivity
    }

    override fun onDetachedFromActivityForConfigChanges() {
        activity = null
    }

    override fun onReattachedToActivityForConfigChanges(binding: ActivityPluginBinding) {
        activity = binding.activity as? FragmentActivity
    }

    override fun onDetachedFromActivity() {
        activity = null
    }

    override fun onMethodCall(call: MethodCall, result: Result) {
        when (call.method) {
            "isHardwareBackedAvailable" -> {
                result.success(isHardwareBackedAvailable())
            }
            "isStrongBoxAvailable" -> {
                result.success(isStrongBoxAvailable())
            }
            "generateKeyPair" -> {
                val alias = call.argument<String>("alias") ?: return result.error(
                    "INVALID_ARGUMENT", "alias is required", null
                )
                val requireBiometric = call.argument<Boolean>("requireBiometric") ?: true
                generateKeyPair(alias, requireBiometric, result)
            }
            "hasKey" -> {
                val alias = call.argument<String>("alias") ?: return result.error(
                    "INVALID_ARGUMENT", "alias is required", null
                )
                result.success(keyStore.containsAlias(alias))
            }
            "getPublicKey" -> {
                val alias = call.argument<String>("alias") ?: return result.error(
                    "INVALID_ARGUMENT", "alias is required", null
                )
                getPublicKey(alias, result)
            }
            "sign" -> {
                val alias = call.argument<String>("alias") ?: return result.error(
                    "INVALID_ARGUMENT", "alias is required", null
                )
                val data = call.argument<ByteArray>("data") ?: return result.error(
                    "INVALID_ARGUMENT", "data is required", null
                )
                sign(alias, data, result)
            }
            "deleteKey" -> {
                val alias = call.argument<String>("alias") ?: return result.error(
                    "INVALID_ARGUMENT", "alias is required", null
                )
                deleteKey(alias, result)
            }
            else -> result.notImplemented()
        }
    }

    private fun isHardwareBackedAvailable(): Boolean {
        // All modern Android devices (API 23+) have hardware-backed keystore
        return Build.VERSION.SDK_INT >= Build.VERSION_CODES.M
    }

    private fun isStrongBoxAvailable(): Boolean {
        // StrongBox requires API 28+ and hardware support
        return Build.VERSION.SDK_INT >= Build.VERSION_CODES.P &&
                context.packageManager.hasSystemFeature("android.hardware.strongbox_keystore")
    }

    private fun generateKeyPair(alias: String, requireBiometric: Boolean, result: Result) {
        try {
            // Delete existing key if present
            if (keyStore.containsAlias(alias)) {
                keyStore.deleteEntry(alias)
            }

            val keyPairGenerator = KeyPairGenerator.getInstance(
                KeyProperties.KEY_ALGORITHM_EC,
                "AndroidKeyStore"
            )

            val builder = KeyGenParameterSpec.Builder(
                alias,
                KeyProperties.PURPOSE_SIGN or KeyProperties.PURPOSE_VERIFY
            )
                .setAlgorithmParameterSpec(ECGenParameterSpec("secp256r1"))
                .setDigests(KeyProperties.DIGEST_SHA256)
                .setUserAuthenticationRequired(requireBiometric)

            // Try StrongBox first, fall back to TEE
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P && isStrongBoxAvailable()) {
                try {
                    builder.setIsStrongBoxBacked(true)
                } catch (e: StrongBoxUnavailableException) {
                    // Fall back to TEE
                }
            }

            // Require biometric for every use (not just unlock)
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R && requireBiometric) {
                builder.setUserAuthenticationParameters(
                    0, // Require auth for every use
                    KeyProperties.AUTH_BIOMETRIC_STRONG
                )
            } else if (requireBiometric) {
                @Suppress("DEPRECATION")
                builder.setUserAuthenticationValidityDurationSeconds(-1)
            }

            keyPairGenerator.initialize(builder.build())
            val keyPair = keyPairGenerator.generateKeyPair()

            // Return the public key bytes
            result.success(keyPair.public.encoded)

        } catch (e: Exception) {
            result.error("KEY_GENERATION_FAILED", e.message, null)
        }
    }

    private fun getPublicKey(alias: String, result: Result) {
        try {
            val entry = keyStore.getCertificate(alias)
            if (entry == null) {
                result.error("KEY_NOT_FOUND", "Key not found: $alias", null)
                return
            }
            result.success(entry.publicKey.encoded)
        } catch (e: Exception) {
            result.error("GET_PUBLIC_KEY_FAILED", e.message, null)
        }
    }

    private fun sign(alias: String, data: ByteArray, result: Result) {
        val fragmentActivity = activity
        if (fragmentActivity == null) {
            result.error("NO_ACTIVITY", "Activity not available for biometric prompt", null)
            return
        }

        try {
            val privateKey = keyStore.getKey(alias, null) as? PrivateKey
            if (privateKey == null) {
                result.error("KEY_NOT_FOUND", "Private key not found: $alias", null)
                return
            }

            val signature = Signature.getInstance("SHA256withECDSA")
            signature.initSign(privateKey)

            // Create biometric prompt
            val executor = ContextCompat.getMainExecutor(context)
            val biometricPrompt = BiometricPrompt(
                fragmentActivity,
                executor,
                object : BiometricPrompt.AuthenticationCallback() {
                    override fun onAuthenticationSucceeded(authResult: BiometricPrompt.AuthenticationResult) {
                        try {
                            // Sign the data after successful authentication
                            val cryptoObject = authResult.cryptoObject
                            val sig = cryptoObject?.signature ?: signature
                            sig.update(data)
                            val signatureBytes = sig.sign()
                            result.success(signatureBytes)
                        } catch (e: Exception) {
                            result.error("SIGNING_FAILED", e.message, null)
                        }
                    }

                    override fun onAuthenticationError(errorCode: Int, errString: CharSequence) {
                        when (errorCode) {
                            BiometricPrompt.ERROR_USER_CANCELED,
                            BiometricPrompt.ERROR_NEGATIVE_BUTTON -> {
                                result.error("USER_CANCELED", errString.toString(), null)
                            }
                            else -> {
                                result.error("BIOMETRIC_FAILED", errString.toString(), null)
                            }
                        }
                    }

                    override fun onAuthenticationFailed() {
                        // Don't report error - user can retry
                    }
                }
            )

            val promptInfo = BiometricPrompt.PromptInfo.Builder()
                .setTitle("Authenticate")
                .setSubtitle("Confirm to present credential")
                .setNegativeButtonText("Cancel")
                .setAllowedAuthenticators(BiometricPrompt.Authenticators.BIOMETRIC_STRONG)
                .build()

            biometricPrompt.authenticate(
                promptInfo,
                BiometricPrompt.CryptoObject(signature)
            )

        } catch (e: Exception) {
            result.error("SIGNING_FAILED", e.message, null)
        }
    }

    private fun deleteKey(alias: String, result: Result) {
        try {
            if (keyStore.containsAlias(alias)) {
                keyStore.deleteEntry(alias)
            }
            result.success(null)
        } catch (e: Exception) {
            result.error("DELETE_FAILED", e.message, null)
        }
    }
}
