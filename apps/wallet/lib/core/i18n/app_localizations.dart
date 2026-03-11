import 'package:flutter/material.dart';

/// VaultPass localization support for EN/MS.
///
/// Language code is `ms` (ISO 639-1), never `bm`.
/// Both EN and MS keys must be present for every string.
class AppLocalizations {
  final Locale locale;

  AppLocalizations(this.locale);

  static AppLocalizations of(BuildContext context) {
    return Localizations.of<AppLocalizations>(context, AppLocalizations)!;
  }

  static const LocalizationsDelegate<AppLocalizations> delegate =
      _AppLocalizationsDelegate();

  static const List<Locale> supportedLocales = [
    Locale('en'),
    Locale('ms'),
  ];

  static final Map<String, Map<String, String>> _localizedValues = {
    'en': {
      // Common
      'app_name': 'VaultPass',
      'ok': 'OK',
      'cancel': 'Cancel',
      'continue': 'Continue',
      'done': 'Done',
      'retry': 'Retry',
      'close': 'Close',
      'save': 'Save',
      'delete': 'Delete',
      'edit': 'Edit',
      'loading': 'Loading...',
      'error': 'Error',
      'success': 'Success',

      // Onboarding
      'welcome_title': 'Welcome to VaultPass',
      'welcome_subtitle':
          'Privacy-preserving access control for your property.',
      'get_started': 'Get Started',
      'registration_title': 'Create Account',
      'registration_subtitle':
          'Register with your property administrator to get started.',
      'ekyc_title': 'Verify Identity',
      'ekyc_subtitle':
          'Complete identity verification via MyDigital ID to activate your credentials.',
      'ekyc_start': 'Start Verification',
      'ekyc_in_progress': 'Verification in progress...',
      'ekyc_success': 'Identity verified successfully.',
      'ekyc_failed': 'Verification failed. Please try again.',
      'ekyc_skip': 'Skip for now',
      'key_generation_title': 'Secure Your Wallet',
      'key_generation_subtitle':
          'We are generating a secure key bound to this device.',
      'key_generation_in_progress': 'Generating secure key...',
      'key_generation_success': 'Your wallet is ready.',
      'setup_biometrics': 'Set Up Biometrics',
      'biometrics_required':
          'Biometric authentication protects your credentials.',

      // Credentials
      'credentials_title': 'My Credentials',
      'no_credentials': 'No credentials yet',
      'no_credentials_subtitle':
          'Request a credential from your property administrator.',
      'credential_detail': 'Credential Details',
      'credential_valid': 'Valid',
      'credential_expired': 'Expired',
      'credential_revoked': 'Revoked',
      'credential_expiring_soon': 'Expiring soon',
      'expires_on': 'Expires on',
      'issued_on': 'Issued on',
      'issued_by': 'Issued by',
      'present_credential': 'Present Credential',

      // Credential types
      'resident_badge': 'Resident Badge',
      'visitor_pass': 'Visitor Pass',
      'contractor_badge': 'Contractor Badge',
      'emergency_access': 'Emergency Access',

      // Presentation
      'presentation_title': 'Present Credential',
      'presentation_detecting': 'Detecting verifier...',
      'presentation_connecting': 'Connecting...',
      'presentation_authenticating': 'Authenticate to continue',
      'presentation_sending': 'Sending credential...',
      'presentation_granted': 'Access Granted',
      'presentation_denied': 'Access Denied',
      'presentation_error': 'Presentation failed. Please try again.',
      'tap_to_present': 'Tap to present via NFC',

      // Scanning
      'scanning_title': 'Scanning',
      'scanning_ble': 'Scanning for nearby verifiers...',
      'scanning_or': 'Or',
      'scanning_again': 'Scan Again',
      'access_not_authorized': 'Access not authorized',
      'scanning_found': 'Verifier found',
      'scanning_none': 'No verifiers nearby',
      'scanning_enable_bluetooth': 'Please enable Bluetooth to continue.',
      'scanning_enable_location':
          'Location permission is required for Bluetooth scanning.',

      // Settings
      'settings_title': 'Settings',
      'settings_account': 'Account',
      'settings_security': 'Security',
      'settings_biometrics': 'Biometrics',
      'settings_biometrics_subtitle': 'Require biometrics to present credentials',
      'settings_notifications': 'Notifications',
      'settings_language': 'Language',
      'settings_about': 'About',
      'settings_version': 'Version',
      'settings_logout': 'Log Out',
      'settings_logout_confirm':
          'Are you sure you want to log out? Your credentials will be removed from this device.',

      // Security
      'security_title': 'Security',
      'security_device_bound':
          'Credentials are bound to this device and cannot be transferred.',
      'security_biometric_enabled': 'Biometric authentication is enabled.',
      'security_warning_rooted':
          'Warning: This device may be rooted or jailbroken. Your credentials may be at risk.',

      // Errors
      'error_network': 'Network error. Check your connection and try again.',
      'error_biometric_failed': 'Biometric authentication failed.',
      'error_credential_expired': 'This credential has expired.',
      'error_credential_revoked': 'This credential has been revoked.',
      'error_presentation_timeout': 'Presentation timed out. Please try again.',
      'error_ble_unavailable': 'Bluetooth is not available on this device.',
      'error_nfc_unavailable': 'NFC is not available on this device.',
    },
    'ms': {
      // Common
      'app_name': 'VaultPass',
      'ok': 'OK',
      'cancel': 'Batal',
      'continue': 'Teruskan',
      'done': 'Selesai',
      'retry': 'Cuba Lagi',
      'close': 'Tutup',
      'save': 'Simpan',
      'delete': 'Padam',
      'edit': 'Sunting',
      'loading': 'Memuatkan...',
      'error': 'Ralat',
      'success': 'Berjaya',

      // Onboarding
      'welcome_title': 'Selamat Datang ke VaultPass',
      'welcome_subtitle':
          'Kawalan akses yang memelihara privasi untuk hartanah anda.',
      'get_started': 'Mulakan',
      'registration_title': 'Cipta Akaun',
      'registration_subtitle':
          'Daftar dengan pentadbir hartanah anda untuk bermula.',
      'ekyc_title': 'Sahkan Identiti',
      'ekyc_subtitle':
          'Lengkapkan pengesahan identiti melalui MyDigital ID untuk mengaktifkan kelayakan anda.',
      'ekyc_start': 'Mula Pengesahan',
      'ekyc_in_progress': 'Pengesahan sedang dijalankan...',
      'ekyc_success': 'Identiti disahkan dengan jayanya.',
      'ekyc_failed': 'Pengesahan gagal. Sila cuba lagi.',
      'ekyc_skip': 'Langkau buat masa ini',
      'key_generation_title': 'Lindungi Dompet Anda',
      'key_generation_subtitle':
          'Kami sedang menjana kunci selamat yang terikat pada peranti ini.',
      'key_generation_in_progress': 'Menjana kunci selamat...',
      'key_generation_success': 'Dompet anda sedia.',
      'setup_biometrics': 'Sediakan Biometrik',
      'biometrics_required':
          'Pengesahan biometrik melindungi kelayakan anda.',

      // Credentials
      'credentials_title': 'Kelayakan Saya',
      'no_credentials': 'Tiada kelayakan lagi',
      'no_credentials_subtitle':
          'Minta kelayakan daripada pentadbir hartanah anda.',
      'credential_detail': 'Butiran Kelayakan',
      'credential_valid': 'Sah',
      'credential_expired': 'Tamat Tempoh',
      'credential_revoked': 'Dibatalkan',
      'credential_expiring_soon': 'Akan tamat tempoh',
      'expires_on': 'Tamat tempoh pada',
      'issued_on': 'Dikeluarkan pada',
      'issued_by': 'Dikeluarkan oleh',
      'present_credential': 'Kemukakan Kelayakan',

      // Credential types
      'resident_badge': 'Lencana Penghuni',
      'visitor_pass': 'Pas Pelawat',
      'contractor_badge': 'Lencana Kontraktor',
      'emergency_access': 'Akses Kecemasan',

      // Presentation
      'presentation_title': 'Kemukakan Kelayakan',
      'presentation_detecting': 'Mengesan pengesah...',
      'presentation_connecting': 'Menyambung...',
      'presentation_authenticating': 'Sahkan untuk meneruskan',
      'presentation_sending': 'Menghantar kelayakan...',
      'presentation_granted': 'Akses Dibenarkan',
      'presentation_denied': 'Akses Ditolak',
      'presentation_error': 'Pembentangan gagal. Sila cuba lagi.',
      'tap_to_present': 'Ketik untuk kemukakan melalui NFC',

      // Scanning
      'scanning_title': 'Mengimbas',
      'scanning_ble': 'Mengimbas pengesah berdekatan...',
      'scanning_found': 'Pengesah dijumpai',
      'scanning_none': 'Tiada pengesah berdekatan',
      'scanning_enable_bluetooth': 'Sila aktifkan Bluetooth untuk meneruskan.',
      'scanning_enable_location':
          'Kebenaran lokasi diperlukan untuk pengimbasan Bluetooth.',
      'scanning_or': 'Atau',
      'scanning_again': 'Imbas Semula',
      'access_not_authorized': 'Akses tidak dibenarkan',

      // Settings
      'settings_title': 'Tetapan',
      'settings_account': 'Akaun',
      'settings_security': 'Keselamatan',
      'settings_biometrics': 'Biometrik',
      'settings_biometrics_subtitle':
          'Memerlukan biometrik untuk mengemukakan kelayakan',
      'settings_notifications': 'Pemberitahuan',
      'settings_language': 'Bahasa',
      'settings_about': 'Tentang',
      'settings_version': 'Versi',
      'settings_logout': 'Log Keluar',
      'settings_logout_confirm':
          'Adakah anda pasti mahu log keluar? Kelayakan anda akan dikeluarkan daripada peranti ini.',

      // Security
      'security_title': 'Keselamatan',
      'security_device_bound':
          'Kelayakan terikat pada peranti ini dan tidak boleh dipindahkan.',
      'security_biometric_enabled': 'Pengesahan biometrik diaktifkan.',
      'security_warning_rooted':
          'Amaran: Peranti ini mungkin di-root atau jailbreak. Kelayakan anda mungkin berisiko.',

      // Errors
      'error_network':
          'Ralat rangkaian. Semak sambungan anda dan cuba lagi.',
      'error_biometric_failed': 'Pengesahan biometrik gagal.',
      'error_credential_expired': 'Kelayakan ini telah tamat tempoh.',
      'error_credential_revoked': 'Kelayakan ini telah dibatalkan.',
      'error_presentation_timeout':
          'Pembentangan tamat masa. Sila cuba lagi.',
      'error_ble_unavailable': 'Bluetooth tidak tersedia pada peranti ini.',
      'error_nfc_unavailable': 'NFC tidak tersedia pada peranti ini.',
    },
  };

