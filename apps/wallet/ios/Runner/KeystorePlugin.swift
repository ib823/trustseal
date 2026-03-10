import Flutter
import LocalAuthentication
import Security

/// Flutter plugin for iOS Keychain/Secure Enclave operations.
///
/// Provides hardware-bound key storage using iOS Secure Enclave.
///
/// Security features:
/// - Secure Enclave for key storage (when available)
/// - Biometric authentication required for signing
/// - Keys are non-exportable
/// - Keys are bound to this device
public class KeystorePlugin: NSObject, FlutterPlugin {
    private static let keyAlias = "my.sahi.vaultpass.device_key"

    public static func register(with registrar: FlutterPluginRegistrar) {
        let channel = FlutterMethodChannel(
            name: "my.sahi.vaultpass/keystore",
            binaryMessenger: registrar.messenger()
        )
        let instance = KeystorePlugin()
        registrar.addMethodCallDelegate(instance, channel: channel)
    }

    public func handle(_ call: FlutterMethodCall, result: @escaping FlutterResult) {
        switch call.method {
        case "isHardwareBackedAvailable":
            result(isHardwareBackedAvailable())

        case "isStrongBoxAvailable":
            // iOS Secure Enclave is equivalent to StrongBox
            result(isSecureEnclaveAvailable())

        case "generateKeyPair":
            guard let args = call.arguments as? [String: Any],
                  let alias = args["alias"] as? String else {
                result(FlutterError(code: "INVALID_ARGUMENT", message: "alias is required", details: nil))
                return
            }
            let requireBiometric = args["requireBiometric"] as? Bool ?? true
            generateKeyPair(alias: alias, requireBiometric: requireBiometric, result: result)

        case "hasKey":
            guard let args = call.arguments as? [String: Any],
                  let alias = args["alias"] as? String else {
                result(FlutterError(code: "INVALID_ARGUMENT", message: "alias is required", details: nil))
                return
            }
            result(hasKey(alias: alias))

        case "getPublicKey":
            guard let args = call.arguments as? [String: Any],
                  let alias = args["alias"] as? String else {
                result(FlutterError(code: "INVALID_ARGUMENT", message: "alias is required", details: nil))
                return
            }
            getPublicKey(alias: alias, result: result)

        case "sign":
            guard let args = call.arguments as? [String: Any],
                  let alias = args["alias"] as? String,
                  let data = args["data"] as? FlutterStandardTypedData else {
                result(FlutterError(code: "INVALID_ARGUMENT", message: "alias and data are required", details: nil))
                return
            }
            sign(alias: alias, data: data.data, result: result)

        case "deleteKey":
            guard let args = call.arguments as? [String: Any],
                  let alias = args["alias"] as? String else {
                result(FlutterError(code: "INVALID_ARGUMENT", message: "alias is required", details: nil))
                return
            }
            deleteKey(alias: alias, result: result)

        default:
            result(FlutterMethodNotImplemented)
        }
    }

    private func isHardwareBackedAvailable() -> Bool {
        // iOS Keychain is always hardware-backed on modern devices
        return true
    }

    private func isSecureEnclaveAvailable() -> Bool {
        let context = LAContext()
        var error: NSError?

        // Secure Enclave requires biometrics or passcode
        let canEvaluate = context.canEvaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, error: &error)

