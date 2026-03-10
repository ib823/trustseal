import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../features/credentials/screens/credentials_screen.dart';
import '../features/credentials/screens/credential_detail_screen.dart';
import '../features/credentials/screens/presentation_screen.dart';
import '../features/onboarding/screens/welcome_screen.dart';
import '../features/onboarding/screens/registration_screen.dart';
import '../features/onboarding/screens/ekyc_screen.dart';
import '../features/onboarding/screens/key_generation_screen.dart';
import '../features/scanning/screens/scanning_screen.dart';
import '../features/settings/screens/settings_screen.dart';
import '../features/settings/screens/security_settings_screen.dart';
import '../services/auth_service.dart';

/// Route paths as constants for type safety.
abstract class Routes {
  // Onboarding
  static const welcome = '/welcome';
  static const registration = '/registration';
  static const ekyc = '/ekyc';
  static const keyGeneration = '/key-generation';

  // Main
  static const credentials = '/';
  static const credentialDetail = '/credential/:id';
  static const presentation = '/presentation/:id';

  // Scanning
  static const scanning = '/scanning';

  // Settings
  static const settings = '/settings';
  static const securitySettings = '/settings/security';
}

/// GoRouter provider for declarative routing.
final routerProvider = Provider<GoRouter>((ref) {
  final authState = ref.watch(authStateProvider);

  return GoRouter(
    initialLocation: Routes.credentials,
    debugLogDiagnostics: true,

    // Redirect unauthenticated users to onboarding
    redirect: (context, state) {
      final isOnboarding = state.matchedLocation.startsWith('/welcome') ||
          state.matchedLocation.startsWith('/registration') ||
          state.matchedLocation.startsWith('/ekyc') ||
          state.matchedLocation.startsWith('/key-generation');

      if (!authState.isAuthenticated && !isOnboarding) {
        return Routes.welcome;
      }

      if (authState.isAuthenticated && isOnboarding) {
        return Routes.credentials;
      }

      return null;
    },

    routes: [
      // Onboarding flow
      GoRoute(
        path: Routes.welcome,
        builder: (context, state) => const WelcomeScreen(),
      ),
      GoRoute(
        path: Routes.registration,
        builder: (context, state) => const RegistrationScreen(),
      ),
      GoRoute(
        path: Routes.ekyc,
        builder: (context, state) => const EkycScreen(),
      ),
      GoRoute(
        path: Routes.keyGeneration,
        builder: (context, state) => const KeyGenerationScreen(),
      ),

      // Main credential list (home)
      GoRoute(
        path: Routes.credentials,
        builder: (context, state) => const CredentialsScreen(),
      ),

      // Credential detail
      GoRoute(
        path: Routes.credentialDetail,
        builder: (context, state) {
          final id = state.pathParameters['id']!;
          return CredentialDetailScreen(credentialId: id);
        },
      ),

      // Presentation flow
      GoRoute(
        path: Routes.presentation,
        builder: (context, state) {
          final id = state.pathParameters['id']!;
          return PresentationScreen(credentialId: id);
        },
      ),

      // Scanning
      GoRoute(
        path: Routes.scanning,
        builder: (context, state) => const ScanningScreen(),
      ),

      // Settings
      GoRoute(
        path: Routes.settings,
        builder: (context, state) => const SettingsScreen(),
        routes: [
          GoRoute(
            path: 'security',
            builder: (context, state) => const SecuritySettingsScreen(),
          ),
        ],
      ),
    ],

    errorBuilder: (context, state) => Scaffold(
      body: Center(
        child: Text('Route not found: ${state.matchedLocation}'),
      ),
    ),
  );
});
