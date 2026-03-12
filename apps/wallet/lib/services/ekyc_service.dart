import 'package:dio/dio.dart';
import 'package:url_launcher/url_launcher.dart';

/// VP-9: eKYC service for MyDigital ID integration.
///
/// Provides OAuth 2.0 PKCE flow for identity verification via MyDigital ID.
class EkycService {
  EkycService({
    String? baseUrl,
    Dio? dio,
  })  : _baseUrl = baseUrl ??
            const String.fromEnvironment(
              'API_BASE_URL',
              defaultValue: 'http://localhost:3000',
            ),
        _dio = dio ?? Dio();

  final String _baseUrl;
  final Dio _dio;

  /// Current verification state.
  EkycVerification? _currentVerification;

  /// Get current verification if any.
  EkycVerification? get currentVerification => _currentVerification;

  /// Initiate eKYC verification.
  ///
  /// Returns the authorization URL that should be opened in a browser.
  Future<InitiateResult> initiateVerification({
    required String tenantId,
    String? userId,
  }) async {
    try {
      final response = await _dio.post<Map<String, dynamic>>(
        '$_baseUrl/api/v1/ekyc/initiate',
        data: {
          'tenant_id': tenantId,
          if (userId != null) 'user_id': userId,
        },
      );

      final data = response.data!;

      _currentVerification = EkycVerification(
        id: data['verification_id'] as String,
        status: VerificationStatus.pending,
        state: data['state'] as String,
        expiresAt: DateTime.parse(data['expires_at'] as String),
      );

      return InitiateResult(
        verificationId: data['verification_id'] as String,
        authorizationUrl: data['authorization_url'] as String,
        state: data['state'] as String,
      );
    } on DioException catch (e) {
      throw EkycException(
        'Failed to initiate verification: ${e.message}',
        code: 'SAHI_2300',
      );
    }
  }

  /// Open authorization URL in system browser.
  ///
  /// The user will complete verification in MyDigital ID and be redirected
  /// back to the app via deep link.
  Future<bool> openAuthorizationUrl(String url) async {
    final uri = Uri.parse(url);
    if (await canLaunchUrl(uri)) {
      return launchUrl(
        uri,
        mode: LaunchMode.externalApplication,
      );
    }
    return false;
  }

  /// Handle OAuth callback from deep link.
  ///
  /// Called when the app receives the callback after user completes
  /// verification in MyDigital ID.
  Future<CallbackResult> handleCallback({
    required String code,
    required String state,
  }) async {
    if (_currentVerification == null) {
      throw const EkycException(
        'No verification in progress',
        code: 'SAHI_2300',
      );
    }

    // Verify state matches to prevent CSRF
    if (_currentVerification!.state != state) {
      throw const EkycException(
        'State mismatch - possible security issue',
        code: 'SAHI_2307',
      );
    }

    try {
      final response = await _dio.post<Map<String, dynamic>>(
        '$_baseUrl/api/v1/ekyc/callback',
        data: {
          'code': code,
          'state': state,
          'verification_id': _currentVerification!.id,
        },
      );

      final data = response.data!;

      _currentVerification = _currentVerification!.copyWith(
        status: _parseStatus(data['status'] as String),
        assuranceLevel: _parseAssuranceLevel(data['assurance_level'] as String),
        verifiedAt: DateTime.parse(data['verified_at'] as String),
        expiresAt: DateTime.parse(data['expires_at'] as String),
      );

      return CallbackResult(
        verificationId: data['verification_id'] as String,
        status: _currentVerification!.status,
        assuranceLevel: _currentVerification!.assuranceLevel,
      );
    } on DioException catch (e) {
      _currentVerification = _currentVerification!.copyWith(
        status: VerificationStatus.failed,
      );
      throw EkycException(
        'Verification callback failed: ${e.message}',
        code: 'SAHI_2300',
      );
    }
  }

