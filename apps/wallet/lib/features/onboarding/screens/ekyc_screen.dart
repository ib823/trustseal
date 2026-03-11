import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../app/router.dart';
import '../../../core/i18n/app_localizations.dart';
import '../../../core/theme/sahi_colors.dart';
import '../../../services/ekyc_service.dart';

/// eKYC verification state.
enum EkycState { idle, inProgress, success, failed }

/// eKYC service provider.
final ekycServiceProvider = Provider<EkycService>((ref) => EkycService());

/// eKYC screen.
///
/// Completes identity verification via MyDigital ID using OAuth 2.0 PKCE.
class EkycScreen extends ConsumerStatefulWidget {
  const EkycScreen({super.key});

  @override
  ConsumerState<EkycScreen> createState() => _EkycScreenState();
}

class _EkycScreenState extends ConsumerState<EkycScreen> {
  EkycState _state = EkycState.idle;
  String? _errorMessage;

  Future<void> _startVerification() async {
    setState(() {
      _state = EkycState.inProgress;
      _errorMessage = null;
    });

    try {
      final ekycService = ref.read(ekycServiceProvider);

      // Step 1: Initiate verification with backend
      // In production, tenant_id would come from app state
      final result = await ekycService.initiateVerification(
        tenantId: 'TNT_default',
      );

      // Step 2: Open MyDigital ID authorization URL in browser
      final launched = await ekycService.openAuthorizationUrl(
        result.authorizationUrl,
      );

      if (!launched) {
        throw const EkycException(
          'Could not open verification page',
          code: 'SAHI_5001',
        );
      }

      // Note: The actual callback will be handled via deep link
      // when MyDigital ID redirects back to the app.
      // For now, we simulate success after a delay for development.
      // In production, the app would listen for the callback URI.

      // Development simulation - remove in production
      await Future.delayed(const Duration(seconds: 3));

      setState(() => _state = EkycState.success);

      // Auto-proceed after short delay
      await Future.delayed(const Duration(milliseconds: 500));

      if (mounted) {
        context.go(Routes.keyGeneration);
      }
    } on EkycException catch (e) {
      setState(() {
        _state = EkycState.failed;
        _errorMessage = e.message;
      });
    } catch (e) {
      setState(() {
        _state = EkycState.failed;
        _errorMessage = e.toString();
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);

    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: () => context.go(Routes.registration),
        ),
      ),
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
                l10n.ekycTitle,
                style: Theme.of(context).textTheme.displaySmall,
                textAlign: TextAlign.center,
              ),
              const SizedBox(height: 16),

              // Subtitle/Status
              Text(
                _getStatusText(l10n),
                style: Theme.of(context).textTheme.bodyLarge?.copyWith(
                      color: _getStatusColor(),
                    ),
                textAlign: TextAlign.center,
              ),

              // Error message
              if (_errorMessage != null) ...[
                const SizedBox(height: 8),
                Text(
                  _errorMessage!,
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                        color: SahiColors.signalError,
                      ),
                  textAlign: TextAlign.center,
                ),
              ],

              const Spacer(flex: 2),

              // Action button
              if (_state == EkycState.idle || _state == EkycState.failed)
                SizedBox(
                  width: double.infinity,
                  child: ElevatedButton(
                    onPressed: _startVerification,
                    child: Text(l10n.ekycStart),
                  ),
                ),

              if (_state == EkycState.failed) ...[
                const SizedBox(height: 16),
                TextButton(
                  onPressed: () => context.go(Routes.keyGeneration),
                  child: Text(l10n.ekycSkip),
                ),
              ],
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildIcon() {
    final (icon, color) = switch (_state) {
      EkycState.idle => (Icons.verified_user_outlined, SahiColors.slate400),
      EkycState.inProgress => (Icons.hourglass_top, SahiColors.slate400),
      EkycState.success => (Icons.check_circle, SahiColors.signalSuccess),
      EkycState.failed => (Icons.error_outline, SahiColors.signalError),
    };

    return Stack(
      alignment: Alignment.center,
      children: [
        if (_state == EkycState.inProgress)
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
      EkycState.idle => l10n.ekycSubtitle,
      EkycState.inProgress => l10n.ekycInProgress,
      EkycState.success => l10n.ekycSuccess,
      EkycState.failed => l10n.ekycFailed,
    };
  }

  Color _getStatusColor() {
    return switch (_state) {
      EkycState.success => SahiColors.signalSuccess,
      EkycState.failed => SahiColors.signalError,
      _ => SahiColors.slate300,
    };
  }
}
