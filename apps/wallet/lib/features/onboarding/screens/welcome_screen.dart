import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../../../app/router.dart';
import '../../../core/i18n/app_localizations.dart';
import '../../../core/theme/sahi_colors.dart';

/// Welcome screen - first screen of onboarding.
///
/// One primary action per screen per spec.
class WelcomeScreen extends StatelessWidget {
  const WelcomeScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);

    return Scaffold(
      backgroundColor: SahiColors.slate950,
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(32),
          child: Column(
            children: [
              const Spacer(flex: 2),

              // Logo/Icon
              Container(
                width: 120,
                height: 120,
                decoration: BoxDecoration(
                  color: SahiColors.slate900,
                  borderRadius: BorderRadius.circular(24),
                  border: Border.all(color: SahiColors.slate800),
                ),
                child: const Icon(
                  Icons.security,
                  size: 56,
                  color: SahiColors.slate300,
                ),
              ),
              const SizedBox(height: 48),

              // Title
              Text(
                l10n.welcomeTitle,
                style: Theme.of(context).textTheme.displaySmall,
                textAlign: TextAlign.center,
              ),
              const SizedBox(height: 16),

              // Subtitle
              Text(
                l10n.welcomeSubtitle,
                style: Theme.of(context).textTheme.bodyLarge,
                textAlign: TextAlign.center,
              ),

              const Spacer(flex: 3),

              // Primary action button
              SizedBox(
                width: double.infinity,
                child: ElevatedButton(
                  onPressed: () => context.go(Routes.registration),
                  child: Text(l10n.getStarted),
                ),
              ),
              const SizedBox(height: 16),

              // Language toggle
              TextButton(
                onPressed: () {
                  // TODO: Toggle language
                },
                child: const Text('English / Bahasa Melayu'),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
