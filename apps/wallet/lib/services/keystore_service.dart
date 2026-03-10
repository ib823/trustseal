import 'dart:typed_data';

import 'package:flutter/services.dart';

/// Platform channel service for hardware-bound key operations.
///
/// Android: Uses Android Keystore (StrongBox preferred)
/// iOS: Uses Secure Enclave
///
/// Keys are:
/// - Hardware-bound (cannot be extracted)
/// - Biometric-protected (userAuthenticationRequired)
/// - Non-exportable
class KeystoreService {
  static const _channel = MethodChannel('my.sahi.vaultpass/keystore');

  static const String _keyAlias = 'vaultpass_device_key';

  /// Check if hardware-backed keystore is available.
  static Future<bool> isHardwareBackedAvailable() async {
    try {
      final result = await _channel.invokeMethod<bool>('isHardwareBackedAvailable');
      return result ?? false;
    } on PlatformException {
      return false;
    }
  }

  /// Check if StrongBox (Android) or Secure Enclave (iOS) is available.
  static Future<bool> isStrongBoxAvailable() async {
    try {
      final result = await _channel.invokeMethod<bool>('isStrongBoxAvailable');
      return result ?? false;
    } on PlatformException {
      return false;
    }
  }

  /// Generate a new device-bound key pair.
  ///
  /// Returns the public key bytes (for registration with the server).
  /// The private key remains in hardware and cannot be extracted.
  static Future<Uint8List> generateKeyPair({
    bool requireBiometric = true,
  }) async {
    try {
      final result = await _channel.invokeMethod<Uint8List>(
        'generateKeyPair',
        {
          'alias': _keyAlias,
          'requireBiometric': requireBiometric,
        },
      );
      if (result == null) {
        throw KeystoreException('Failed to generate key pair');
      }
      return result;
    } on PlatformException catch (e) {
      throw KeystoreException('Key generation failed: ${e.message}');
    }
  }

  /// Check if a device key exists.
  static Future<bool> hasDeviceKey() async {
    try {
      final result = await _channel.invokeMethod<bool>(
        'hasKey',
        {'alias': _keyAlias},
      );
      return result ?? false;
    } on PlatformException {
      return false;
    }
  }

  /// Get the public key bytes.
  static Future<Uint8List?> getPublicKey() async {
    try {
      return await _channel.invokeMethod<Uint8List>(
        'getPublicKey',
        {'alias': _keyAlias},
      );
    } on PlatformException {
      return null;
    }
  }

  /// Sign data with the device key.
  ///
  /// This will trigger biometric authentication if the key requires it.
  static Future<Uint8List> sign(Uint8List data) async {
    try {
      final result = await _channel.invokeMethod<Uint8List>(
        'sign',
        {
          'alias': _keyAlias,
          'data': data,
        },
      );
      if (result == null) {
        throw KeystoreException('Signing failed');
      }
      return result;
    } on PlatformException catch (e) {
      if (e.code == 'USER_CANCELED') {
        throw KeystoreBiometricCanceledException();
      }
      if (e.code == 'BIOMETRIC_FAILED') {
        throw KeystoreBiometricFailedException();
      }
      throw KeystoreException('Signing failed: ${e.message}');
    }
  }

  /// Delete the device key.
  ///
  /// WARNING: This is irreversible. All credentials will become unusable.
  static Future<void> deleteKey() async {
    try {
      await _channel.invokeMethod<void>(
        'deleteKey',
        {'alias': _keyAlias},
      );
    } on PlatformException catch (e) {
      throw KeystoreException('Key deletion failed: ${e.message}');
    }
  }
}

/// Base exception for keystore operations.
class KeystoreException implements Exception {
  final String message;
  KeystoreException(this.message);

  @override
  String toString() => 'KeystoreException: $message';
}

/// Thrown when user cancels biometric authentication.
class KeystoreBiometricCanceledException extends KeystoreException {
  KeystoreBiometricCanceledException() : super('Biometric authentication canceled');
}

/// Thrown when biometric authentication fails.
class KeystoreBiometricFailedException extends KeystoreException {
  KeystoreBiometricFailedException() : super('Biometric authentication failed');
}
