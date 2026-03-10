import 'package:flutter/material.dart';

/// Sahi color palette.
///
/// 95% of the UI uses the slate monochrome palette.
/// Signal colors appear ONLY when carrying semantic meaning.
abstract class SahiColors {
  // Slate palette (monochrome)
  static const Color slate50 = Color(0xFFF8FAFC);
  static const Color slate100 = Color(0xFFF1F5F9);
  static const Color slate200 = Color(0xFFE2E8F0);
  static const Color slate300 = Color(0xFFCBD5E1);
  static const Color slate400 = Color(0xFF94A3B8);
  static const Color slate500 = Color(0xFF64748B);
  static const Color slate600 = Color(0xFF475569);
  static const Color slate700 = Color(0xFF334155);
  static const Color slate800 = Color(0xFF1E293B);
  static const Color slate900 = Color(0xFF0F172A);
  static const Color slate950 = Color(0xFF020617);

  // Signal colors - ONLY for semantic meaning
  /// Green-500: granted, valid, active
  static const Color signalSuccess = Color(0xFF10B981);

  /// Red-500: denied, invalid, revoked
  static const Color signalError = Color(0xFFEF4444);

  /// Amber-500: expiring, degraded, attention
  static const Color signalWarning = Color(0xFFF59E0B);

  // Credential card gradients (subtle)
  static const LinearGradient residentBadgeGradient = LinearGradient(
    begin: Alignment.topLeft,
    end: Alignment.bottomRight,
    colors: [
      Color(0xFF1E293B), // slate-800
      Color(0xFF0F172A), // slate-900
    ],
  );

  static const LinearGradient visitorPassGradient = LinearGradient(
    begin: Alignment.topLeft,
    end: Alignment.bottomRight,
    colors: [
      Color(0xFF1E3A5F), // blue-tinted slate
      Color(0xFF0F172A), // slate-900
    ],
  );

  static const LinearGradient contractorBadgeGradient = LinearGradient(
    begin: Alignment.topLeft,
    end: Alignment.bottomRight,
    colors: [
      Color(0xFF2D2A1E), // amber-tinted slate
      Color(0xFF0F172A), // slate-900
    ],
  );

  static const LinearGradient emergencyAccessGradient = LinearGradient(
    begin: Alignment.topLeft,
    end: Alignment.bottomRight,
    colors: [
      Color(0xFF3B1E1E), // red-tinted slate
      Color(0xFF0F172A), // slate-900
    ],
  );
}