  /// Get verification status.
  Future<EkycVerification> getStatus(String verificationId) async {
    try {
      final response = await _dio.get<Map<String, dynamic>>(
        '$_baseUrl/api/v1/ekyc/status/$verificationId',
      );

      final data = response.data!;

      return EkycVerification(
        id: data['verification_id'] as String,
        status: _parseStatus(data['status'] as String),
        assuranceLevel: _parseAssuranceLevel(data['assurance_level'] as String),
        did: data['did'] as String?,
        verifiedAt: data['verified_at'] != null
            ? DateTime.parse(data['verified_at'] as String)
            : null,
        expiresAt: data['expires_at'] != null
            ? DateTime.parse(data['expires_at'] as String)
            : null,
      );
    } on DioException catch (e) {
      throw EkycException(
        'Failed to get verification status: ${e.message}',
        code: 'SAHI_2300',
      );
    }
  }

  /// Bind wallet DID to verified identity.
  ///
  /// Should be called after key generation to link the DID to the
  /// verified identity.
  Future<void> bindDid({
    required String verificationId,
    required String did,
  }) async {
    try {
      await _dio.post<void>(
        '$_baseUrl/api/v1/ekyc/bind-did',
        data: {
          'verification_id': verificationId,
          'did': did,
        },
      );
    } on DioException catch (e) {
      throw EkycException(
        'Failed to bind DID: ${e.message}',
        code: 'SAHI_2300',
      );
    }
  }

  /// Clear current verification state.
  void clearVerification() {
    _currentVerification = null;
  }

  VerificationStatus _parseStatus(String status) {
    return switch (status) {
      'pending' => VerificationStatus.pending,
      'in_progress' => VerificationStatus.inProgress,
      'verified' => VerificationStatus.verified,
      'failed' => VerificationStatus.failed,
      'expired' => VerificationStatus.expired,
      _ => VerificationStatus.pending,
    };
  }

  AssuranceLevel _parseAssuranceLevel(String level) {
    return switch (level) {
      'P1' => AssuranceLevel.p1,
      'P2' => AssuranceLevel.p2,
      'P3' => AssuranceLevel.p3,
      _ => AssuranceLevel.p1,
    };
  }
}

/// Result of initiating verification.
class InitiateResult {
  const InitiateResult({
    required this.verificationId,
    required this.authorizationUrl,
    required this.state,
  });

  final String verificationId;
  final String authorizationUrl;
  final String state;
}

/// Result of handling callback.
class CallbackResult {
  const CallbackResult({
    required this.verificationId,
    required this.status,
    required this.assuranceLevel,
  });

  final String verificationId;
  final VerificationStatus status;
  final AssuranceLevel assuranceLevel;
}

/// eKYC verification state.
class EkycVerification {
  const EkycVerification({
    required this.id,
    required this.status,
    this.state,
    this.assuranceLevel = AssuranceLevel.p1,
    this.did,
    this.verifiedAt,
    this.expiresAt,
  });

  final String id;
  final VerificationStatus status;
  final String? state;
  final AssuranceLevel assuranceLevel;
  final String? did;
  final DateTime? verifiedAt;
  final DateTime? expiresAt;

  EkycVerification copyWith({
    String? id,
    VerificationStatus? status,
    String? state,
    AssuranceLevel? assuranceLevel,
    String? did,
    DateTime? verifiedAt,
    DateTime? expiresAt,
  }) {
    return EkycVerification(
      id: id ?? this.id,
      status: status ?? this.status,
      state: state ?? this.state,
      assuranceLevel: assuranceLevel ?? this.assuranceLevel,
      did: did ?? this.did,
      verifiedAt: verifiedAt ?? this.verifiedAt,
      expiresAt: expiresAt ?? this.expiresAt,
    );
  }
}

/// Verification status.
enum VerificationStatus {
  pending,
  inProgress,
  verified,
  failed,
  expired,
}

/// Assurance level per TrustMark spec.
enum AssuranceLevel {
  /// Basic: email/phone verified, synced passkey OK.
  p1,

  /// Enhanced: eKYC-verified identity, biometric passkey.
  p2,

  /// Qualified: CA-issued certificate, hardware security key.
  p3,
}

/// eKYC exception.
class EkycException implements Exception {
  const EkycException(this.message, {this.code});

  final String message;
  final String? code;

  @override
  String toString() => code != null ? '[$code] $message' : message;
}
