import 'package:flutter/material.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../core/theme/sahi_theme.dart';
import '../core/i18n/app_localizations.dart';
import 'router.dart';

/// Root application widget.
///
/// Configures:
/// - Dark theme only (slate-950 background)
/// - GoRouter for declarative navigation
/// - Localization (EN/MS)
/// - Riverpod for state management
class VaultPassApp extends ConsumerWidget {
  const VaultPassApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final router = ref.watch(routerProvider);

    return MaterialApp.router(
      title: 'VaultPass',
      debugShowCheckedModeBanner: false,

      // Theme: Dark mode only per spec
      theme: SahiTheme.dark,
      darkTheme: SahiTheme.dark,
      themeMode: ThemeMode.dark,

      // Routing
      routerConfig: router,

      // Localization
      localizationsDelegates: const [
        AppLocalizations.delegate,
        GlobalMaterialLocalizations.delegate,
        GlobalWidgetsLocalizations.delegate,
        GlobalCupertinoLocalizations.delegate,
      ],
      supportedLocales: AppLocalizations.supportedLocales,
    );
  }
}
