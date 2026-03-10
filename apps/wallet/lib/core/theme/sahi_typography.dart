import 'package:flutter/material.dart';

import 'sahi_colors.dart';

/// Sahi typography system.
///
/// Fonts:
/// - Plus Jakarta Sans: Primary UI text
/// - JetBrains Mono: IDs, hashes, codes, technical data
abstract class SahiTypography {
  static const String fontFamilyPrimary = 'PlusJakartaSans';
  static const String fontFamilyMono = 'JetBrainsMono';

  static const TextTheme textTheme = TextTheme(
    // Display
    displayLarge: TextStyle(
      fontFamily: fontFamilyPrimary,
      fontSize: 32,
      fontWeight: FontWeight.w700,
      color: SahiColors.slate100,
      letterSpacing: -0.5,
    ),
    displayMedium: TextStyle(
      fontFamily: fontFamilyPrimary,
      fontSize: 28,
      fontWeight: FontWeight.w700,
      color: SahiColors.slate100,
      letterSpacing: -0.5,
    ),
    displaySmall: TextStyle(
      fontFamily: fontFamilyPrimary,
      fontSize: 24,
      fontWeight: FontWeight.w600,
      color: SahiColors.slate100,
    ),

    // Headlines
    headlineLarge: TextStyle(
      fontFamily: fontFamilyPrimary,
      fontSize: 22,
      fontWeight: FontWeight.w600,
      color: SahiColors.slate100,
    ),
    headlineMedium: TextStyle(
      fontFamily: fontFamilyPrimary,
      fontSize: 20,
      fontWeight: FontWeight.w600,
      color: SahiColors.slate100,
    ),
    headlineSmall: TextStyle(
      fontFamily: fontFamilyPrimary,
      fontSize: 18,
      fontWeight: FontWeight.w600,
      color: SahiColors.slate100,
    ),

    // Titles
    titleLarge: TextStyle(
      fontFamily: fontFamilyPrimary,
      fontSize: 18,
      fontWeight: FontWeight.w600,
      color: SahiColors.slate100,
    ),
    titleMedium: TextStyle(
      fontFamily: fontFamilyPrimary,
      fontSize: 16,
      fontWeight: FontWeight.w600,
      color: SahiColors.slate100,
    ),
    titleSmall: TextStyle(
      fontFamily: fontFamilyPrimary,
      fontSize: 14,
      fontWeight: FontWeight.w600,
      color: SahiColors.slate100,
    ),

    // Body
    bodyLarge: TextStyle(
      fontFamily: fontFamilyPrimary,
      fontSize: 16,
      fontWeight: FontWeight.w400,
      color: SahiColors.slate200,
    ),
    bodyMedium: TextStyle(
      fontFamily: fontFamilyPrimary,
      fontSize: 14,
      fontWeight: FontWeight.w400,
      color: SahiColors.slate300,
    ),
    bodySmall: TextStyle(
      fontFamily: fontFamilyPrimary,
      fontSize: 12,
      fontWeight: FontWeight.w400,
      color: SahiColors.slate400,
    ),

    // Labels
    labelLarge: TextStyle(
      fontFamily: fontFamilyPrimary,
      fontSize: 14,
      fontWeight: FontWeight.w500,
      color: SahiColors.slate200,
    ),
    labelMedium: TextStyle(
      fontFamily: fontFamilyPrimary,
      fontSize: 12,
      fontWeight: FontWeight.w500,
      color: SahiColors.slate300,
    ),
    labelSmall: TextStyle(
      fontFamily: fontFamilyPrimary,
      fontSize: 11,
      fontWeight: FontWeight.w500,
      color: SahiColors.slate400,
      letterSpacing: 0.5,
    ),
  );

  /// Monospace style for IDs, hashes, codes.
  static const TextStyle mono = TextStyle(
    fontFamily: fontFamilyMono,
    fontSize: 13,
    fontWeight: FontWeight.w400,
    color: SahiColors.slate300,
    letterSpacing: 0.5,
  );

  /// Monospace style for small codes (e.g., credential IDs).
  static const TextStyle monoSmall = TextStyle(
    fontFamily: fontFamilyMono,
    fontSize: 11,
    fontWeight: FontWeight.w400,
    color: SahiColors.slate400,
    letterSpacing: 0.5,
  );
}
