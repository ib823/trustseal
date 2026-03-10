import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../core/i18n/app_localizations.dart';
import '../../../core/models/credential.dart';
import '../../../core/storage/secure_credential_storage.dart';
import '../../../core/theme/sahi_colors.dart';
import '../../../services/auth_service.dart';
import '../../../services/ble_service.dart';

/// Presentation flow state.
enum PresentationPhase {
  detecting,
  authenticating,
  connecting,
  sending,
  granted,
  denied,
  error,
}

/// Provider for presentation state.
final presentationPhaseProvider =
    StateProvider<PresentationPhase>((ref) => PresentationPhase.detecting);

/// Credential presentation screen.
///
/// Flow per spec:
/// detect -> biometric -> present -> result (auto-dismiss 3s)
class PresentationScreen extends ConsumerStatefulWidget {
  final String credentialId;

  const PresentationScreen({
    super.key,
    required this.credentialId,
  });

  @override
  ConsumerState<PresentationScreen> createState() => _PresentationScreenState();
}

class _PresentationScreenState extends ConsumerState<PresentationScreen> {
  Credential? _credential;
  Timer? _autoDismissTimer;

  @override
  void initState() {
    super.initState();
    _loadCredential();
    _startPresentation();
  }

  @override
  void dispose() {
    _autoDismissTimer?.cancel();
    super.dispose();
  }

  Future<void> _loadCredential() async {
    final storage = SecureCredentialStorage();
    final credential = await storage.getCredential(widget.credentialId);
    if (mounted) {
      setState(() => _credential = credential);
    }
  }

  Future<void> _startPresentation() async {
    final bleService = ref.read(bleServiceProvider);
    final authService = ref.read(authServiceProvider);

    // Phase 1: Detect verifier
    ref.read(presentationPhaseProvider.notifier).state =
        PresentationPhase.detecting;

    await bleService.startScanning();

    // Wait for verifier detection
    await for (final state in bleService.stateStream) {
      if (state == BleState.connecting) {
        // Phase 2: Authenticate
        ref.read(presentationPhaseProvider.notifier).state =
            PresentationPhase.authenticating;

        final authenticated = await authService.authenticateWithBiometrics(
          reason: 'Authenticate to present credential',
        );

        if (!authenticated) {
          ref.read(presentationPhaseProvider.notifier).state =
              PresentationPhase.error;
          return;
        }

        // Phase 3: Connect and send
        ref.read(presentationPhaseProvider.notifier).state =
            PresentationPhase.connecting;
        break;
      }

      if (state == BleState.error) {
        ref.read(presentationPhaseProvider.notifier).state =
            PresentationPhase.error;
        return;
      }
    }

    // Phase 4: Present credential
    ref.read(presentationPhaseProvider.notifier).state =
        PresentationPhase.sending;

    // TODO: Create actual presentation with crypto service
    // For now, simulate with empty bytes
    final result =
        await bleService.present(Uint8List(0));

    // Phase 5: Show result
    final phase = switch (result) {
      PresentationResult.granted => PresentationPhase.granted,
      PresentationResult.denied => PresentationPhase.denied,
      _ => PresentationPhase.error,
    };

    ref.read(presentationPhaseProvider.notifier).state = phase;

    // Auto-dismiss after 3 seconds per spec
    _autoDismissTimer = Timer(const Duration(seconds: 3), () {
      if (mounted) {
        context.pop();
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final phase = ref.watch(presentationPhaseProvider);

    return Scaffold(
      backgroundColor: SahiColors.slate950,
      body: SafeArea(
        child: Center(
          child: Padding(
            padding: const EdgeInsets.all(32),
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                _buildIcon(phase),
                const SizedBox(height: 32),
                _buildTitle(context, l10n, phase),
                const SizedBox(height: 16),
                _buildSubtitle(context, l10n, phase),
                const SizedBox(height: 48),
                if (phase == PresentationPhase.error)
                  OutlinedButton(
                    onPressed: () => context.pop(),
                    child: Text(l10n.close),
                  ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildIcon(PresentationPhase phase) {
    final (icon, color, showProgress) = switch (phase) {
      PresentationPhase.detecting => (Icons.wifi_find, SahiColors.slate400, true),
      PresentationPhase.authenticating =>
        (Icons.fingerprint, SahiColors.slate400, false),
      PresentationPhase.connecting =>
        (Icons.bluetooth_connected, SahiColors.slate400, true),
      PresentationPhase.sending => (Icons.send, SahiColors.slate400, true),
      PresentationPhase.granted =>
        (Icons.check_circle, SahiColors.signalSuccess, false),
      PresentationPhase.denied =>
        (Icons.cancel, SahiColors.signalError, false),
      PresentationPhase.error =>
        (Icons.error_outline, SahiColors.signalError, false),
    };

    return Stack(
      alignment: Alignment.center,
      children: [
        if (showProgress)
          SizedBox(
            width: 120,
            height: 120,
            child: CircularProgressIndicator(
              strokeWidth: 3,
              color: SahiColors.slate700,
            ),
          ),
        Icon(icon, size: 64, color: color),
      ],
    );
  }

  Widget _buildTitle(
    BuildContext context,
    AppLocalizations l10n,
    PresentationPhase phase,
  ) {
    final text = switch (phase) {
      PresentationPhase.detecting => l10n.presentationDetecting,
      PresentationPhase.authenticating => l10n.presentationAuthenticating,
      PresentationPhase.connecting => l10n.presentationConnecting,
      PresentationPhase.sending => l10n.presentationSending,
      PresentationPhase.granted => l10n.presentationGranted,
      PresentationPhase.denied => l10n.presentationDenied,
      PresentationPhase.error => l10n.presentationError,
    };

    final color = switch (phase) {
      PresentationPhase.granted => SahiColors.signalSuccess,
      PresentationPhase.denied => SahiColors.signalError,
      PresentationPhase.error => SahiColors.signalError,
      _ => SahiColors.slate100,
    };

    return Text(
      text,
      style: Theme.of(context).textTheme.headlineSmall?.copyWith(color: color),
      textAlign: TextAlign.center,
    );
  }

  Widget _buildSubtitle(
    BuildContext context,
    AppLocalizations l10n,
    PresentationPhase phase,
  ) {
    if (_credential == null) return const SizedBox.shrink();

    final text = switch (phase) {
      PresentationPhase.granted => _credential!.propertyName,
      PresentationPhase.denied => l10n.accessNotAuthorized,
      PresentationPhase.error => l10n.retry,
      _ => _credential!.propertyName,
    };

    return Text(
      text,
      style: Theme.of(context).textTheme.bodyLarge,
      textAlign: TextAlign.center,
    );
  }
}

// Re-export for import
typedef Uint8List = List<int>;
