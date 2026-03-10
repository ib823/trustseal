import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../app/router.dart';
import '../../../core/i18n/app_localizations.dart';
import '../../../core/models/credential.dart';
import '../../../core/storage/secure_credential_storage.dart';
import '../../../core/theme/sahi_colors.dart';
import '../widgets/credential_card.dart';

/// Provider for credentials list.
final credentialsProvider = FutureProvider<List<Credential>>((ref) async {
  final storage = SecureCredentialStorage();
  return await storage.getAllCredentials();
});

/// Main credentials list screen (home).
///
/// Shows all stored credentials as cards.
/// One primary action per screen per spec.
class CredentialsScreen extends ConsumerWidget {
  const CredentialsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final credentialsAsync = ref.watch(credentialsProvider);

    return Scaffold(
      appBar: AppBar(
        title: Text(l10n.credentialsTitle),
        actions: [
          IconButton(
            icon: const Icon(Icons.settings_outlined),
            onPressed: () => context.push(Routes.settings),
          ),
        ],
      ),
      body: credentialsAsync.when(
        data: (credentials) {
          if (credentials.isEmpty) {
            return _buildEmptyState(context, l10n);
          }
          return _buildCredentialsList(context, credentials);
        },
        loading: () => const Center(
          child: CircularProgressIndicator(),
        ),
        error: (error, stack) => Center(
          child: Text(
            l10n.error,
            style: Theme.of(context).textTheme.bodyLarge,
          ),
        ),
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: () => context.push(Routes.scanning),
        backgroundColor: SahiColors.slate100,
        foregroundColor: SahiColors.slate950,
        child: const Icon(Icons.nfc),
      ),
    );
  }

  Widget _buildEmptyState(BuildContext context, AppLocalizations l10n) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(32),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(
              Icons.badge_outlined,
              size: 80,
              color: SahiColors.slate600,
            ),
            const SizedBox(height: 24),
            Text(
              l10n.noCredentials,
              style: Theme.of(context).textTheme.headlineSmall,
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: 8),
            Text(
              l10n.noCredentialsSubtitle,
              style: Theme.of(context).textTheme.bodyMedium,
              textAlign: TextAlign.center,
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildCredentialsList(
      BuildContext context, List<Credential> credentials) {
    return ListView.builder(
      padding: const EdgeInsets.all(16),
      itemCount: credentials.length,
      itemBuilder: (context, index) {
        final credential = credentials[index];
        return Padding(
          padding: const EdgeInsets.only(bottom: 16),
          child: CredentialCard(
            credential: credential,
            onTap: () => context.push(
              Routes.credentialDetail.replaceFirst(':id', credential.id),
            ),
          ),
        );
      },
    );
  }
}
