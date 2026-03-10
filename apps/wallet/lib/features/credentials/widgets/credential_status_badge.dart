import 'package:flutter/material.dart';

import '../../../core/i18n/app_localizations.dart';
import '../../../core/models/credential.dart';
import '../../../core/theme/sahi_colors.dart';

/// Status badge for credentials.
///
/// Uses signal colors ONLY for semantic meaning per spec.
class CredentialStatusBadge extends StatelessWidget {
  final CredentialStatus status;

  const CredentialStatusBadge({
    super.key,
    required this.status,
  });

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);

    final (label, color, bgColor) = switch (status) {
      CredentialStatus.valid => (
          l10n.credentialValid,
          SahiColors.signalSuccess,
          SahiColors.signalSuccess.withOpacity(0.15),
        ),
      CredentialStatus.expired => (
          l10n.credentialExpired,
          SahiColors.signalError,
          SahiColors.signalError.withOpacity(0.15),
        ),
      CredentialStatus.revoked => (
          l10n.credentialRevoked,
          SahiColors.signalError,
          SahiColors.signalError.withOpacity(0.15),
        ),
      CredentialStatus.expiringSoon => (
          l10n.credentialExpiringSoon,
          SahiColors.signalWarning,
          SahiColors.signalWarning.withOpacity(0.15),
        ),
    };

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
      decoration: BoxDecoration(
        color: bgColor,
        borderRadius: BorderRadius.circular(12),
      ),
      child: Text(
        label,
        style: TextStyle(
          fontSize: 12,
          fontWeight: FontWeight.w500,
          color: color,
        ),
      ),
    );
  }
}
