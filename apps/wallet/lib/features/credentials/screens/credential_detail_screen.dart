import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:intl/intl.dart';

import '../../../app/router.dart';
import '../../../core/i18n/app_localizations.dart';
import '../../../core/models/credential.dart';
import '../../../core/storage/secure_credential_storage.dart';
import '../../../core/theme/sahi_colors.dart';
import '../../../core/theme/sahi_typography.dart';
import '../widgets/credential_status_badge.dart';

/// Provider for a single credential.
final credentialDetailProvider =
    FutureProvider.family<Credential?, String>((ref, id) async {
  final storage = SecureCredentialStorage();
  return await storage.getCredential(id);
});

/// Credential detail screen.
///
/// Shows full credential information and presentation button.
class CredentialDetailScreen extends ConsumerWidget {
  final String credentialId;

  const CredentialDetailScreen({
    super.key,
    required this.credentialId,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final credentialAsync = ref.watch(credentialDetailProvider(credentialId));

    return Scaffold(
      appBar: AppBar(
        title: Text(l10n.credentialDetail),
      ),
      body: credentialAsync.when(
        data: (credential) {
          if (credential == null) {
            return Center(child: Text(l10n.error));
          }
          return _buildContent(context, l10n, credential);
        },
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (_, __) => Center(child: Text(l10n.error)),
      ),
    );
  }

  Widget _buildContent(
    BuildContext context,
    AppLocalizations l10n,
    Credential credential,
  ) {
    final dateFormat = DateFormat.yMMMd();

    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Credential type header
          _buildHeader(context, l10n, credential),
          const SizedBox(height: 32),

          // Status
          _buildSection(
            context,
            'Status',
            child: CredentialStatusBadge(status: credential.status),
          ),
          const SizedBox(height: 24),

          // Property
          _buildSection(
            context,
            'Property',
            value: credential.propertyName,
          ),
          if (credential.unitId != null) ...[
            const SizedBox(height: 16),
            _buildSection(
              context,
              'Unit',
              value: credential.unitId!,
            ),
          ],
          const SizedBox(height: 24),

          // Dates
          _buildSection(
            context,
            l10n.issuedOn,
            value: dateFormat.format(credential.issuedAt),
          ),
          const SizedBox(height: 16),
          _buildSection(
            context,
            l10n.expiresOn,
            value: dateFormat.format(credential.expiresAt),
            valueColor: credential.status == CredentialStatus.expiringSoon
                ? SahiColors.signalWarning
                : null,
          ),
          const SizedBox(height: 32),

          // Technical details (collapsible)
          _buildTechnicalDetails(context, credential),
          const SizedBox(height: 48),

          // Present button (primary action)
          SizedBox(
            width: double.infinity,
            child: ElevatedButton.icon(
              onPressed: credential.canPresent
                  ? () => context.push(
                        Routes.presentation
                            .replaceFirst(':id', credential.id),
                      )
                  : null,
              icon: const Icon(Icons.nfc),
              label: Text(l10n.presentCredential),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildHeader(
    BuildContext context,
    AppLocalizations l10n,
    Credential credential,
  ) {
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

    return Row(
      children: [
        Container(
          width: 56,
          height: 56,
          decoration: BoxDecoration(
            color: SahiColors.slate800,
            borderRadius: BorderRadius.circular(12),
          ),
          child: Icon(icon, size: 28, color: SahiColors.slate300),
        ),
        const SizedBox(width: 16),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                typeName,
                style: Theme.of(context).textTheme.headlineSmall,
              ),
              const SizedBox(height: 4),
              Text(
                credential.id,
                style: SahiTypography.monoSmall,
              ),
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildSection(
    BuildContext context,
    String label, {
    String? value,
    Widget? child,
    Color? valueColor,
  }) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          label,
          style: Theme.of(context).textTheme.labelSmall,
        ),
        const SizedBox(height: 4),
        if (child != null)
          child
        else
          Text(
            value ?? '',
            style: Theme.of(context).textTheme.bodyLarge?.copyWith(
                  color: valueColor,
                ),
          ),
      ],
    );
  }

  Widget _buildTechnicalDetails(BuildContext context, Credential credential) {
    return ExpansionTile(
      title: Text(
        'Technical Details',
        style: Theme.of(context).textTheme.titleSmall,
      ),
      tilePadding: EdgeInsets.zero,
      childrenPadding: const EdgeInsets.only(top: 8),
      children: [
        _buildDetailRow(context, 'Issuer DID', credential.issuerDid),
        const SizedBox(height: 8),
        _buildDetailRow(context, 'Subject DID', credential.subjectDid),
        const SizedBox(height: 8),
        _buildDetailRow(
            context, 'Status List Index', credential.statusListIndex.toString()),
        const SizedBox(height: 8),
        _buildDetailRow(context, 'Disclosable Claims',
            credential.selectiveDisclosureClaims.join(', ')),
      ],
    );
  }

  Widget _buildDetailRow(BuildContext context, String label, String value) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        SizedBox(
          width: 120,
          child: Text(
            label,
            style: Theme.of(context).textTheme.bodySmall,
          ),
        ),
        Expanded(
          child: Text(
            value,
            style: SahiTypography.monoSmall,
          ),
        ),
      ],
    );
  }
}
