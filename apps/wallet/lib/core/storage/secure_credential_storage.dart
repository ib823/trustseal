import 'dart:convert';

import 'package:flutter_secure_storage/flutter_secure_storage.dart';

import '../models/credential.dart';

/// Secure storage for credentials using flutter_secure_storage.
///
/// Credentials are encrypted using AES with a keystore-wrapped key.
/// On Android: Uses Android Keystore (StrongBox preferred)
/// On iOS: Uses iOS Keychain
class SecureCredentialStorage {
  static const _credentialsKey = 'vaultpass_credentials';
  static const _userIdKey = 'vaultpass_user_id';
  static const _deviceIdKey = 'vaultpass_device_id';

  final FlutterSecureStorage _storage;

  SecureCredentialStorage()
      : _storage = const FlutterSecureStorage(
          aOptions: AndroidOptions(
            encryptedSharedPreferences: true,
            keyCipherAlgorithm:
                KeyCipherAlgorithm.RSA_ECB_OAEPwithSHA_256andMGF1Padding,
            storageCipherAlgorithm: StorageCipherAlgorithm.AES_GCM_NoPadding,
          ),
          iOptions: IOSOptions(
            accessibility: KeychainAccessibility.first_unlock_this_device,
          ),
        );

  /// Store a credential.
  Future<void> storeCredential(Credential credential) async {
    final credentials = await getAllCredentials();
    credentials.removeWhere((c) => c.id == credential.id);
    credentials.add(credential);
    await _saveCredentials(credentials);
  }

  /// Get a credential by ID.
  Future<Credential?> getCredential(String id) async {
    final credentials = await getAllCredentials();
    try {
      return credentials.firstWhere((c) => c.id == id);
    } catch (_) {
      return null;
    }
  }

  /// Get all credentials.
  Future<List<Credential>> getAllCredentials() async {
    final json = await _storage.read(key: _credentialsKey);
    if (json == null || json.isEmpty) {
      return [];
    }

    try {
      final List<dynamic> list = jsonDecode(json);
      return list.map((e) => Credential.fromJson(e)).toList();
    } catch (_) {
      return [];
    }
  }

  /// Delete a credential by ID.
  Future<void> deleteCredential(String id) async {
    final credentials = await getAllCredentials();
    credentials.removeWhere((c) => c.id == id);
    await _saveCredentials(credentials);
  }

  /// Delete all credentials.
  Future<void> deleteAllCredentials() async {
    await _storage.delete(key: _credentialsKey);
  }

  /// Store user ID.
  Future<void> storeUserId(String userId) async {
    await _storage.write(key: _userIdKey, value: userId);
  }

  /// Get user ID.
  Future<String?> getUserId() async {
    return await _storage.read(key: _userIdKey);
  }

  /// Store device ID.
  Future<void> storeDeviceId(String deviceId) async {
    await _storage.write(key: _deviceIdKey, value: deviceId);
  }

  /// Get device ID.
  Future<String?> getDeviceId() async {
    return await _storage.read(key: _deviceIdKey);
  }

  /// Clear all stored data (logout).
  Future<void> clearAll() async {
    await _storage.deleteAll();
  }

  Future<void> _saveCredentials(List<Credential> credentials) async {
    final json = jsonEncode(credentials.map((c) => c.toJson()).toList());
    await _storage.write(key: _credentialsKey, value: json);
  }
}
