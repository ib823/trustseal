import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:local_auth/local_auth.dart';

import '../core/storage/secure_credential_storage.dart';
import 'keystore_service.dart';

/// Authentication state.
class AuthState {
  final bool isAuthenticated;
  final String? userId;
  final String? deviceId;
  final bool biometricsEnabled;
  final bool hasDeviceKey;

  const AuthState({
    this.isAuthenticated = false,
    this.userId,
    this.deviceId,
    this.biometricsEnabled = false,
    this.hasDeviceKey = false,
  });

  AuthState copyWith({
    bool? isAuthenticated,
    String? userId,
    String? deviceId,
    bool? biometricsEnabled,
    bool? hasDeviceKey,
  }) {
    return AuthState(
      isAuthenticated: isAuthenticated ?? this.isAuthenticated,
      userId: userId ?? this.userId,
      deviceId: deviceId ?? this.deviceId,
      biometricsEnabled: biometricsEnabled ?? this.biometricsEnabled,
      hasDeviceKey: hasDeviceKey ?? this.hasDeviceKey,
    );
  }
}

/// Authentication service.
///
/// Manages user authentication state and biometric enrollment.
class AuthService extends StateNotifier<AuthState> {
  final SecureCredentialStorage _storage;
  final LocalAuthentication _localAuth;

  AuthService({
    SecureCredentialStorage? storage,
    LocalAuthentication? localAuth,
  })  : _storage = storage ?? SecureCredentialStorage(),
        _localAuth = localAuth ?? LocalAuthentication(),
        super(const AuthState());

  /// Initialize auth state from storage.
  Future<void> initialize() async {
    final userId = await _storage.getUserId();
    final deviceId = await _storage.getDeviceId();
    final hasDeviceKey = await KeystoreService.hasDeviceKey();
    final canAuthenticate = await _localAuth.canCheckBiometrics;

    state = AuthState(
      isAuthenticated: userId != null && hasDeviceKey,
      userId: userId,
      deviceId: deviceId,
      biometricsEnabled: canAuthenticate,
      hasDeviceKey: hasDeviceKey,
    );
  }

  /// Check if biometrics are available.
  Future<bool> isBiometricsAvailable() async {
    final canAuthenticate = await _localAuth.canCheckBiometrics;
    final isDeviceSupported = await _localAuth.isDeviceSupported();
    return canAuthenticate && isDeviceSupported;
  }

  /// Get available biometric types.
  Future<List<BiometricType>> getAvailableBiometrics() async {
    return await _localAuth.getAvailableBiometrics();
  }

  /// Authenticate with biometrics.
  ///
  /// Returns true if authentication succeeds.
  Future<bool> authenticateWithBiometrics({
    required String reason,
  }) async {
    try {
      return await _localAuth.authenticate(
        localizedReason: reason,
        options: const AuthenticationOptions(
          stickyAuth: true,
          biometricOnly: true,
        ),
      );
    } catch (e) {
      return false;
    }
  }

  /// Complete registration.
  Future<void> completeRegistration({
    required String userId,
    required String deviceId,
  }) async {
    await _storage.storeUserId(userId);
    await _storage.storeDeviceId(deviceId);

    final hasDeviceKey = await KeystoreService.hasDeviceKey();

    state = state.copyWith(
      isAuthenticated: hasDeviceKey,
      userId: userId,
      deviceId: deviceId,
      hasDeviceKey: hasDeviceKey,
    );
  }

  /// Mark device key as generated.
  void markDeviceKeyGenerated() {
    state = state.copyWith(
      isAuthenticated: true,
      hasDeviceKey: true,
    );
  }

  /// Log out and clear all data.
  Future<void> logout() async {
    // Delete device key
    await KeystoreService.deleteKey();

    // Clear storage
    await _storage.clearAll();

    // Reset state
    state = const AuthState();
  }
}

/// Provider for auth state.
final authStateProvider =
    StateNotifierProvider<AuthService, AuthState>((ref) {
  final service = AuthService();

  // Initialize on creation
  service.initialize();

  return service;
});

/// Provider for auth service (for calling methods).
final authServiceProvider = Provider<AuthService>((ref) {
  return ref.watch(authStateProvider.notifier);
});
