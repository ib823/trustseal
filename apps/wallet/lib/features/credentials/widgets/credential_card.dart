import 'package:flutter/material.dart';

import '../../../core/i18n/app_localizations.dart';
import '../../../core/models/credential.dart';
import '../../../core/theme/sahi_colors.dart';
import '../../../core/theme/sahi_typography.dart';
import 'credential_status_badge.dart';

/// Credential card widget.
///
/// Displays a credential with subtle gradient background per type.
class CredentialCard extends StatelessWidget {
  final Credential credential;
  final VoidCallback? onTap;

  const CredentialCard({
    super.key,
    required this.credential,
    this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);

    final gradient = switch (credential.type) {
      CredentialType.residentBadge => SahiColors.residentBadgeGradient,
      CredentialType.visitorPass => SahiColors.visitorPassGradient,
      CredentialType.contractorBadge => SahiColors.contractorBadgeGradient,
      CredentialType.emergencyAccess => SahiColors.emergencyAccessGradient,
    };

    final typeName = switch (credential.type) {
      CredentialType.residentBadge => l10n.residentBadge,
      CredentialType.visitorPass => l10n.visitorPass,
      CredentialType.contractorBadge => l10n.contractorBadge,
      CredentialType.emergencyAccess => l10n.emergencyAccess,
    };

    final icon = switch (credential.type) {
      CredentialType.residentBadge => Icons.home_outlined,
      CredentialType.visitorPass => Icons.person_outline,
      CredentialType.contractorBadge => Icons.engineering_outlined,
      CredentialType.emergencyAccess => Icons.emergency_outlined,
    };

    return GestureDetector(
      onTap: onTap,
      child: Container(
        decoration: BoxDecoration(
          gradient: gradient,
          borderRadius: BorderRadius.circular(16),
          border: Border.all(color: SahiColors.slate700),
        ),
        child: Padding(
          padding: const EdgeInsets.all(20),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // Header row
              Row(
                children: [
                  Icon(icon, color: SahiColors.slate300, size: 24),
                  const SizedBox(width: 12),
                  Expanded(
                    child: Text(
                      typeName,
                      style: Theme.of(context).textTheme.titleMedium,
                    ),
                  ),
                  CredentialStatusBadge(status: credential.status),
                ],
              ),
              const SizedBox(height: 16),

              // Property name
              Text(
                credential.propertyName,
                style: Theme.of(context).textTheme.headlineSmall,
              ),
              if (credential.unitId != null) ...[
                const SizedBox(height: 4),
                Text(
                  'Unit ${credential.unitId}',
                  style: Theme.of(context).textTheme.bodyMedium,
                ),
              ],
              const SizedBox(height: 16),

              // Footer
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Text(
                    credential.id,
                    style: SahiTypography.monoSmall,
                  ),
                  Icon(
                    Icons.arrow_forward_ios,
                    size: 14,
                    color: SahiColors.slate500,
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }
}