  String translate(String key) {
    return _localizedValues[locale.languageCode]?[key] ?? key;
  }

  // Common
  String get appName => translate('app_name');
  String get ok => translate('ok');
  String get cancel => translate('cancel');
  String get continueText => translate('continue');
  String get done => translate('done');
  String get retry => translate('retry');
  String get close => translate('close');
  String get save => translate('save');
  String get delete => translate('delete');
  String get edit => translate('edit');
  String get loading => translate('loading');
  String get error => translate('error');
  String get success => translate('success');

  // Onboarding
  String get welcomeTitle => translate('welcome_title');
  String get welcomeSubtitle => translate('welcome_subtitle');
  String get getStarted => translate('get_started');
  String get registrationTitle => translate('registration_title');
  String get registrationSubtitle => translate('registration_subtitle');
  String get ekycTitle => translate('ekyc_title');
  String get ekycSubtitle => translate('ekyc_subtitle');
  String get ekycStart => translate('ekyc_start');
  String get ekycInProgress => translate('ekyc_in_progress');
  String get ekycSuccess => translate('ekyc_success');
  String get ekycFailed => translate('ekyc_failed');
  String get ekycSkip => translate('ekyc_skip');
  String get keyGenerationTitle => translate('key_generation_title');
  String get keyGenerationSubtitle => translate('key_generation_subtitle');
  String get keyGenerationInProgress => translate('key_generation_in_progress');
  String get keyGenerationSuccess => translate('key_generation_success');
  String get setupBiometrics => translate('setup_biometrics');
  String get biometricsRequired => translate('biometrics_required');

