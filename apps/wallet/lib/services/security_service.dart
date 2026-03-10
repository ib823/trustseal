import 'dart:async';

import 'package:flutter/foundation.dart';

/// Security threat types detected.
enum SecurityThreat {
  /// Device is rooted (Android) or jailbroken (iOS).
  rootedDevice,

  /// App is running in an emulator.
  emulator,

  /// App binary has been tampered with.
  appTampered,

  /// Debugger is attached.
  debuggerAttached,

  /// App is running in developer mode.
  developerMode,

  /// Unofficial app store installation.
  unofficialStore,
}

/// Security service for device integrity checks.
///
/// Uses freeRASP-like detection.
///
/// IMPORTANT per spec:
/// - Warn user + log event
/// - Do NOT block operations
class SecurityService {
  static final _threatController =
      StreamController<Set<SecurityThreat>>.broadcast();

  static Stream<Set<SecurityThreat>> get threatStream =>
      _threatController.stream;

  static Set<SecurityThreat> _detectedThreats = {};
  static Set<SecurityThreat> get detectedThreats => _detectedThreats;

  /// Initialize security checks.
  ///
  /// Runs all integrity checks and populates [detectedThreats].
  static Future<void> initialize() async {
    _detectedThreats = {};

    // Check for root/jailbreak
    if (await _isRooted()) {
      _detectedThreats.add(SecurityThreat.rootedDevice);
    }

    // Check for emulator
    if (await _isEmulator()) {
      _detectedThreats.add(SecurityThreat.emulator);
    }

    // Check for debugger
    if (_isDebuggerAttached()) {
      _detectedThreats.add(SecurityThreat.debuggerAttached);
    }

    // Check for debug mode
    if (kDebugMode) {
      _detectedThreats.add(SecurityThreat.developerMode);
    }

    // Notify listeners
    if (_detectedThreats.isNotEmpty) {
      _threatController.add(_detectedThreats);
      _logThreats();
    }
  }

  /// Check if any threats were detected.
  static bool get hasThreats => _detectedThreats.isNotEmpty;

  /// Check if device is rooted/jailbroken.
  static bool get isDeviceCompromised =>
      _detectedThreats.contains(SecurityThreat.rootedDevice);

  /// Check for root/jailbreak.
  ///
  /// Note: In production, this would use freeRASP or similar.
  /// This is a simplified placeholder.
  static Future<bool> _isRooted() async {
    // In production:
    // - Android: Check for su binary, Magisk, root management apps
    // - iOS: Check for Cydia, common jailbreak paths
    //
    // For now, always return false (placeholder).
    // Real implementation would use freeRASP.
    return false;
  }

  /// Check if running in emulator.
  static Future<bool> _isEmulator() async {
    // In production:
    // - Android: Check Build properties, hardware characteristics
    // - iOS: Check for simulator environment
    //
    // Placeholder - real implementation would use freeRASP.
    return false;
  }

  /// Check if debugger is attached.
  static bool _isDebuggerAttached() {
    // Check if running in debug mode
    bool isDebugging = false;
    assert(() {
      isDebugging = true;
      return true;
    }());
    return isDebugging;
  }

  /// Log detected threats.
  static void _logThreats() {
    // In production, this would send to server audit log.
    // Per spec: Log event, but do NOT block operations.
    debugPrint('VaultPass Security: Detected threats: $_detectedThreats');
  }

  /// Get human-readable threat descriptions.
  static List<String> getThreatDescriptions() {
    return _detectedThreats.map((threat) {
      return switch (threat) {
        SecurityThreat.rootedDevice =>
          'This device may be rooted or jailbroken. Your credentials may be at risk.',
        SecurityThreat.emulator =>
          'This app is running in an emulator, which is not recommended for production use.',
        SecurityThreat.appTampered =>
          'The app binary may have been modified.',
        SecurityThreat.debuggerAttached =>
          'A debugger is attached to this app.',
        SecurityThreat.developerMode => 'The app is running in developer mode.',
        SecurityThreat.unofficialStore =>
          'This app was not installed from an official app store.',
      };
    }).toList();
  }

  /// Dispose resources.
  static void dispose() {
    _threatController.close();
  }
}
