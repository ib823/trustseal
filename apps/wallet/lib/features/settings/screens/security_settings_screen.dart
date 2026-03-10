import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/i18n/app_localizations.dart';
import '../../../core/theme/sahi_colors.dart';
import '../../../core/theme/sahi_typography.dart';
import '../../../services/keystore_service.dart';
import '../../../services/security_service.dart';

/// Security settings screen.
///
/// Shows device security status and key information.
class SecuritySettingsScreen extends ConsumerStatefulWidget {
  const SecuritySettingsScreen({super.key});

  @override
  ConsumerState<SecuritySettingsScreen> createState() =>
      _SecuritySettingsScreenState();
}

class _SecuritySettingsScreenState
    extends ConsumerState<SecuritySettingsScreen> {
  bool _hasStrongBox = false;
  bool _hasHardwareBacked = false;
  bool _hasDeviceKey = false;

  @override
  void initState() {
    super.initState();
    _loadSecurityInfo();
  }

  Future<void> _loadSecurityInfo() async {
    final hasStrongBox = await KeystoreService.isStrongBoxAvailable();
    final hasHardwareBacked = await KeystoreService.isHardwareBackedAvailable();
    final hasDeviceKey = await KeystoreService.hasDeviceKey();

    if (mounted) {
      setState(() {
        _hasStrongBox = hasStrongBox;
        _hasHardwareBacked = hasHardwareBacked;
        _hasDeviceKey = hasDeviceKey;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);

    return Scaffold(
      appBar: AppBar(
        title: Text(l10n.securityTitle),
      ),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          // Device security status
          _buildSecurityCard(
            context,
            title: 'Device Security',
            items: [
              _SecurityItem(
                label: 'Hardware-backed Keystore',
                value: _hasHardwareBacked ? 'Available' : 'Not available',
                isSecure: _hasHardwareBacked,
              ),
              _SecurityItem(
                label: 'StrongBox / Secure Enclave',
                value: _hasStrongBox ? 'Available' : 'Not available',
                isSecure: _hasStrongBox,
              ),
              _SecurityItem(
                label: 'Device Key',
                value: _hasDeviceKey ? 'Generated' : 'Not generated',
                isSecure: _hasDeviceKey,
              ),
            ],
          ),
          const SizedBox(height: 16),

          // Threat detection
          _buildSecurityCard(
            context,
            title: 'Integrity Checks',
            items: [
              _SecurityItem(
                label: 'Root/Jailbreak Detection',
                value: SecurityService.isDeviceCompromised
                    ? 'Warning: Detected'
                    : 'Passed',
                isSecure: !SecurityService.isDeviceCompromised,
              ),
              _SecurityItem(
                label: 'Debug Mode',
                value: SecurityService.detectedThreats
                        .contains(SecurityThreat.developerMode)
                    ? 'Enabled'
                    : 'Disabled',
                isSecure: !SecurityService.detectedThreats
                    .contains(SecurityThreat.developerMode),
              ),
            ],
          ),
          const SizedBox(height: 24),

          // Info section
          Container(
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              color: SahiColors.slate900,
              borderRadius: BorderRadius.circular(12),
              border: Border.all(color: SahiColors.slate800),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: [
                    const Icon(
                      Icons.info_outline,
                      size: 20,
                      color: SahiColors.slate400,
                    ),
                    const SizedBox(width: 8),
                    Text(
                      'About Device Binding',
                      style: Theme.of(context).textTheme.titleSmall,
                    ),
                  ],
                ),
                const SizedBox(height: 12),
                Text(
                  l10n.securityDeviceBound,
                  style: Theme.of(context).textTheme.bodyMedium,
                ),
                const SizedBox(height: 8),
                Text(
                  'Your credentials cannot be copied to another device. If you lose this device, contact your property administrator to issue new credentials.',
                  style: Theme.of(context).textTheme.bodySmall,
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildSecurityCard(
    BuildContext context, {
    required String title,
    required List<_SecurityItem> items,
  }) {
    return Container(
      decoration: BoxDecoration(
        color: SahiColors.slate900,
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: SahiColors.slate800),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Padding(
            padding: const EdgeInsets.all(16),
            child: Text(
              title,
              style: Theme.of(context).textTheme.titleSmall,
            ),
          ),
          const Divider(height: 1),
          ...items.map((item) => _buildSecurityItem(context, item)),
        ],
      ),
    );
  }

  Widget _buildSecurityItem(BuildContext context, _SecurityItem item) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      child: Row(
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  item.label,
                  style: Theme.of(context).textTheme.bodyMedium,
                ),
                const SizedBox(height: 2),
                Text(
                  item.value,
                  style: SahiTypography.monoSmall.copyWith(
                    color: item.isSecure
                        ? SahiColors.signalSuccess
                        : SahiColors.signalWarning,
                  ),
                ),
              ],
            ),
          ),
          Icon(
            item.isSecure ? Icons.check_circle : Icons.warning_amber,
            color: item.isSecure
                ? SahiColors.signalSuccess
                : SahiColors.signalWarning,
            size: 20,
          ),
        ],
      ),
    );
  }
}

class _SecurityItem {
  final String label;
  final String value;
  final bool isSecure;

  const _SecurityItem({
    required this.label,
    required this.value,
    required this.isSecure,
  });
}