  // Credentials
  String get credentialsTitle => translate('credentials_title');
  String get noCredentials => translate('no_credentials');
  String get noCredentialsSubtitle => translate('no_credentials_subtitle');
  String get credentialDetail => translate('credential_detail');
  String get credentialValid => translate('credential_valid');
  String get credentialExpired => translate('credential_expired');
  String get credentialRevoked => translate('credential_revoked');
  String get credentialExpiringSoon => translate('credential_expiring_soon');
  String get expiresOn => translate('expires_on');
  String get issuedOn => translate('issued_on');
  String get issuedBy => translate('issued_by');
  String get presentCredential => translate('present_credential');

  // Credential types
  String get residentBadge => translate('resident_badge');
  String get visitorPass => translate('visitor_pass');
  String get contractorBadge => translate('contractor_badge');
  String get emergencyAccess => translate('emergency_access');

  // Presentation
  String get presentationTitle => translate('presentation_title');
  String get presentationDetecting => translate('presentation_detecting');
  String get presentationConnecting => translate('presentation_connecting');
  String get presentationAuthenticating =>
      translate('presentation_authenticating');
  String get presentationSending => translate('presentation_sending');
  String get presentationGranted => translate('presentation_granted');
  String get presentationDenied => translate('presentation_denied');
  String get presentationError => translate('presentation_error');
  String get tapToPresent => translate('tap_to_present');

