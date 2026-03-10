import 'dart:async';

import 'package:connectivity_plus/connectivity_plus.dart';
import 'package:dio/dio.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../core/models/credential.dart';
import '../core/storage/secure_credential_storage.dart';
import 'crypto_service.dart';

/// Background sync service for credentials and status lists.
///
/// Responsibilities:
/// - Sync credentials from server
/// - Fetch and cache status lists for offline revocation checking
/// - Background refresh on connectivity change
class SyncService {
  final SecureCredentialStorage _storage;
  final CryptoService _cryptoService;
  final Dio _dio;

  Timer? _periodicSyncTimer;
  StreamSubscription<List<ConnectivityResult>>? _connectivitySubscription;

  // Cached status lists (credential URL -> bitstring)
  final Map<String, String> _statusListCache = {};

  // Status list TTL (15 minutes per spec)
  static const _statusListTtl = Duration(minutes: 15);
  final Map<String, DateTime> _statusListTimestamps = {};

  SyncService({
    required SecureCredentialStorage storage,
    required CryptoService cryptoService,
    Dio? dio,
  })  : _storage = storage,
        _cryptoService = cryptoService,
        _dio = dio ??
            Dio(BaseOptions(
              baseUrl: const String.fromEnvironment(
                'API_BASE_URL',
                defaultValue: 'https://api.sahi.my',
              ),
              connectTimeout: const Duration(seconds: 10),
              receiveTimeout: const Duration(seconds: 10),
            ));

  /// Start background sync.
  void startBackgroundSync() {
    // Sync every 15 minutes
    _periodicSyncTimer = Timer.periodic(
      const Duration(minutes: 15),
      (_) => syncAll(),
    );

    // Sync on connectivity change
    _connectivitySubscription = Connectivity()
        .onConnectivityChanged
        .listen((results) {
      final hasConnection = results.any((r) =>
          r == ConnectivityResult.wifi || r == ConnectivityResult.mobile);
      if (hasConnection) {
        syncAll();
      }
    });
  }

  /// Stop background sync.
  void stopBackgroundSync() {
    _periodicSyncTimer?.cancel();
    _periodicSyncTimer = null;
    _connectivitySubscription?.cancel();
    _connectivitySubscription = null;
  }

  /// Sync all credentials and status lists.
  Future<void> syncAll() async {
    await syncCredentials();
    await syncStatusLists();
  }

  /// Sync credentials from server.
  Future<void> syncCredentials() async {
    try {
      final userId = await _storage.getUserId();
      if (userId == null) return;

      final response = await _dio.get('/v1/credentials', queryParameters: {
        'user_id': userId,
      });

      if (response.statusCode == 200) {
        final List<dynamic> data = response.data['credentials'];
        for (final json in data) {
          final credential = Credential.fromJson(json);
          await _storage.storeCredential(credential);
        }
      }
    } catch (e) {
      // Silently fail - we'll retry on next sync
    }
  }

  /// Sync status lists for all stored credentials.
  Future<void> syncStatusLists() async {
    try {
      final credentials = await _storage.getAllCredentials();

      // Collect unique status list URLs
      final urls = credentials.map((c) => c.statusListCredential).toSet();

      for (final url in urls) {
        await _fetchStatusList(url);
      }
    } catch (e) {
      // Silently fail
    }
  }

  /// Fetch and cache a status list.
  Future<void> _fetchStatusList(String url) async {
    // Check if cache is still valid
    final timestamp = _statusListTimestamps[url];
    if (timestamp != null &&
        DateTime.now().difference(timestamp) < _statusListTtl) {
      return; // Cache is still valid
    }

    try {
      final response = await _dio.get(url);
      if (response.statusCode == 200) {
        final statusList = response.data['credentialSubject']['encodedList'];
        _statusListCache[url] = statusList;
        _statusListTimestamps[url] = DateTime.now();
      }
    } catch (e) {
      // Keep stale cache if fetch fails
    }
  }

  /// Check if a credential is revoked.
  ///
  /// Uses cached status list for offline operation.
  Future<bool> isRevoked(Credential credential) async {
    final statusList = _statusListCache[credential.statusListCredential];
    if (statusList == null) {
      // No cached status list - assume not revoked (fail-open for checking)
      // Note: Gate verifiers fail-closed, but wallet shows best effort
      return false;
    }

    try {
      return await _cryptoService.checkRevocationStatus(
        statusListCredential: statusList,
        index: credential.statusListIndex,
      );
    } catch (e) {
      return false;
    }
  }

  /// Force refresh a specific credential's status.
  Future<bool> refreshCredentialStatus(Credential credential) async {
    // Invalidate cache
    _statusListTimestamps.remove(credential.statusListCredential);

    // Fetch fresh
    await _fetchStatusList(credential.statusListCredential);

    // Check status
    return await isRevoked(credential);
  }

  /// Clean up resources.
  void dispose() {
    stopBackgroundSync();
    _dio.close();
  }
}

/// Provider for sync service.
final syncServiceProvider = Provider<SyncService>((ref) {
  final storage = SecureCredentialStorage();
  final cryptoService = ref.watch(cryptoServiceProvider);

  final service = SyncService(
    storage: storage,
    cryptoService: cryptoService,
  );

  // Start background sync
  service.startBackgroundSync();

  ref.onDispose(() => service.dispose());
  return service;
});
