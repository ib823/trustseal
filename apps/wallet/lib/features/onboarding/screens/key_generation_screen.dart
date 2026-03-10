import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../app/router.dart';
import '../../../core/i18n/app_localizations.dart';
import '../../../core/theme/sahi_colors.dart';
import '../../../services/auth_service.dart';
import '../../../services/keystore_service.dart';

/// Key generation state.
enum KeyGenState { idle, generating, settingBiometrics, success, error }

/// Key generation screen.
///
/// Generates a hardware-bound key in Android Keystore / iOS Secure Enclave.
class KeyGenerationScreen extends ConsumerStatefulWidget {
  const KeyGenerationScreen({super.key});

  @override
  ConsumerState<KeyGenerationScreen> createState() =>
      _KeyGenerationScreenState();
}

class _KeyGenerationScreenState extends ConsumerState<KeyGenerationScreen> {
  KeyGenState _state = KeyGenState.idle;
  String? _errorMessage;

  @override
  void initState() {
    super.initState();
    _startKeyGeneration();
  }

  Future<void> _startKeyGeneration() async {
    setState(() {
      _state = KeyGenState.generating;
      _errorMessage = null;
    });

    try {
      // Check hardware capability
      final hasStrongBox = await KeystoreService.isStrongBoxAvailable();
      final hasHardwareBacked =
          await KeystoreService.isHardwareBackedAvailable();

      if (!hasHardwareBacked) {
        // Warn but don't block - per spec
        debugPrint('Warning: No hardware-backed keystore available');
      }

      // Generate key pair
      final publicKey = await KeystoreService.generateKeyPair(
        requireBiometric: true,
      );

      // TODO: Register public key with server
      debugPrint('Generated public key: ${publicKey.length} bytes');

      setState(() => _state = KeyGenState.settingBiometrics);

      // Set up biometrics
      final authService = ref.read(authServiceProvider);
      final authenticated = await authService.authenticateWithBiometrics(
        reason: 'Confirm biometric setup for credential protection',
      );

      if (!authenticated) {
        setState(() {
          _state = KeyGenState.error;
          _errorMessage = 'Biometric setup is required to continue';
        });
        return;
      }

      // Mark setup complete
      authService.markDeviceKeyGenerated();

      setState(() => _state = KeyGenState.success);

      // Navigate to home after delay
      await Future.delayed(const Duration(seconds: 1));
      if (mounted) {
        context.go(Routes.credentials);
      }
    } on KeystoreException catch (e) {
      setState(() {
        _state = KeyGenState.error;
        _errorMessage = e.message;
      });
    } catch (e) {
      setState(() {
        _state = KeyGenState.error;
        _errorMessage = 'An unexpected error occurred';
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);

    return Scaffold(
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(32),
          child: Column(
            children: [
              const Spacer(),

              // Icon
              _buildIcon(),
              const SizedBox(height: 32),

              // Title
              Text(
                _state == KeyGenState.settingBiometrics
                    ? l10n.setupBiometrics
                    : l10n.keyGenerationTitle,
                style: Theme.of(context).textTheme.displaySmall,
                textAlign: TextAlign.center,
              ),
              const SizedBox(height: 16),

              // Status text
              Text(
                _getStatusText(l10n),
                style: Theme.of(context).textTheme.bodyLarge?.copyWith(
                      color: _getStatusColor(),
                    ),
                textAlign: TextAlign.center,
              ),

              const Spacer(flex: 2),

              // Retry button on error
              if (_state == KeyGenState.error) ...[
                if (_errorMessage != null) ...[
                  Container(
                    padding: const EdgeInsets.all(12),
                    decoration: BoxDecoration(
                      color: SahiColors.signalError.withOpacity(0.1),
                      borderRadius: BorderRadius.circular(8),
                    ),
                    child: Text(
                      _errorMessage!,
                      style: const TextStyle(
                        color: SahiColors.signalError,
                        fontSize: 14,
                      ),
                      textAlign: TextAlign.center,
                    ),
                  ),
                  const SizedBox(height: 24),
                ],
                SizedBox(
                  width: double.infinity,
                  child: ElevatedButton(
                    onPressed: _startKeyGeneration,
                    child: Text(l10n.retry),
                  ),
                ),
              ],
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildIcon() {
    final (icon, color, showProgress) = switch (_state) {
      KeyGenState.idle => (Icons.key, SahiColors.slate400, false),
      KeyGenState.generating => (Icons.key, SahiColors.slate400, true),
      KeyGenState.settingBiometrics =>
        (Icons.fingerprint, SahiColors.slate400, false),
      KeyGenState.success =>
        (Icons.check_circle, SahiColors.signalSuccess, false),
      KeyGenState.error => (Icons.error_outline, SahiColors.signalError, false),
    };

    return Stack(
      alignment: Alignment.center,
      children: [
        if (showProgress)
          const SizedBox(
            width: 100,
            height: 100,
            child: CircularProgressIndicator(
              strokeWidth: 3,
              color: SahiColors.slate700,
            ),
          ),
        Icon(icon, size: 64, color: color),
      ],
    );
  }

  String _getStatusText(AppLocalizations l10n) {
    return switch (_state) {
      KeyGenState.idle => l10n.keyGenerationSubtitle,
      KeyGenState.generating => l10n.keyGenerationInProgress,
      KeyGenState.settingBiometrics => l10n.biometricsRequired,
      KeyGenState.success => l10n.keyGenerationSuccess,
      KeyGenState.error => l10n.error,
    };
  }

  Color _getStatusColor() {
    return switch (_state) {
      KeyGenState.success => SahiColors.signalSuccess,
      KeyGenState.error => SahiColors.signalError,
      _ => SahiColors.slate300,
    };
  }
}