  // Scanning
  String get scanningTitle => translate('scanning_title');
  String get scanningBle => translate('scanning_ble');
  String get scanningFound => translate('scanning_found');
  String get scanningNone => translate('scanning_none');
  String get scanningEnableBluetooth => translate('scanning_enable_bluetooth');
  String get scanningEnableLocation => translate('scanning_enable_location');
  String get scanningOr => translate('scanning_or');
  String get scanningAgain => translate('scanning_again');
  String get accessNotAuthorized => translate('access_not_authorized');

  // Settings
  String get settingsTitle => translate('settings_title');
  String get settingsAccount => translate('settings_account');
  String get settingsSecurity => translate('settings_security');
  String get settingsBiometrics => translate('settings_biometrics');
  String get settingsBiometricsSubtitle =>
      translate('settings_biometrics_subtitle');
  String get settingsNotifications => translate('settings_notifications');
  String get settingsLanguage => translate('settings_language');
  String get settingsAbout => translate('settings_about');
  String get settingsVersion => translate('settings_version');
  String get settingsLogout => translate('settings_logout');
  String get settingsLogoutConfirm => translate('settings_logout_confirm');

  // Security
  String get securityTitle => translate('security_title');
  String get securityDeviceBound => translate('security_device_bound');
  String get securityBiometricEnabled =>
      translate('security_biometric_enabled');
  String get securityWarningRooted => translate('security_warning_rooted');

  // Errors
  String get errorNetwork => translate('error_network');
  String get errorBiometricFailed => translate('error_biometric_failed');
  String get errorCredentialExpired => translate('error_credential_expired');
  String get errorCredentialRevoked => translate('error_credential_revoked');
  String get errorPresentationTimeout =>
      translate('error_presentation_timeout');
  String get errorBleUnavailable => translate('error_ble_unavailable');
  String get errorNfcUnavailable => translate('error_nfc_unavailable');
}

class _AppLocalizationsDelegate
    extends LocalizationsDelegate<AppLocalizations> {
  const _AppLocalizationsDelegate();

  @override
  bool isSupported(Locale locale) {
    return ['en', 'ms'].contains(locale.languageCode);
  }

  @override
  Future<AppLocalizations> load(Locale locale) async {
    return AppLocalizations(locale);
  }

  @override
  bool shouldReload(_AppLocalizationsDelegate old) => false;
}
