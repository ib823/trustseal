import 'package:flutter/material.dart';

import 'sahi_colors.dart';
import 'sahi_typography.dart';

/// Sahi dark theme for VaultPass wallet.
///
/// Design principles:
/// - Dark mode only (slate-950 background)
/// - Monochrome-first (95% slate palette)
/// - Signal colors ONLY for semantic meaning
/// - Border-only elevation (no shadows except modals)
/// - Plus Jakarta Sans primary, JetBrains Mono for codes/hashes
class SahiTheme {
  SahiTheme._();

  static ThemeData get dark => ThemeData(
        useMaterial3: true,
        brightness: Brightness.dark,

        // Colors
        colorScheme: const ColorScheme.dark(
          primary: SahiColors.slate100,
          onPrimary: SahiColors.slate950,
          secondary: SahiColors.slate400,
          onSecondary: SahiColors.slate950,
          surface: SahiColors.slate900,
          onSurface: SahiColors.slate100,
          error: SahiColors.signalError,
          onError: SahiColors.slate950,
          outline: SahiColors.slate700,
        ),

        scaffoldBackgroundColor: SahiColors.slate950,
        canvasColor: SahiColors.slate950,

        // Typography
        fontFamily: SahiTypography.fontFamilyPrimary,
        textTheme: SahiTypography.textTheme,

        // AppBar
        appBarTheme: const AppBarTheme(
          backgroundColor: SahiColors.slate950,
          foregroundColor: SahiColors.slate100,
          elevation: 0,
          centerTitle: true,
          titleTextStyle: TextStyle(
            fontFamily: SahiTypography.fontFamilyPrimary,
            fontSize: 18,
            fontWeight: FontWeight.w600,
            color: SahiColors.slate100,
          ),
        ),

        // Cards
        cardTheme: CardTheme(
          color: SahiColors.slate900,
          elevation: 0,
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(12),
            side: const BorderSide(color: SahiColors.slate800),
          ),
        ),

        // Buttons
        elevatedButtonTheme: ElevatedButtonThemeData(
          style: ElevatedButton.styleFrom(
            backgroundColor: SahiColors.slate100,
            foregroundColor: SahiColors.slate950,
            elevation: 0,
            padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 16),
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(12),
            ),
            textStyle: const TextStyle(
              fontFamily: SahiTypography.fontFamilyPrimary,
              fontSize: 16,
              fontWeight: FontWeight.w600,
            ),
          ),
        ),

        outlinedButtonTheme: OutlinedButtonThemeData(
          style: OutlinedButton.styleFrom(
            foregroundColor: SahiColors.slate100,
            padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 16),
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(12),
            ),
            side: const BorderSide(color: SahiColors.slate700),
            textStyle: const TextStyle(
              fontFamily: SahiTypography.fontFamilyPrimary,
              fontSize: 16,
              fontWeight: FontWeight.w600,
            ),
          ),
        ),

        textButtonTheme: TextButtonThemeData(
          style: TextButton.styleFrom(
            foregroundColor: SahiColors.slate400,
            textStyle: const TextStyle(
              fontFamily: SahiTypography.fontFamilyPrimary,
              fontSize: 14,
              fontWeight: FontWeight.w500,
            ),
          ),
        ),

        // Input fields
        inputDecorationTheme: InputDecorationTheme(
          filled: true,
          fillColor: SahiColors.slate900,
          border: OutlineInputBorder(
            borderRadius: BorderRadius.circular(12),
            borderSide: const BorderSide(color: SahiColors.slate700),
          ),
          enabledBorder: OutlineInputBorder(
            borderRadius: BorderRadius.circular(12),
            borderSide: const BorderSide(color: SahiColors.slate700),
          ),
          focusedBorder: OutlineInputBorder(
            borderRadius: BorderRadius.circular(12),
            borderSide: const BorderSide(color: SahiColors.slate500, width: 2),
          ),
          errorBorder: OutlineInputBorder(
            borderRadius: BorderRadius.circular(12),
            borderSide: const BorderSide(color: SahiColors.signalError),
          ),
          contentPadding:
              const EdgeInsets.symmetric(horizontal: 16, vertical: 16),
          hintStyle: const TextStyle(color: SahiColors.slate500),
          labelStyle: const TextStyle(color: SahiColors.slate400),
        ),

        // Bottom navigation
        bottomNavigationBarTheme: const BottomNavigationBarThemeData(
          backgroundColor: SahiColors.slate900,
          selectedItemColor: SahiColors.slate100,
          unselectedItemColor: SahiColors.slate500,
          type: BottomNavigationBarType.fixed,
          elevation: 0,
        ),

        // Dividers
        dividerTheme: const DividerThemeData(
          color: SahiColors.slate800,
          thickness: 1,
        ),

        // Dialogs
        dialogTheme: DialogTheme(
          backgroundColor: SahiColors.slate900,
          elevation: 8,
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(16),
          ),
        ),

        // Bottom sheets
        bottomSheetTheme: const BottomSheetThemeData(
          backgroundColor: SahiColors.slate900,
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.vertical(top: Radius.circular(20)),
          ),
        ),

        // Snackbars
        snackBarTheme: SnackBarThemeData(
          backgroundColor: SahiColors.slate800,
          contentTextStyle: const TextStyle(color: SahiColors.slate100),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(8),
          ),
          behavior: SnackBarBehavior.floating,
        ),

        // Icons
        iconTheme: const IconThemeData(
          color: SahiColors.slate400,
          size: 24,
        ),

        // Lists
        listTileTheme: const ListTileThemeData(
          iconColor: SahiColors.slate400,
          textColor: SahiColors.slate100,
          contentPadding: EdgeInsets.symmetric(horizontal: 16, vertical: 8),
        ),

        // Switches
        switchTheme: SwitchThemeData(
          thumbColor: WidgetStateProperty.resolveWith((states) {
            if (states.contains(WidgetState.selected)) {
              return SahiColors.signalSuccess;
            }
            return SahiColors.slate500;
          }),
          trackColor: WidgetStateProperty.resolveWith((states) {
            if (states.contains(WidgetState.selected)) {
              return SahiColors.signalSuccess.withOpacity(0.3);
            }
            return SahiColors.slate700;
          }),
        ),
      );
}