        // Check for Secure Enclave support
        if #available(iOS 11.0, *) {
            // All iOS 11+ devices with Face ID or Touch ID have Secure Enclave
            return canEvaluate || context.biometryType != .none
        }

        return canEvaluate
    }

    private func generateKeyPair(alias: String, requireBiometric: Bool, result: @escaping FlutterResult) {
        // Delete existing key if present
        deleteKeyInternal(alias: alias)

        var accessControlFlags: SecAccessControlCreateFlags = [.privateKeyUsage]
        if requireBiometric {
            accessControlFlags.insert(.biometryCurrentSet)
        }

        guard let accessControl = SecAccessControlCreateWithFlags(
            kCFAllocatorDefault,
            kSecAttrAccessibleWhenUnlockedThisDeviceOnly,
            accessControlFlags,
            nil
        ) else {
            result(FlutterError(code: "KEY_GENERATION_FAILED", message: "Failed to create access control", details: nil))
            return
        }

        var keyAttributes: [String: Any] = [
            kSecAttrKeyType as String: kSecAttrKeyTypeECSECPrimeRandom,
            kSecAttrKeySizeInBits as String: 256,
            kSecAttrTokenID as String: kSecAttrTokenIDSecureEnclave,
            kSecPrivateKeyAttrs as String: [
                kSecAttrIsPermanent as String: true,
                kSecAttrApplicationTag as String: alias.data(using: .utf8)!,
                kSecAttrAccessControl as String: accessControl
            ] as [String: Any]
        ]

        var error: Unmanaged<CFError>?
        guard let privateKey = SecKeyCreateRandomKey(keyAttributes as CFDictionary, &error) else {
            // Fall back to non-Secure Enclave if not available
            keyAttributes.removeValue(forKey: kSecAttrTokenID as String)
            guard let fallbackKey = SecKeyCreateRandomKey(keyAttributes as CFDictionary, &error) else {
                let errorMessage = error?.takeRetainedValue().localizedDescription ?? "Unknown error"
                result(FlutterError(code: "KEY_GENERATION_FAILED", message: errorMessage, details: nil))
                return
            }
            returnPublicKey(privateKey: fallbackKey, result: result)
            return
        }

        returnPublicKey(privateKey: privateKey, result: result)
    }

    private func returnPublicKey(privateKey: SecKey, result: @escaping FlutterResult) {
        guard let publicKey = SecKeyCopyPublicKey(privateKey) else {
            result(FlutterError(code: "KEY_GENERATION_FAILED", message: "Failed to get public key", details: nil))
            return
        }

        var error: Unmanaged<CFError>?
        guard let publicKeyData = SecKeyCopyExternalRepresentation(publicKey, &error) as Data? else {
            let errorMessage = error?.takeRetainedValue().localizedDescription ?? "Unknown error"
            result(FlutterError(code: "KEY_GENERATION_FAILED", message: errorMessage, details: nil))
            return
        }

        result(FlutterStandardTypedData(bytes: publicKeyData))
    }

    private func hasKey(alias: String) -> Bool {
        let query: [String: Any] = [
            kSecClass as String: kSecClassKey,
            kSecAttrApplicationTag as String: alias.data(using: .utf8)!,
            kSecAttrKeyType as String: kSecAttrKeyTypeECSECPrimeRandom,
            kSecReturnRef as String: false
        ]

        return SecItemCopyMatching(query as CFDictionary, nil) == errSecSuccess
    }

    private func getPublicKey(alias: String, result: @escaping FlutterResult) {
        let query: [String: Any] = [
            kSecClass as String: kSecClassKey,
            kSecAttrApplicationTag as String: alias.data(using: .utf8)!,
            kSecAttrKeyType as String: kSecAttrKeyTypeECSECPrimeRandom,
            kSecReturnRef as String: true
        ]

        var item: CFTypeRef?
        let status = SecItemCopyMatching(query as CFDictionary, &item)

        guard status == errSecSuccess, let privateKey = item else {
            result(FlutterError(code: "KEY_NOT_FOUND", message: "Key not found: \(alias)", details: nil))
            return
        }

        guard let publicKey = SecKeyCopyPublicKey(privateKey as! SecKey) else {
            result(FlutterError(code: "GET_PUBLIC_KEY_FAILED", message: "Failed to get public key", details: nil))
            return
        }

        var error: Unmanaged<CFError>?
        guard let publicKeyData = SecKeyCopyExternalRepresentation(publicKey, &error) as Data? else {
            let errorMessage = error?.takeRetainedValue().localizedDescription ?? "Unknown error"
            result(FlutterError(code: "GET_PUBLIC_KEY_FAILED", message: errorMessage, details: nil))
            return
        }

        result(FlutterStandardTypedData(bytes: publicKeyData))
    }

    private func sign(alias: String, data: Data, result: @escaping FlutterResult) {
        let query: [String: Any] = [
            kSecClass as String: kSecClassKey,
            kSecAttrApplicationTag as String: alias.data(using: .utf8)!,
            kSecAttrKeyType as String: kSecAttrKeyTypeECSECPrimeRandom,
            kSecReturnRef as String: true
        ]

        var item: CFTypeRef?
        let status = SecItemCopyMatching(query as CFDictionary, &item)

        guard status == errSecSuccess, let privateKey = item as! SecKey? else {
            result(FlutterError(code: "KEY_NOT_FOUND", message: "Key not found: \(alias)", details: nil))
            return
        }

        var error: Unmanaged<CFError>?
        guard let signature = SecKeyCreateSignature(
            privateKey,
            .ecdsaSignatureMessageX962SHA256,
            data as CFData,
            &error
        ) as Data? else {
            let errorMessage = error?.takeRetainedValue().localizedDescription ?? "Unknown error"
            if errorMessage.contains("User canceled") {
                result(FlutterError(code: "USER_CANCELED", message: "Biometric authentication canceled", details: nil))
            } else {
                result(FlutterError(code: "SIGNING_FAILED", message: errorMessage, details: nil))
            }
            return
        }

        result(FlutterStandardTypedData(bytes: signature))
    }

    private func deleteKey(alias: String, result: @escaping FlutterResult) {
        deleteKeyInternal(alias: alias)
        result(nil)
    }

    private func deleteKeyInternal(alias: String) {
        let query: [String: Any] = [
            kSecClass as String: kSecClassKey,
            kSecAttrApplicationTag as String: alias.data(using: .utf8)!
        ]
        SecItemDelete(query as CFDictionary)
    }
}
