import 'package:equatable/equatable.dart';

/// Credential status.
enum CredentialStatus {
  valid,
  expired,
  revoked,
  expiringSoon,
}

/// Credential type matching VaultPass types.
enum CredentialType {
  residentBadge,
  visitorPass,
  contractorBadge,
  emergencyAccess,
}

/// A VaultPass credential stored in the wallet.
class Credential extends Equatable {
  /// Unique credential ID (ULID with CRD_ prefix).
  final String id;

  /// Credential type.
  final CredentialType type;

  /// Property ID this credential grants access to.
  final String propertyId;

  /// Property name for display.
  final String propertyName;

  /// Unit ID (for resident badges).
  final String? unitId;

  /// Raw SD-JWT token.
  final String sdJwt;

  /// Issuer DID.
  final String issuerDid;

  /// Subject DID (user's DID).
  final String subjectDid;

  /// Issue timestamp.
  final DateTime issuedAt;

  /// Expiration timestamp.
  final DateTime expiresAt;

  /// Status list index for revocation checking.
  final int statusListIndex;

  /// Status list credential URL.
  final String statusListCredential;

  /// Selective disclosure claims that can be revealed.
  final List<String> selectiveDisclosureClaims;

  const Credential({
    required this.id,
    required this.type,
    required this.propertyId,
    required this.propertyName,
    this.unitId,
    required this.sdJwt,
    required this.issuerDid,
    required this.subjectDid,
    required this.issuedAt,
    required this.expiresAt,
    required this.statusListIndex,
    required this.statusListCredential,
    required this.selectiveDisclosureClaims,
  });

  /// Current status based on expiration.
  /// Note: Revocation status requires checking the status list.
  CredentialStatus get status {
    final now = DateTime.now();
    if (now.isAfter(expiresAt)) {
      return CredentialStatus.expired;
    }
    // Expiring soon if less than 24 hours remaining
    if (expiresAt.difference(now).inHours < 24) {
      return CredentialStatus.expiringSoon;
    }
    return CredentialStatus.valid;
  }

  /// Whether this credential can be presented.
  bool get canPresent => status == CredentialStatus.valid;

  /// Time until expiration.
  Duration get timeUntilExpiration => expiresAt.difference(DateTime.now());

  factory Credential.fromJson(Map<String, dynamic> json) {
    return Credential(
      id: json['id'] as String,
      type: CredentialType.values.firstWhere(
        (t) => t.name == json['type'],
        orElse: () => CredentialType.residentBadge,
      ),
      propertyId: json['propertyId'] as String,
      propertyName: json['propertyName'] as String,
      unitId: json['unitId'] as String?,
      sdJwt: json['sdJwt'] as String,
      issuerDid: json['issuerDid'] as String,
      subjectDid: json['subjectDid'] as String,
      issuedAt: DateTime.parse(json['issuedAt'] as String),
      expiresAt: DateTime.parse(json['expiresAt'] as String),
      statusListIndex: json['statusListIndex'] as int,
      statusListCredential: json['statusListCredential'] as String,
      selectiveDisclosureClaims:
          List<String>.from(json['selectiveDisclosureClaims'] as List),
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'type': type.name,
      'propertyId': propertyId,
      'propertyName': propertyName,
      'unitId': unitId,
      'sdJwt': sdJwt,
      'issuerDid': issuerDid,
      'subjectDid': subjectDid,
      'issuedAt': issuedAt.toIso8601String(),
      'expiresAt': expiresAt.toIso8601String(),
      'statusListIndex': statusListIndex,
      'statusListCredential': statusListCredential,
      'selectiveDisclosureClaims': selectiveDisclosureClaims,
    };
  }

  @override
  List<Object?> get props => [
        id,
        type,
        propertyId,
        propertyName,
        unitId,
        sdJwt,
        issuerDid,
        subjectDid,
        issuedAt,
        expiresAt,
        statusListIndex,
        statusListCredential,
        selectiveDisclosureClaims,
      ];
}
