import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../core/i18n/app_localizations.dart';
import '../../../core/theme/sahi_colors.dart';
import '../../../services/ble_service.dart';
import '../../../services/nfc_service.dart';

/// Scanning screen for detecting nearby verifiers.
///
/// Shows BLE scanning status and NFC option.
class ScanningScreen extends ConsumerStatefulWidget {
  const ScanningScreen({super.key});

  @override
  ConsumerState<ScanningScreen> createState() => _ScanningScreenState();
}

class _ScanningScreenState extends ConsumerState<ScanningScreen> {
  bool _bleAvailable = false;
  bool _nfcAvailable = false;
  bool _isScanning = false;

  @override
  void initState() {
    super.initState();
    _checkCapabilities();
  }

  Future<void> _checkCapabilities() async {
    final bleService = ref.read(bleServiceProvider);
    final nfcService = ref.read(nfcServiceProvider);

    final bleAvailable = await bleService.isAvailable();
    final nfcAvailable = await nfcService.isAvailable();

    if (mounted) {
      setState(() {
        _bleAvailable = bleAvailable;
        _nfcAvailable = nfcAvailable;
      });

      // Auto-start BLE scanning if available
      if (bleAvailable) {
        _startScanning();
      }
    }
  }

  Future<void> _startScanning() async {
    final bleService = ref.read(bleServiceProvider);

    setState(() => _isScanning = true);
    await bleService.startScanning();
  }

  Future<void> _stopScanning() async {
    final bleService = ref.read(bleServiceProvider);

    await bleService.stopScanning();
    setState(() => _isScanning = false);
  }

  @override
  void dispose() {
    _stopScanning();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final bleService = ref.watch(bleServiceProvider);

    return Scaffold(
      appBar: AppBar(
        title: Text(l10n.scanningTitle),
        leading: IconButton(
          icon: const Icon(Icons.close),
          onPressed: () => context.pop(),
        ),
      ),
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            children: [
              const Spacer(),

              // Scanning animation
              _buildScanningIndicator(),
              const SizedBox(height: 32),

              // Status text
              Text(
                _isScanning ? l10n.scanningBle : l10n.scanningNone,
                style: Theme.of(context).textTheme.titleMedium,
                textAlign: TextAlign.center,
              ),
              const SizedBox(height: 8),

              // Instructions
              if (!_bleAvailable && !_nfcAvailable)
                Text(
                  l10n.scanningEnableBluetooth,
                  style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                        color: SahiColors.signalWarning,
                      ),
                  textAlign: TextAlign.center,
                ),

              const Spacer(flex: 2),

              // NFC option
              if (_nfcAvailable) ...[
                const Divider(),
                const SizedBox(height: 16),
                Text(
                  l10n.scanningOr,
                  style: Theme.of(context).textTheme.bodyMedium,
                ),
                const SizedBox(height: 16),
                SizedBox(
                  width: double.infinity,
                  child: OutlinedButton.icon(
                    onPressed: () {
                      // TODO: Show NFC tap instructions
                    },
                    icon: const Icon(Icons.nfc),
                    label: Text(l10n.tapToPresent),
                  ),
                ),
              ],

              const SizedBox(height: 24),

              // Manual scan button
              if (_bleAvailable && !_isScanning)
                SizedBox(
                  width: double.infinity,
                  child: ElevatedButton(
                    onPressed: _startScanning,
                    child: Text(l10n.scanningAgain),
                  ),
                ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildScanningIndicator() {
    return SizedBox(
      width: 200,
      height: 200,
      child: Stack(
        alignment: Alignment.center,
        children: [
          // Ripple effect when scanning
          if (_isScanning) ...[
            _buildRipple(180, 0.1, 0),
            _buildRipple(140, 0.2, 0.3),
            _buildRipple(100, 0.3, 0.6),
          ],

          // Center icon
          Container(
            width: 80,
            height: 80,
            decoration: BoxDecoration(
              color: SahiColors.slate900,
              shape: BoxShape.circle,
              border: Border.all(
                color: _isScanning ? SahiColors.slate600 : SahiColors.slate700,
                width: 2,
              ),
            ),
            child: Icon(
              Icons.bluetooth_searching,
              size: 36,
              color: _isScanning ? SahiColors.slate300 : SahiColors.slate500,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildRipple(double size, double opacity, double delay) {
    return TweenAnimationBuilder<double>(
      tween: Tween(begin: 0.0, end: 1.0),
      duration: const Duration(seconds: 2),
      curve: Curves.easeOut,
      builder: (context, value, child) {
        return Container(
          width: size,
          height: size,
          decoration: BoxDecoration(
            shape: BoxShape.circle,
            border: Border.all(
              color: SahiColors.slate500.withOpacity(opacity * (1 - value)),
              width: 2,
            ),
          ),
        );
      },
    );
  }
}
