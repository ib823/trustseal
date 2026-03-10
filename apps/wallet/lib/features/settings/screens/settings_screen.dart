import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../app/router.dart';
import '../../../core/i18n/app_localizations.dart';
import '../../../core/theme/sahi_colors.dart';
import '../../../services/auth_service.dart';
import '../../../services/security_service.dart';

/// Settings screen.
class SettingsScreen extends ConsumerWidget {
  const SettingsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final authState = ref.watch(authStateProvider);

    return Scaffold(
      appBar: AppBar(
        title: Text(l10n.settingsTitle),
      ),
      body: ListView(
        children: [
          // Security warning if device is compromised
          if (SecurityService.isDeviceCompromised) ...[
            Container(
              margin: const EdgeInsets.all(16),
              padding: const EdgeInsets.all(16),
              decoration: BoxDecoration(
                color: SahiColors.signalWarning.withOpacity(0.1),
                borderRadius: BorderRadius.circular(12),
                border: Border.all(
                  color: SahiColors.signalWarning.withOpacity(0.3),
                ),
              ),
              child: Row(
                children: [
                  const Icon(
                    Icons.warning_amber,
                    color: SahiColors.signalWarning,
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    child: Text(
                      l10n.securityWarningRooted,
                      style: const TextStyle(
                        color: SahiColors.signalWarning,
                        fontSize: 14,
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ],

          // Account section
          _buildSectionHeader(context, l10n.settingsAccount),
          _buildListTile(
            context,
            icon: Icons.person_outline,
            title: 'User ID',
            subtitle: authState.userId ?? 'Not registered',
          ),
          _buildListTile(
            context,
            icon: Icons.smartphone,
            title: 'Device ID',
            subtitle: authState.deviceId ?? 'Not registered',
          ),

          const Divider(height: 32),

          // Security section
          _buildSectionHeader(context, l10n.settingsSecurity),
          _buildListTile(
            context,
            icon: Icons.security,
            title: l10n.settingsSecurity,
            trailing: const Icon(Icons.chevron_right),
            onTap: () => context.push(Routes.securitySettings),
          ),
          SwitchListTile(
            secondary: const Icon(Icons.fingerprint),
            title: Text(l10n.settingsBiometrics),
            subtitle: Text(l10n.settingsBiometricsSubtitle),
            value: authState.biometricsEnabled,
            onChanged: (value) {
              // TODO: Toggle biometrics requirement
            },
          ),

          const Divider(height: 32),

          // App section
          _buildSectionHeader(context, l10n.settingsAbout),
          _buildListTile(
            context,
            icon: Icons.language,
            title: l10n.settingsLanguage,
            subtitle: 'English',
            trailing: const Icon(Icons.chevron_right),
            onTap: () {
              // TODO: Show language picker
            },
          ),
          _buildListTile(
            context,
            icon: Icons.info_outline,
            title: l10n.settingsVersion,
            subtitle: '1.0.0 (1)',
          ),

          const Divider(height: 32),

          // Logout
          Padding(
            padding: const EdgeInsets.all(16),
            child: OutlinedButton(
              onPressed: () => _showLogoutDialog(context, ref, l10n),
              style: OutlinedButton.styleFrom(
                foregroundColor: SahiColors.signalError,
                side: const BorderSide(color: SahiColors.signalError),
              ),
              child: Text(l10n.settingsLogout),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildSectionHeader(BuildContext context, String title) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 16, 16, 8),
      child: Text(
        title.toUpperCase(),
        style: Theme.of(context).textTheme.labelSmall?.copyWith(
              letterSpacing: 1.2,
            ),
      ),
    );
  }

  Widget _buildListTile(
    BuildContext context, {
    required IconData icon,
    required String title,
    String? subtitle,
    Widget? trailing,
    VoidCallback? onTap,
  }) {
    return ListTile(
      leading: Icon(icon),
      title: Text(title),
      subtitle: subtitle != null ? Text(subtitle) : null,
      trailing: trailing,
      onTap: onTap,
    );
  }

  Future<void> _showLogoutDialog(
    BuildContext context,
    WidgetRef ref,
    AppLocalizations l10n,
  ) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(l10n.settingsLogout),
        content: Text(l10n.settingsLogoutConfirm),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: Text(l10n.cancel),
          ),
          TextButton(
            onPressed: () => Navigator.pop(context, true),
            style: TextButton.styleFrom(
              foregroundColor: SahiColors.signalError,
            ),
            child: Text(l10n.settingsLogout),
          ),
        ],
      ),
    );

    if (confirmed == true) {
      await ref.read(authServiceProvider).logout();
      if (context.mounted) {
        context.go(Routes.welcome);
      }
    }
  }
}
