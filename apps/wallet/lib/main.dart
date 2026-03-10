import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'app/app.dart';
import 'services/security_service.dart';

/// VaultPass Wallet entry point.
///
/// Initialization sequence:
/// 1. Ensure Flutter bindings
/// 2. Lock to portrait orientation
/// 3. Initialize security checks (root/jailbreak detection)
/// 4. Pre-warm FFI bridge during splash
/// 5. Launch app
void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  // Lock to portrait mode for consistent UX
  await SystemChrome.setPreferredOrientations([
    DeviceOrientation.portraitUp,
    DeviceOrientation.portraitDown,
  ]);

  // Set system UI overlay style for dark theme
  SystemChrome.setSystemUIOverlayStyle(
    const SystemUiOverlayStyle(
      statusBarColor: Colors.transparent,
      statusBarIconBrightness: Brightness.light,
      systemNavigationBarColor: Color(0xFF020617), // slate-950
      systemNavigationBarIconBrightness: Brightness.light,
    ),
  );

  // Initialize security service (freeRASP checks)
  // Note: Warns user but does NOT block operations per spec
  await SecurityService.initialize();

  runApp(
    const ProviderScope(
      child: VaultPassApp(),
    ),
  );
}
